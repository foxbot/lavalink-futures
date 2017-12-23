use futures::prelude::*;
use futures::unsync::mpsc::{
    self,
    Receiver as UnsyncReceiver,
    SendError as UnsyncSendError,
    Sender as UnsyncSender,
};
use futures::{Future, StartSend, future};
use lavalink::model::{IntoWebSocketMessage, IsConnectedResponse};
use lavalink::player::*;
use lavalink::opcodes::Opcode;
use serde::Deserialize;
use serde_json::{self, Value};
use std::rc::Rc;
use super::{NodeConfig, State};
use websocket::async::Handle;
use websocket::header::Headers;
use websocket::{ClientBuilder, OwnedMessage};
use ::{Error, EventHandler};

pub struct Node {
    pub state: Rc<State>,
    pub user_to_node: UnsyncSender<OwnedMessage>,
    pub user_from_node: UnsyncReceiver<OwnedMessage>,
}

impl Node {
    pub fn connect<'a>(
        handle: &'a Handle,
        config: &'a NodeConfig,
        player_manager: Rc<AudioPlayerManager>,
        mut handler: Rc<Box<EventHandler>>,
    ) -> impl Future<Item = Self, Error = Error> + 'a {
        let mut headers = Headers::new();
        headers.set_raw("Authorization", vec![config.password.clone().into_bytes()]);
        headers.set_raw("Num-Shards", vec![config.num_shards.to_string().into_bytes()]);
        headers.set_raw("User-Id", vec![config.user_id.clone().into_bytes()]);

        let done = future::result(ClientBuilder::new(&config.websocket_host).map_err(From::from))
            .and_then(move |builder| {
                builder.custom_headers(&headers)
                    .async_connect_insecure(handle)
            })
            .map(|(duplex, _)| {
                // user_to_node: user send to node (node handles)
                // node_from_user: node receive from user (user handles)
                // node_to_user: node send to user (node handles)
                // user_from_node: to receive from node sending into user (user handles)
                let (user_to_node, node_from_user) = mpsc::channel(0);
                let (node_to_user, user_from_node) = mpsc::channel(0);

                let (sink, stream) = duplex.split();
                let state = Rc::new(State::default());
                let ws_state = Rc::clone(&state);

                let future = stream
                    .map(move |msg| {
                        match msg {
                            OwnedMessage::Close(data) => {
                                info!("Received a close: {:?}", data);

                                Box::new(future::ok(Some(OwnedMessage::Close(None))))
                            },
                            OwnedMessage::Ping(data) => {
                                debug!("Received a ping: {:?}", data);

                                Box::new(future::ok(Some(OwnedMessage::Pong(data))))
                            },
                            OwnedMessage::Text(data) => {
                                trace!("Received text: {:?}", data);

                                unreachable!();

                                // handle_message(&node_to_user, data.into_bytes(), &mut handler)
                            },
                            OwnedMessage::Binary(data) => {
                                trace!("Received binary: {:?}", data);

                                unreachable!();

                                // handle_message(&node_to_user, data, &mut handler)
                            },
                            OwnedMessage::Pong(data) => {
                                warn!("Received a pong somehow? {:?}", data);

                                Box::new(future::ok(None))
                            },
                        }
                    })
                    .for_each(|future| future.then(|res| sink.send(res)));

                // handle.spawn(future);

                Self {
                    state,
                    user_to_node,
                    user_from_node,
                }
            })
            .map_err(From::from);

        done
    }

    /// Sends a close code over the WebSocket, terminating the connection.
    pub fn close(&mut self)
        -> StartSend<OwnedMessage, UnsyncSendError<OwnedMessage>> {
        self.user_to_node.start_send(OwnedMessage::Close(None))
    }
}

fn handle_message(
    holder: &UnsyncSender<OwnedMessage>,
    bytes: Vec<u8>,
    handler: &mut Rc<Box<EventHandler>>,
) -> Box<Future<Item = Option<OwnedMessage>, Error = ()>> {
    let json = match serde_json::from_slice::<Value>(&bytes) {
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
    //     Opcode::ValidationReq => Self::handle_validation_req,
        // Opcode::IsConnectedReq => handle_is_connected_req(handler, &json),
    //     Opcode::PlayerUpdate => Self::handle_player_update,
    //     Opcode::Stats => Self::handle_stats,
    //     Opcode::Event => Self::handle_event,
        _ => return Box::new(future::ok(None)),
    }
}

fn handle_is_connected_req(handler: &mut Rc<Box<EventHandler>>, json: &Value)
    -> impl Future<Item = Option<OwnedMessage>, Error = ()> {
    let shard_id = json["shardId"].as_u64().unwrap();

    let future = match Rc::get_mut(handler) {
        Some(handler) => handler.is_connected(shard_id),
        None => {
            warn!("Failed to get mutable reference to EventHnalder");

            Box::new(future::ok(true))
        },
    };

    future.map(|connected| {
        Some(IsConnectedResponse::new(shard_id, connected).into_ws_message().unwrap())
    })
}

fn handle_send_ws(handler: &mut Rc<Box<EventHandler>>, json: &Value)
    -> impl Future<Item = Option<OwnedMessage>, Error = ()> {
    let shard_id = json["shardId"].as_u64().unwrap();
    let msg = json["message"].as_str().unwrap();

    match Rc::get_mut(handler) {
        Some(handler) => handler.forward(shard_id, msg),
        None => {
            warn!("Failed to get mutable reference to EventHandler");

            Box::new(future::ok(None))
        },
    }
}
