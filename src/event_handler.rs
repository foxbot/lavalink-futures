use player::AudioPlayer;

use futures::Future;
use websocket::OwnedMessage;

/// Trait that must be implemented determining what to do on certain events from
/// lavalink, some that will be used to reply to Lavalink with.
pub trait EventHandler {
    /// Tymethod called for forwarding a WebSocket message to Discord.
    fn forward(&mut self, shard_id: u64, message: &str)
        -> Box<Future<Item = Option<OwnedMessage>, Error = ()>>;

    /// Tymethod called for checking if a shard is connected.
    fn is_connected(&mut self, shard_id: u64)
        -> Box<Future<Item = bool, Error = ()>>;

    /// Tymethod called for checking if a guild - and optionally voice channel
    /// - combination is valid.
    fn is_valid(&mut self, guild_id: &str, channel_id: Option<String>)
        -> Box<Future<Item = bool, Error = ()>>;

    /// Tymethod called when a track ends. This is useful for then subsequently
    /// playing a new track.
    fn track_end(&mut self, player: &mut AudioPlayer, track: String, reason: String)
        -> Box<Future<Item = (), Error = ()>>;

    /// Tymethod called when an exception occurs during a track playing.
    fn track_exception(&mut self, player: &mut AudioPlayer, track: String, error: String)
        -> Box<Future<Item = (), Error = ()>>;

    /// Tymethod called when a track is determined as being "stuck" in playing.
    ///
    /// Includes the threshold in milliseconds before a track is detected as
    /// being stuck.
    fn track_stuck(&mut self, player: &mut AudioPlayer, track: String, threshold_ms: i64)
        -> Box<Future<Item = (), Error = ()>>;
}
