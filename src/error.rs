use websocket::client::ParseError as WebSocketClientParseError;
use websocket::WebSocketError;

#[derive(Debug)]
pub enum Error {
    WebSocket(WebSocketError),
    WebSocketClientParse(WebSocketClientParseError),
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
