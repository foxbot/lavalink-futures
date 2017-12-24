use futures::unsync::mpsc::Sender as UnsyncSender;
use futures::{Future, Sink};
use lavalink::model::{IntoWebSocketMessage, Pause, Play, Seek, Stop, Volume};
use std::collections::HashMap;
use websocket::OwnedMessage;
use ::Error;

#[derive(Clone, Debug, Default)]
pub struct AudioPlayerManager {
    players: HashMap<u64, AudioPlayer>,
}

impl AudioPlayerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, guild_id: u64, sender: UnsyncSender<OwnedMessage>)
        -> Result<&mut AudioPlayer, Error> {
        if self.players.contains_key(&guild_id) {
            return Err(Error::PlayerAlreadyExists);
        }

        self.players.insert(guild_id, AudioPlayer::new(guild_id, sender));

        Ok(self.players.get_mut(&guild_id).unwrap())
    }

    pub fn get(&self, guild_id: &u64) -> Option<&AudioPlayer> {
        self.players.get(guild_id)
    }

    pub fn get_mut(&self, guild_id: &u64) -> Option<&mut AudioPlayer> {
        self.players.get_mut(guild_id)
    }

    pub fn has(&self, guild_id: &u64) -> bool {
        self.players.contains_key(guild_id)
    }
}

#[derive(Clone, Debug)]
pub struct AudioPlayer {
    pub guild_id: u64,
    pub paused: bool,
    pub position: i64,
    sender: UnsyncSender<OwnedMessage>,
    pub time: i64,
    pub track: Option<String>,
    pub volume: i32,
}

impl AudioPlayer {
    /// Creates a new audio player.
    ///
    /// Using [`AudioPlayerManager::create`] or [`NodeManager::create_player`]
    /// is the preferred method of creating a new player.
    ///
    /// [`AudioPlayerManager::create`]: struct.AudioPlayerManager.html#method.create
    /// [`NodeManager::create_player`]: ../nodes/struct.NodeManager.html#method.create_player
    pub fn new(guild_id: u64, sender: UnsyncSender<OwnedMessage>) -> Self {
        Self {
            paused: false,
            position: 0,
            time: 0,
            track: None,
            volume: 100,
            guild_id,
            sender,
        }
    }

    /// Sends a message to Lavalink telling it to either pause or unpause the
    /// player.
    pub fn pause(&mut self, pause: bool) -> Result<(), Error> {
        let msg = Pause::new(&self.guild_id.to_string()[..], pause)
            .into_ws_message()?;

        self.send(msg)
    }

    /// Sends a message to Lavalink telling it to play a track with optional
    /// configuration settings.
    pub fn play(
        &mut self,
        track: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<(), Error> {
        let msg = Play::new(&self.guild_id.to_string()[..], track, start_time, end_time)
            .into_ws_message()?;

        self.send(msg)
    }

    /// Sends a message to Lavalink telling it to seek the player to a certain
    /// position.
    pub fn seek(&mut self, position: i64) -> Result<(), Error> {
        let msg = Seek::new(&self.guild_id.to_string()[..], position)
            .into_ws_message()?;

        self.send(msg)
    }

    /// Sends a message to Lavalink telling it to stop the player.
    pub fn stop(&mut self) -> Result<(), Error> {
        let msg = Stop::new(&self.guild_id.to_string()[..]).into_ws_message()?;

        self.send(msg)
    }

    /// Sends a message to Lavalink telling it to mutate the volume setting.
    pub fn volume(&mut self, volume: i32) -> Result<(), Error> {
        let msg = Volume::new(&self.guild_id.to_string()[..], volume)
            .into_ws_message()?;

        self.send(msg)
    }

    #[inline]
    fn send(&self, message: OwnedMessage) -> Result<(), Error> {
        self.sender.start_send(message).map(|_| ()).map_err(From::from)
    }
}
