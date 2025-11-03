use redis_protocol::resp2::types::OwnedFrame as Frame;

pub trait StringOperations {
    /// Sets the value of a key.
    fn set(&self, key: &[u8], value: &[u8], extra_args: super::types::SetFlags) -> impl std::future::Future<Output = Frame> + Send;

    /// Gets the value of a key.
    fn get(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Deletes a key.
    fn del(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Gets the value of a key and deletes it.
    fn getdel(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Increments the integer value of a key by one.
    fn incr(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Decrements the integer value of a key by one.
    fn decr(&self, key: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Increments the integer value of a key by the given amount.
    fn incr_by(&self, key: &[u8], increment: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Decrements the integer value of a key by the given amount.
    fn decr_by(&self, key: &[u8], decrement: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    /// Appends a value to a key.
    fn append(&self, key: &[u8], value: &[u8]) -> impl std::future::Future<Output = Frame> + Send;
}
