use crate::gateway::RedisGateway;
use crate::connection::operations::ConnectionOperations;
use redis_protocol::resp2::types::OwnedFrame as Frame;

impl ConnectionOperations for RedisGateway {
    async fn ping(&self, message: Vec<&[u8]>) -> Frame {
        let msg = if let Some(arg) = message.first() {
            std::str::from_utf8(arg).ok()
        } else {
            None
        };

        match msg {
            Some(res) => Frame::SimpleString(res.into()),
            None => Frame::SimpleString(b"PONG".to_vec()),
        }
    }
}
