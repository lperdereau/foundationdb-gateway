use redis_protocol::resp2::types::OwnedFrame as Frame;

#[allow(dead_code)]
pub trait ConnectionOperations {
    /// Responds to a PING command with an optional message.
    async fn ping(&self, message: Vec<&[u8]>) -> Frame;

    /// Echoes the given message.
    async fn echo(&self, message: Vec<&[u8]>) -> Frame;

    /// Minimal HELLO implementation (handshake).
    async fn hello(&self, args: Vec<&[u8]>) -> Frame;

    /// Reset the connection state (stub).
    async fn reset(&self) -> Frame;

    /// Select a database index (stub).
    async fn select(&self, index: &[u8]) -> Frame;

    /// Authenticate (stub).
    async fn auth(&self, username: Option<&[u8]>, password: &[u8]) -> Frame;

    /// CLIENT GETNAME
    async fn client_getname(&self) -> Frame;

    /// CLIENT SETNAME name
    async fn client_setname(&self, name: &[u8]) -> Frame;

    /// QUIT the connection (handler returns reply; server decides to close socket).
    async fn quit(&self) -> Frame;
}
