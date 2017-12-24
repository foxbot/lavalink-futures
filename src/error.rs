use futures::unsync::mpsc::SendError as UnsyncSendError;
use lavalink::Error as LavalinkError;
use websocket::client::ParseError as WebSocketClientParseError;
use websocket::{OwnedMessage, WebSocketError};

#[derive(Debug)]
pub enum Error {
    Lavalink(LavalinkError),
    PlayerAlreadyExists,
    UnsyncSend(UnsyncSendError<OwnedMessage>),
    WebSocket(WebSocketError),
    WebSocketClientParse(WebSocketClientParseError),
}

impl From<LavalinkError> for Error {
    fn from(err: LavalinkError) -> Self {
        Error::Lavalink(err)
    }
}

impl From<UnsyncSendError<OwnedMessage>> for Error {
    fn from(err: UnsyncSendError<OwnedMessage>) -> Self {
        Error::UnsyncSend(err)
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
