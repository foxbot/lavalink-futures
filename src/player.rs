//! Module containing structs for interacting with Lavalink nodes and playing
//! audio for guilds.

use futures::sync::mpsc::Sender as MpscSender;
use futures::Sink;
use lavalink::model::{
    Connect,
    Disconnect,
    IntoWebSocketMessage,
    Pause,
    Play,
    Seek,
    Stop,
    Volume,
};
use std::collections::HashMap;
use websocket::OwnedMessage;
use ::Error;

/// A light wrapper around a hashmap keyed by guild IDs with audio players.
#[derive(Clone, Debug, Default)]
pub struct AudioPlayerManager {
    players: HashMap<u64, AudioPlayer>,
}

impl AudioPlayerManager {
    /// Creates a new default `AudioPlayerManager`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an audio player for the guild of the given ID.
    ///
    /// The `sender` must be a clone of [`Node::user_to_node`].
    ///
    /// It may be preferable to use [`NodeManager::create_player`].
    ///
    /// [`Node::user_to_node`]: ../nodes/struct.Node.html#structfield.user_to_node
    /// [`NodeManager::create_player`]: ../nodes/struct.NodeManager.html#method.create_player
    pub fn create(&mut self, guild_id: u64, sender: MpscSender<OwnedMessage>)
        -> Result<&mut AudioPlayer, Error> {
        if self.players.contains_key(&guild_id) {
            return Err(Error::PlayerAlreadyExists);
        }

        self.players.insert(guild_id, AudioPlayer::new(guild_id, sender));

        Ok(self.players.get_mut(&guild_id).unwrap())
    }

    /// Retrieves an immutable reference to the audio player for the guild, if
    /// it exists.
    pub fn get(&self, guild_id: &u64) -> Option<&AudioPlayer> {
        self.players.get(guild_id)
    }

    /// Retrieves a mutable reference to the audio player for the guild, if it
    /// exists.
    pub fn get_mut(&mut self, guild_id: &u64) -> Option<&mut AudioPlayer> {
        self.players.get_mut(guild_id)
    }

    /// Whether the manager contains a player for the given guild.
    pub fn has(&self, guild_id: &u64) -> bool {
        self.players.contains_key(guild_id)
    }
}

/// A struct containing the state of a guild's audio player.
#[derive(Clone, Debug)]
pub struct AudioPlayer {
    /// The ID of the guild that the player represents.
    pub guild_id: u64,
    /// Whether the player is paused.
    pub paused: bool,
    /// The estimated position of the player.
    pub position: i64,
    sender: MpscSender<OwnedMessage>,
    /// The current time of the player.
    pub time: i64,
    /// The track that the player is playing.
    pub track: Option<String>,
    /// The volume setting, on a scale of 0 to 150.
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
    pub fn new(guild_id: u64, sender: MpscSender<OwnedMessage>) -> Self {
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

    /// Sends a message to Lavalink telling it to join a given guild's voice
    /// channel.
    pub fn join(&mut self, channel_id: u64) -> Result<(), Error> {
        let msg = Connect::new(
            self.guild_id.to_string(),
            channel_id.to_string(),
        ).into_ws_message()?;

        debug!("{:?}", msg);

        self.send(msg)
    }

    /// Sends a message to Lavalink telling it to leave the guild's voice
    /// channel.
    pub fn leave(&mut self) -> Result<(), Error> {
        let msg = Disconnect::new(self.guild_id.to_string()).into_ws_message()?;

        self.send(msg)
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
        let msg = Play::new(
            &self.guild_id.to_string()[..],
            track,
            start_time,
            end_time,
        ).into_ws_message()?;

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
    fn send(&mut self, message: OwnedMessage) -> Result<(), Error> {
        self.sender.start_send(message).map(|_| ()).map_err(From::from)
    }
}
