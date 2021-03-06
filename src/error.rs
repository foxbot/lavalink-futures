use futures::sync::mpsc::SendError as SyncSendError;
use lavalink::Error as LavalinkError;
use serde_json::Error as JsonError;
use std::cell::BorrowMutError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use websocket::client::ParseError as WebSocketClientParseError;
use websocket::{OwnedMessage, WebSocketError};

/// An error enum wrapping all potential errors that could return from the
/// library's functions.
#[derive(Debug)]
pub enum Error {
    /// A RefCell's data couldn't be mutably borrowed.
    BorrowMut(BorrowMutError),
    /// An error from the `serde_json` crate.
    Json(JsonError),
    /// An error from the `lavalink` crate.
    Lavalink(LavalinkError),
    /// An indicator that something that should have been present wasn't.
    ///
    /// This is basically a representation of `Option::None`.
    None,
    /// A player already existed when one was attempted to be made.
    PlayerAlreadyExists,
    /// There was an error sending a message over the WebSocket sender.
    SyncSend(SyncSendError<OwnedMessage>),
    /// An error from the `websocket` crate.
    WebSocket(WebSocketError),
    /// There was an error while the `websocket` crate was parsing a URI.
    WebSocketClientParse(WebSocketClientParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            BorrowMut(ref inner) => inner.description(),
            Json(ref inner) => inner.description(),
            Lavalink(ref inner) => inner.description(),
            None => "No value found",
            PlayerAlreadyExists => "A player for that guild already exists",
            SyncSend(ref inner) => inner.description(),
            WebSocket(ref inner) => inner.description(),
            WebSocketClientParse(ref inner) => inner.description(),
        }
    }
}

impl From<BorrowMutError> for Error {
    fn from(err: BorrowMutError) -> Self {
        Error::BorrowMut(err)
    }
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        Error::Json(err)
    }
}

impl From<LavalinkError> for Error {
    fn from(err: LavalinkError) -> Self {
        Error::Lavalink(err)
    }
}

impl From<SyncSendError<OwnedMessage>> for Error {
    fn from(err: SyncSendError<OwnedMessage>) -> Self {
        Error::SyncSend(err)
    }
}

impl From<WebSocketError> for Error {
    fn from(err: WebSocketError) -> Self {
        Error::WebSocket(err)
    }
}

impl From<WebSocketClientParseError> for Error {
    fn from(err: WebSocketClientParseError) -> Self {
        Error::WebSocketClientParse(err)
    }
}
