[![ci-badge][]][ci] [![license-badge][]][license] [![docs-badge][]][docs]

# lavalink-futures

`lavalink-futures` is an asynchronous, Futures-based client implementation for
[Lavalink], built on top of [lavalink.rs].

### Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
lavalink-futures = { git = "https://github.com/zeyla/lavalink-futures" }
```

And then at the top of your `main.rs`/`lib.rs`:

```rust
extern crate lavalink_futures;
```

### Examples

Creating a `NodeManager`, connecting to a `Node` with it, and then playing a
track for a guild:

```rust
extern crate lavalink_futures;
extern crate tokio_core;

use lavalink_futures::nodes::{NodeConfig, NodeManager};
use lavalink_futures::EventHandler;
use std::cell::RefCell;
use std::env;
use tokio_core::reactor::Core;

struct Handler {
    // Structfields here...
}

impl Handler {
    pub fn new() -> Self {
        Self {
            // Instantiate struct here...
        }
    }
}

impl EventHandler for Handler {
    // Implement EventHandler tymethods here...
}

let mut core = Core::new()?;

let handler = RefCell::new(Box::new(Handler::new()));
let mut manager = NodeManager::new(core.handle(), handler);

let done = manager.add_node(NodeConfig {
    http_host: env::var("LAVALINK_HTTP_HOST")?,
    num_shards: env::var("DISCORD_SHARD_COUNT")?.parse()?,
    password: env::var("LAVALINK_PASSWORD")?,
    user_id: env::var("DISCORD_USER_ID")?.parse()?,
    websocket_host: env::var("LAVALINK_WS_HOST")?,
}).map(|manager| {
    // Do more with the manager here, such as connecting to more nodes, creating
    // audio players, and/or attaching the node manager to some structure's
    // state.
}).map(|_| ()).map_err(|_| ());

core.run(done).unwrap();
```

### License

[ISC][LICENSE.md].

[ci]: https://travis-ci.org/zeyla/lavalink-futures
[ci-badge]: https://travis-ci.org/zeyla/lavalink-futures.svg?branch=master
[docs]: https://docs.rs/crate/lavalink-futures
[docs-badge]: https://img.shields.io/badge/docs-online-2020ff.svg
[LICENSE.md]: https://github.com/zeyla/lavalink-futures/blob/master/LICENSE.md
[license]: https://opensource.org/licenses/ISC
[license-badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=flat-square
[Lavalink]: https://github.com/Frederikam/Lavalink
[lavalink.rs]: https://github.com/serenity-rs/lavalink.rs
