//! Structures for connecting to and interacting with Lavalink nodes.

mod node;
mod node_manager;

pub use self::node::Node;
pub use self::node_manager::NodeManager;

use lavalink::stats::RemoteStats;

/// Configuration identifying the bot user when connecting to a Lavalink node
/// via [`Node::connect`].
///
/// [`Node::connect`]: struct.Node.html#method.connect
#[derive(Clone, Debug)]
pub struct NodeConfig {
    /// The HTTP server being connected to.
    ///
    /// For example, this may be `http://127.0.0.1:14002`.
    pub http_host: String,
    /// The WebSocket host being connected to.
    ///
    /// For example, this may be `ws://127.0.0.1:14001`.
    pub websocket_host: String,
    /// The ID of the bot user.
    pub user_id: String,
    /// The password used to connect to the Lavalink instance.
    pub password: String,
    /// The number of shards that the Discord client user (bot) is currently
    /// using. This must be the same number of shards as the bot in total.
    pub num_shards: u64,
}

/// State about a node.
#[derive(Clone, Debug, Default)]
pub struct State {
    /// Statistics about the node's load and players, if there is any.
    pub stats: Option<RemoteStats>,
}
