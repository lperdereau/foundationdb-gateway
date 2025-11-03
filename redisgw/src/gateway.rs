use fdb::FoundationDB;
use redis_protocol::resp2::types::OwnedFrame as Frame;

#[derive(Clone)]
pub struct RedisGateway {
    pub(crate) fdb: FoundationDB,
}

impl RedisGateway {
    pub fn new(fdb: FoundationDB) -> Self {
        Self { fdb }
    }
}

#[allow(dead_code)]
pub trait ListOperations {
    /// Inserts all the specified values at the head of the list stored at key.
    fn lpush(&self, key: &[u8], values: &[&[u8]]) -> impl std::future::Future<Output = Frame> + Send;

    /// Inserts all the specified values at the tail of the list stored at key.
    fn rpush(&self, key: &[u8], values: &[&[u8]]) -> impl std::future::Future<Output = Frame> + Send;

    /// Removes and returns the first element of the list stored at key.
    fn lpop(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Removes and returns the last element of the list stored at key.
    fn rpop(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns the specified elements of the list stored at key.
    fn lrange(&self, key: &[u8], start: isize, stop: isize) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns the element at index in the list stored at key.
    fn lindex(&self, key: &[u8], index: isize) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns the length of the list stored at key.
    fn llen(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;
}

#[allow(dead_code)]
pub trait SetOperations {
    /// Adds the specified members to the set stored at key.
    fn sadd(&self, key: &[u8], members: &[&[u8]]) -> impl std::future::Future<Output = Frame> + Send;

    /// Removes the specified members from the set stored at key.
    fn srem(&self, key: &[u8], members: &[&[u8]]) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns all the members of the set value stored at key.
    fn smembers(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns if member is a member of the set stored at key.
    fn sismember(&self, key: &[u8], member: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns the members of the set resulting from the union of all the given sets.
    fn sunion(&self, keys: &[&[u8]]) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns the members of the set resulting from the intersection of all the given sets.
    fn sinter(&self, keys: &[&[u8]]) -> impl std::future::Future<Output = Frame> + Send;

    /// Returns the members of the set resulting from the difference between the first set and all the successive sets.
    fn sdiff(&self, keys: &[&[u8]]) -> impl std::future::Future<Output = Frame> + Send;
}

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
