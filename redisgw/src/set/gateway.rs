use crate::gateway::RedisGateway;
use crate::set::operations::SetOperations;
use redis_protocol::resp2::types::OwnedFrame as Frame;

impl SetOperations for RedisGateway {
    async fn sadd(&self, _key: &[u8], _members: &[&[u8]]) -> Frame {
        unimplemented!()
    }

    async fn srem(&self, _key: &[u8], _members: &[&[u8]]) -> Frame {
        unimplemented!()
    }

    async fn smembers(&self, _key: &[u8]) -> Frame {
        unimplemented!()
    }

    async fn sismember(&self, _key: &[u8], _member: &[u8]) -> Frame {
        unimplemented!()
    }

    async fn sunion(&self, _keys: &[&[u8]]) -> Frame {
        unimplemented!()
    }

    async fn sinter(&self, _keys: &[&[u8]]) -> Frame {
        unimplemented!()
    }

    async fn sdiff(&self, _keys: &[&[u8]]) -> Frame {
        unimplemented!()
    }
}
