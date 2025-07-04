#[allow(dead_code)]
pub trait RedisOperations {
    /// Responds to a PING command with an optional message.
    async fn ping(&self, message: Vec<&[u8]>) -> Vec<u8>;

    /// Sets the value of a key.
    async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), String>;

    /// Gets the value of a key.
    async fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Deletes a key.
    async fn del(&self, key: &[u8]) -> Result<i64, String>;

    /// Gets the value of a key and deletes it.
    async fn getdel(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Increments the integer value of a key by one.
    async fn incr(&self, key: &[u8]) -> Result<i64, String>;

    /// Decrements the integer value of a key by one.
    async fn decr(&self, key: &[u8]) -> Result<i64, String>;

    /// Increments the integer value of a key by the given amount.
    async fn incr_by(&self, key: &[u8], increment: i64) -> Result<i64, String>;

    /// Decrements the integer value of a key by the given amount.
    async fn decr_by(&self, key: &[u8], decrement: i64) -> Result<i64, String>;

    /// Appends a value to a key.
    async fn append(&self, key: &[u8], value: &[u8]) -> Result<usize, String>;

    /// Inserts all the specified values at the head of the list stored at key.
    async fn lpush(&self, key: &[u8], values: &[&[u8]]) -> Result<usize, String>;

    /// Inserts all the specified values at the tail of the list stored at key.
    async fn rpush(&self, key: &[u8], values: &[&[u8]]) -> Result<usize, String>;

    /// Removes and returns the first element of the list stored at key.
    async fn lpop(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Removes and returns the last element of the list stored at key.
    async fn rpop(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Returns the specified elements of the list stored at key.
    async fn lrange(&self, key: &[u8], start: isize, stop: isize) -> Vec<Vec<u8>>;

    /// Returns the element at index in the list stored at key.
    async fn lindex(&self, key: &[u8], index: isize) -> Option<Vec<u8>>;

    /// Returns the length of the list stored at key.
    async fn llen(&self, key: &[u8]) -> usize;

    /// Adds the specified members to the set stored at key.
    async fn sadd(&self, key: &[u8], members: &[&[u8]]) -> Result<usize, String>;

    /// Removes the specified members from the set stored at key.
    async fn srem(&self, key: &[u8], members: &[&[u8]]) -> Result<usize, String>;

    /// Returns all the members of the set value stored at key.
    async fn smembers(&self, key: &[u8]) -> Vec<Vec<u8>>;

    /// Returns if member is a member of the set stored at key.
    async fn sismember(&self, key: &[u8], member: &[u8]) -> bool;

    /// Returns the members of the set resulting from the union of all the given sets.
    async fn sunion(&self, keys: &[&[u8]]) -> Vec<Vec<u8>>;

    /// Returns the members of the set resulting from the intersection of all the given sets.
    async fn sinter(&self, keys: &[&[u8]]) -> Vec<Vec<u8>>;

    /// Returns the members of the set resulting from the difference between the first set and all the successive sets.
    async fn sdiff(&self, keys: &[&[u8]]) -> Vec<Vec<u8>>;
}
