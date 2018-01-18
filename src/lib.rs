//! # lavalink-futures
//!
//! `lavalink-futures` is an asynchronous, Futures-based client implementation for
//! [Lavalink], built on top of [lavalink.rs].
//!
//! ### Installation
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! lavalink-futures = { git = "https://github.com/zeyla/lavalink-futures" }
//! ```
//!
//! And then at the top of your `main.rs`/`lib.rs`:
//!
//! ```rust,no_run
//! extern crate lavalink_futures;
//! ```
//!
//! ### Examples
//!
//! Creating a `NodeManager`, connecting to a `Node` with it, and then playing a
//! track for a guild:
//!
//! ```rust,no_run
//! # extern crate futures;
//! #
//! extern crate lavalink_futures;
//! extern crate tokio_core;
//!
//! # use futures::{Future, future};
//! # use lavalink_futures::reexports::OwnedMessage;
//! # use std::error::Error;
//! #
//! # fn try_main() -> Result<(), Box<Error>> {
//! #
//! use lavalink_futures::nodes::{NodeConfig, NodeManager};
//! use lavalink_futures::EventHandler;
//! use std::cell::RefCell;
//! use std::env;
//! use tokio_core::reactor::Core;
//!
//! struct Handler {
//!     // Structfields here...
//! }
//!
//! impl Handler {
//!     pub fn new() -> Self {
//!         Self {
//!             // Instantiate struct here...
//!         }
//!     }
//! }
//!
//! impl EventHandler for Handler {
//!     // Implement EventHandler tymethods here...
//! #     fn forward(&mut self, _: u64, _: &str)
//! #         -> Box<Future<Item = Option<OwnedMessage>, Error = ()>> {
//! #         Box::new(future::ok(None))
//! #     }
//! #
//! #     fn is_connected(&mut self, _: u64)
//! #         -> Box<Future<Item = bool, Error = ()>> {
//! #         Box::new(future::ok(true))
//! #     }
//! #
//! #     fn is_valid(&mut self, _: &str, _: Option<String>)
//! #         -> Box<Future<Item = bool, Error = ()>> {
//! #         Box::new(future::ok(true))
//! #     }
//! #
//! #     fn track_end(&mut self, _: String, _: String)
//! #         -> Box<Future<Item = (), Error = ()>> {
//! #         Box::new(future::ok(()))
//! #     }
//! #
//! #     fn track_exception(&mut self, _: String, _: String)
//! #         -> Box<Future<Item = (), Error = ()>> {
//! #         Box::new(future::ok(()))
//! #     }
//! #
//! #     fn track_stuck(&mut self, _: String, _: i64)
//! #         -> Box<Future<Item = (), Error = ()>> {
//! #         Box::new(future::ok(()))
//! #     }
//! }
//!
//! let mut core = Core::new()?;
//!
//! let handler = RefCell::new(Box::new(Handler::new()));
//! let mut manager = NodeManager::new(core.handle(), handler);
//!
//! let done = manager.add_node(NodeConfig {
//!     http_host: env::var("LAVALINK_HTTP_HOST")?,
//!     num_shards: env::var("DISCORD_SHARD_COUNT")?.parse()?,
//!     password: env::var("LAVALINK_PASSWORD")?,
//!     user_id: env::var("DISCORD_USER_ID")?.parse()?,
//!     websocket_host: env::var("LAVALINK_WS_HOST")?,
//! }).map(|manager| {
//!     // Do more with the manager here, such as connecting to more nodes, creating
//!     // audio players, and/or attaching the node manager to some structure's
//!     // state.
//! }).map(|_| ()).map_err(|_| ());
//!
//! core.run(done).unwrap();
//! #     Ok(())
//! # }
//! #
//! # fn main() {
//! #     try_main().unwrap();
//! # }
//! ```
//!
//! [Lavalink]: https://github.com/Frederikam/Lavalink
//! [lavalink.rs]: https://github.com/serenity-rs/lavalink.rs

#![deny(missing_docs)]

#[macro_use] extern crate log;

extern crate evzht9h3nznqzwl as websocket;
extern crate futures;
extern crate hyper;
extern crate lavalink;
extern crate serde;
extern crate serde_json;

pub mod nodes;
pub mod player;
pub mod reexports;

mod error;
mod event_handler;

pub use self::error::Error;
pub use self::event_handler::EventHandler;
