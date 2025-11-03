use crate::gateway::RedisGateway;
use crate::list::operations::ListOperations;
use redis_protocol::resp2::types::OwnedFrame as Frame;

impl ListOperations for RedisGateway {
    async fn lpush(&self, _key: &[u8], _values: &[&[u8]]) -> Frame {
        unimplemented!()
    }

    async fn rpush(&self, _key: &[u8], _values: &[&[u8]]) -> Frame {
        unimplemented!()
    }

    async fn lpop(&self, _key: &[u8]) -> Frame {
        unimplemented!()
    }

    async fn rpop(&self, _key: &[u8]) -> Frame {
        unimplemented!()
    }

    async fn lrange(&self, _key: &[u8], _start: isize, _stop: isize) -> Frame {
        unimplemented!()
    }

    async fn lindex(&self, _key: &[u8], _index: isize) -> Frame {
        unimplemented!()
    }

    async fn llen(&self, _key: &[u8]) -> Frame {
        unimplemented!()
    }
}
