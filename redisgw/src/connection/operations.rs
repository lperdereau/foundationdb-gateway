use redis_protocol::resp2::types::OwnedFrame as Frame;

#[allow(dead_code)]
pub trait ConnectionOperations {
    /// Responds to a PING command with an optional message.
    async fn ping(&self, message: Vec<&[u8]>) -> Frame;
}
