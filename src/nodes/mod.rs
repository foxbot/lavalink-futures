mod node;
mod node_manager;

pub use self::node::Node;
pub use self::node_manager::NodeManager;

use lavalink::stats::RemoteStats;

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub http_host: String,
    pub websocket_host: String,
    pub user_id: String,
    pub password: String,
    pub num_shards: u64,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub stats: Option<RemoteStats>,
}
