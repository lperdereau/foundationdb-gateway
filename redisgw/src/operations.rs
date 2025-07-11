use redis_protocol::resp2::types::OwnedFrame as Frame;

#[allow(dead_code)]
pub trait RedisOperations {
    /// Responds to a PING command with an optional message.
    async fn ping(&self, message: Vec<&[u8]>) -> Frame;

    /// Sets the value of a key.
    async fn set(&self, key: &[u8], value: &[u8]) -> Frame;

    /// Gets the value of a key.
    async fn get(&self, key: &[u8]) -> Frame;

    /// Deletes a key.
    async fn del(&self, key: &[u8]) -> Frame;

    /// Gets the value of a key and deletes it.
    async fn getdel(&self, key: &[u8]) -> Frame;

    /// Increments the integer value of a key by one.
    async fn incr(&self, key: &[u8]) -> Frame;

    /// Decrements the integer value of a key by one.
    async fn decr(&self, key: &[u8]) -> Frame;

    /// Increments the integer value of a key by the given amount.
    async fn incr_by(&self, key: &[u8], increment: &[u8]) -> Frame;

    /// Decrements the integer value of a key by the given amount.
    async fn decr_by(&self, key: &[u8], decrement: &[u8]) -> Frame;

    /// Appends a value to a key.
    async fn append(&self, key: &[u8], value: &[u8]) -> Frame;

    /// Inserts all the specified values at the head of the list stored at key.
    async fn lpush(&self, key: &[u8], values: &[&[u8]]) -> Frame;

    /// Inserts all the specified values at the tail of the list stored at key.
    async fn rpush(&self, key: &[u8], values: &[&[u8]]) -> Frame;

    /// Removes and returns the first element of the list stored at key.
    async fn lpop(&self, key: &[u8]) -> Frame;

    /// Removes and returns the last element of the list stored at key.
    async fn rpop(&self, key: &[u8]) -> Frame;

    /// Returns the specified elements of the list stored at key.
    async fn lrange(&self, key: &[u8], start: isize, stop: isize) -> Frame;

    /// Returns the element at index in the list stored at key.
    async fn lindex(&self, key: &[u8], index: isize) -> Frame;

    /// Returns the length of the list stored at key.
    async fn llen(&self, key: &[u8]) -> Frame;

    /// Adds the specified members to the set stored at key.
    async fn sadd(&self, key: &[u8], members: &[&[u8]]) -> Frame;

    /// Removes the specified members from the set stored at key.
    async fn srem(&self, key: &[u8], members: &[&[u8]]) -> Frame;

    /// Returns all the members of the set value stored at key.
    async fn smembers(&self, key: &[u8]) -> Frame;

    /// Returns if member is a member of the set stored at key.
    async fn sismember(&self, key: &[u8], member: &[u8]) -> Frame;

    /// Returns the members of the set resulting from the union of all the given sets.
    async fn sunion(&self, keys: &[&[u8]]) -> Frame;

    /// Returns the members of the set resulting from the intersection of all the given sets.
    async fn sinter(&self, keys: &[&[u8]]) -> Frame;

    /// Returns the members of the set resulting from the difference between the first set and all the successive sets.
    async fn sdiff(&self, keys: &[&[u8]]) -> Frame;
}
