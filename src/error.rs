use futures::sync::mpsc::SendError as SyncSendError;
use lavalink::Error as LavalinkError;
use websocket::client::ParseError as WebSocketClientParseError;
use websocket::{OwnedMessage, WebSocketError};

#[derive(Debug)]
pub enum Error {
    Lavalink(LavalinkError),
    None,
    PlayerAlreadyExists,
    SyncSend(SyncSendError<OwnedMessage>),
    WebSocket(WebSocketError),
    WebSocketClientParse(WebSocketClientParseError),
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
