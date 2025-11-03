use redis_protocol::resp2::types::OwnedFrame as Frame;

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
