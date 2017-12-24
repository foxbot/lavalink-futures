use futures::unsync::mpsc::Sender as UnsyncSender;
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

    pub fn pause(&mut self, pause: bool) -> Result<(), ()> {
        unimplemented!();
    }

    pub fn play(
        &mut self,
        track: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<(), ()> {
        unimplemented!();
    }

    pub fn seek(&mut self) -> Result<(), ()> {
        unimplemented!();
    }

    pub fn stop(&mut self) -> Result<(), ()> {
        unimplemented!();
    }

    pub fn volume(&mut self, volume: i32) -> Result<(), ()> {
        unimplemented!();
    }
}
