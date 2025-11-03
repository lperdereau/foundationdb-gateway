use redis_protocol::resp2::types::OwnedFrame as Frame;

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
