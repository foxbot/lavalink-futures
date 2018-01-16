use futures::Future;
use websocket::OwnedMessage;

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

    fn track_end(&mut self, track: String, reason: String)
        -> Box<Future<Item = (), Error = ()>>;

    fn track_exception(&mut self, track: String, error: String)
        -> Box<Future<Item = (), Error = ()>>;

    fn track_stuck(&mut self, track: String, threshold_ms: i64)
        -> Box<Future<Item = (), Error = ()>>;
}
