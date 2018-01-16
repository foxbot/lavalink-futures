use futures::prelude::*;
use futures::sync::mpsc::{
    self,
    Receiver as SyncReceiver,
    SendError as SyncSendError,
    Sender as SyncSender,
};
use futures::{Future, StartSend, future};
use lavalink::model::{IntoWebSocketMessage, IsConnectedResponse, ValidationResponse};
use lavalink::opcodes::Opcode;
use serde::Deserialize;
use serde_json::{self, Value};
use std::cell::RefCell;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::rc::Rc;
use super::{NodeConfig, State};
use websocket::async::Handle;
use websocket::header::Headers;
use websocket::{ClientBuilder, OwnedMessage, WebSocketError};
use ::player::*;
use ::{Error, EventHandler};

pub struct Node {
    pub state: Rc<RefCell<State>>,
    pub user_to_node: SyncSender<OwnedMessage>,
    pub user_from_node: SyncReceiver<OwnedMessage>,
}

impl Node {
    pub fn connect(
        handle: Handle,
        config: NodeConfig,
        player_manager: Rc<RefCell<AudioPlayerManager>>,
        handler: Rc<RefCell<Box<EventHandler>>>,
    ) -> impl Future<Item = Self, Error = Error> {
        let mut headers = Headers::new();
        headers.set_raw("Authorization", vec![config.password.clone().into_bytes()]);
        headers.set_raw("Num-Shards", vec![config.num_shards.to_string().into_bytes()]);
        headers.set_raw("User-Id", vec![config.user_id.clone().into_bytes()]);

        let handle2 = handle.clone();
        let handle3 = handle.clone();
        future::result(ClientBuilder::new(&config.websocket_host).map_err(From::from))
            .and_then(move |builder| {
                trace!(
                    "Building node WS client & connecting: {}",
                    config.websocket_host,
                );

                builder.custom_headers(&headers)
                    .async_connect_insecure(&handle2)
            })
            .map(move |(duplex, _)| {
                trace!("Node WS client connected");

                // user_to_node: user send to node (node handles)
                // node_from_user: node receive from user (user handles)
                // node_to_user: node send to user (node handles)
                // user_from_node: to receive from node sending into user (user handles)
                let (user_to_node, node_from_user) = mpsc::channel(0);
                let (node_to_user, user_from_node) = mpsc::channel(0);

                let (sink, stream) = duplex.split();
                let state = Rc::new(RefCell::new(State::default()));
                let ws_state = Rc::clone(&state);
                let (sink_tx, sink_rx) = mpsc::unbounded();

                let future = stream
                    .map(move |msg| (msg, sink_tx.clone()))
                    .filter_map(move |(msg, sink_tx)| {
                        // todo: optimize
                        match msg {
                            OwnedMessage::Close(data) => {
                                info!("Received a close: {:?}", data);

                                Some(OwnedMessage::Close(None))
                            },
                            OwnedMessage::Ping(data) => {
                                debug!("Received a ping: {:?}", data);

                                Some(OwnedMessage::Pong(data))
                            },
                            OwnedMessage::Text(data) => {
                                trace!("Received text: {:?}", data);

                                let done = handle_message(
                                    &node_to_user,
                                    data.as_bytes(),
                                    Rc::clone(&handler),
                                    &Rc::clone(&ws_state),
                                    Rc::clone(&player_manager),
                                ).map(move |msg_| {
                                    if let Some(msg) = msg_ {
                                        if let Err(why) = sink_tx.unbounded_send(msg) {
                                            warn!(
                                                "Err sending to sink: {:?}",
                                                why,
                                            );
                                        }
                                    }

                                    ()
                                }).map_err(|_| ());

                                handle3.spawn(done);

                                None
                            },
                            OwnedMessage::Binary(data) => {
                                trace!("Received binary: {:?}", data);

                                let done = handle_message(
                                    &node_to_user,
                                    &data,
                                    Rc::clone(&handler),
                                    &Rc::clone(&ws_state),
                                    Rc::clone(&player_manager),
                                ).map(move |msg| {
                                    if let Some(msg) = msg {
                                        if let Err(why) = sink_tx.unbounded_send(msg) {
                                            warn!(
                                                "Err sending to sink: {:?}",
                                                why,
                                            );
                                        }
                                    }

                                    ()
                                }).map_err(|_| ());

                                handle3.spawn(done);

                                None
                            },
                            OwnedMessage::Pong(data) => {
                                warn!("Received a pong somehow? {:?}", data);

                                None
                            },
                        }
                    })
                    .select(node_from_user.map_err(|why| {
                        warn!("Err selceting node_from_user: {:?}", why);

                        WebSocketError::IoError(IoError::new(
                            IoErrorKind::Other,
                            "This should be unreachable",
                        ))
                    }))
                    .select(sink_rx.map_err(|why| {
                        warn!("Err selecting sink_rx: {:?}", why);

                        WebSocketError::IoError(IoError::new(
                            IoErrorKind::Other,
                            "This should be unreachable",
                        ))
                    }))
                    .map(|msg| {
                        debug!("msg: {:?}", msg);

                        msg
                    })
                    // .map(move |future| {
                    //     (future, handle3.clone(), sink_tx.clone())
                    // })
                    // .for_each(|(future, handle, sink_tx)| {
                    //     future.and_then(move |res| {
                    //         res.and_then(|msg| {
                    //             handle.spawn(sink_tx.send(msg).map(|_| ()).map_err(|_| ()));
                    //             Some(())
                    //         });
                    //         Box::new(future::done(Ok(())))
                    //     })
                    //     .map_err(|_| {
                    //         IoError::new(
                    //             IoErrorKind::Other,
                    //             "This should be unreachable",
                    //         )
                    //     })
                    //     .from_err()
                    // })
                    .forward(sink)
                    .map(|_| ())
                    .map_err(|_| ());

                handle.spawn(future);

                Self {
                    state,
                    user_to_node,
                    user_from_node,
                }
            })
            .from_err()
    }

    /// Sends a close code over the WebSocket, terminating the connection.
    ///
    /// **Note**: This does _not_ remove it from the manager operating the node.
    /// Prefer to close nodes via the manager.
    pub fn close(&mut self)
        -> StartSend<OwnedMessage, SyncSendError<OwnedMessage>> {
        self.user_to_node.start_send(OwnedMessage::Close(None))
    }

    /// Calculates the penalty of the node.
    ///
    /// Returns `None` if the internal [`state`] could not be accessed at the
    /// time or if there are not yet any stats. The state should never be
    /// inaccessible by only the library's usage, so you should be cautious
    /// about accessing it.
    pub fn penalty(&self) -> Option<i32> {
        let state = self.state.borrow();
        let stats = state.stats.as_ref()?;

        let cpu = 1.05f64.powf(100f64 * stats.cpu.system_load) * 10f64 - 10f64;

        let (deficit_frame, null_frame) = match stats.frame_stats.as_ref() {
            Some(frame_stats) => {
                (
                    1.03f64.powf(500f64 * (f64::from(frame_stats.deficit) / 3000f64)) * 300f64 - 300f64,
                    (1.03f64.powf(500f64 * (f64::from(frame_stats.nulled) / 3000f64)) * 300f64 - 300f64) * 2f64,
                )
            },
            None => (0f64, 0f64),
        };

        Some(stats.playing_players + cpu as i32 + deficit_frame as i32 + null_frame as i32)
    }
}

fn handle_message(
    // todo: why is this not needed?
    _: &SyncSender<OwnedMessage>,
    bytes: &[u8],
    handler: Rc<RefCell<Box<EventHandler>>>,
    state: &Rc<RefCell<State>>,
    mut player_manager: Rc<RefCell<AudioPlayerManager>>,
) -> Box<Future<Item = Option<OwnedMessage>, Error = ()>> {
    let json = match serde_json::from_slice::<Value>(bytes) {
        Ok(json) => json,
        Err(why) => {
            warn!("Error parsing received JSON: {:?}", why);

            return Box::new(future::ok(None));
        },
    };

    let op = match json.get("op").map(Opcode::deserialize) {
        Some(Ok(op)) => op,
        Some(Err(why)) => {
            warn!("Failed to deserialize opcode: {:?}", why);

            return Box::new(future::ok(None));
        },
        None => {
            warn!("No opcode present in payload: {:?}", json);

            return Box::new(future::ok(None));
        },
    };

    match op {
        Opcode::SendWS => {
            Box::new(handle_send_ws(handler, &json))
        },
        Opcode::ValidationReq => Box::new(handle_validation_req(handler, &json)),
        Opcode::IsConnectedReq => Box::new(handle_is_connected_req(handler, &json)),
        Opcode::PlayerUpdate => Box::new(handle_player_update(&json, &mut player_manager)),
        Opcode::Stats => Box::new(handle_state(handler, json, state)),
        Opcode::Event => Box::new(handle_event(handler, &json, &mut player_manager)),
        _ => Box::new(future::ok(None)),
    }
}

fn handle_event(handler: Rc<RefCell<Box<EventHandler>>>, json: &Value, player_manager: &mut Rc<RefCell<AudioPlayerManager>>)
    -> Box<Future<Item = Option<OwnedMessage>, Error = ()>> {
    let guild_id_str = json["guildId"]
        .as_str()
        .expect("invalid json guildId - should be str");
    let guild_id = guild_id_str
        .parse::<u64>()
        .expect("could not parse json guild_id into u64");
    let track = json["track"]
        .as_str()
        .expect("invalid json track - should be str");

    let player_manager = match Rc::get_mut(player_manager) {
        Some(player) => player,
        None => {
            warn!("Failed to get mutable reference to player manager");

            return Box::new(future::ok(None));
        },
    };

    let mut player_manager = player_manager.borrow_mut();

    let player = match player_manager.get_mut(&guild_id) {
        Some(player) => player,
        None => {
            warn!(
                "got invalid audio player update for guild {:?}",
                guild_id,
            );

            return Box::new(future::ok(None));
        }
    };

    match json["type"].as_str().expect("Err parsing type to str") {
        "TrackEndEvent" => {
            let reason = json["reason"]
                .as_str()
                .expect("invalid json reason - should be str");

            // Set the player's track so nothing is playing, reset
            // the time, and reset the position
            player.track = None;
            player.time = 0;
            player.position = 0;

            Box::new(handler.borrow_mut().track_end(
                track.to_owned(),
                reason.to_owned(),
            ).map(|_| None))
        },
        "TrackExceptionEvent" => {
            let error = json["error"]
                .as_str()
                .expect("invalid json error - should be str");

            // TODO: determine if should keep playing

            Box::new(handler.borrow_mut().track_exception(track.to_owned(), error.to_owned())
                .map(|_| None))
        },
        "TrackStuckEvent" => {
            let threshold_ms = json["thresholdMs"]
                .as_i64()
                .expect("invalid json thresholdMs - should be i64");

            Box::new(handler.borrow_mut().track_stuck(
                track.to_owned(),
                threshold_ms,
            ).map(|_| None))
        },
        other => {
            warn!("Unexpected event type: {}", other);

            Box::new(future::ok(None))
        },
    }
}

fn handle_is_connected_req(handler: Rc<RefCell<Box<EventHandler>>>, json: &Value)
    -> impl Future<Item = Option<OwnedMessage>, Error = ()> {
    let shard_id = json["shardId"].as_u64().unwrap();

    Box::new(handler.borrow_mut().is_connected(shard_id)
        .map(move |connected| {
            IsConnectedResponse::new(shard_id, connected).into_ws_message().ok()
        }))
}

fn handle_player_update(json: &Value, player_manager: &mut Rc<RefCell<AudioPlayerManager>>)
    -> impl Future<Item = Option<OwnedMessage>, Error = ()> {
    let guild_id_str = json["guildId"].as_str().unwrap();
    let guild_id = guild_id_str.parse::<u64>().unwrap();
    let state = json["state"].as_object().unwrap();
    let time = state["time"].as_i64().unwrap();
    let position = state["position"].as_i64().unwrap();

    let player_manager = match Rc::get_mut(player_manager) {
        Some(player) => player,
        None => {
            warn!("Failed to get mutable reference to player manager");

            return future::ok(None);
        },
    };

    let mut player_manager = player_manager.borrow_mut();

    match player_manager.get_mut(&guild_id) {
        Some(player) => {
            player.time = time;
            player.position = position;
        },
        None => {
            warn!("Invalid player update received for guild {}", guild_id);
        },
    }

    future::ok(None)
}

fn handle_send_ws(handler: Rc<RefCell<Box<EventHandler>>>, json: &Value)
    -> impl Future<Item = Option<OwnedMessage>, Error = ()> {
    let shard_id = json["shardId"].as_u64().unwrap();
    let msg = json["message"].as_str().unwrap();

    handler.borrow_mut().forward(shard_id, msg)
}

// todo: should this be needed?
fn handle_state(_: Rc<RefCell<Box<EventHandler>>>, json: Value, state: &Rc<RefCell<State>>)
    -> impl Future<Item = Option<OwnedMessage>, Error = ()> {

    match serde_json::from_value(json) {
        Ok(parsed) => {
            state.borrow_mut().stats = Some(parsed);
        },
        Err(why) => {
            warn!("Failed to deserialize state payload: {:?}", why);
        },
    }

    future::ok(None)
}

fn handle_validation_req(handler: Rc<RefCell<Box<EventHandler>>>, json: &Value)
    -> impl Future<Item = Option<OwnedMessage>, Error = ()> {
    let guild_id_str = json["guildId"]
        .as_str()
        .unwrap()
        .to_owned();
    let channel_id_str = json["channelId"].as_str().map(|x| x.to_owned());

    Box::new(handler.borrow_mut().is_valid(
        &guild_id_str,
        channel_id_str.clone(),
    ).map(|valid| {
        ValidationResponse::new(guild_id_str, channel_id_str, valid)
            .into_ws_message()
            .ok()
    }))
}
