#![feature(conservative_impl_trait)]

#[macro_use] extern crate log;

extern crate evzht9h3nznqzwl as websocket;
extern crate futures;
extern crate hyper;
extern crate lavalink;
extern crate serde;
extern crate serde_json;

pub mod nodes;
pub mod player;

mod error;
mod event_handler;

pub use self::error::Error;
pub use self::event_handler::EventHandler;
