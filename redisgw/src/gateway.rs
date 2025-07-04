use crate::operations::RedisOperations;
use fdb::FoundationDB;

#[derive(Clone)]
pub struct RedisGateway {
    fdb: FoundationDB,
}

impl RedisGateway {
    pub fn new(fdb: FoundationDB) -> Self {
        Self { fdb: fdb }
    }
}

impl RedisOperations for RedisGateway {
    async fn ping(&self, message: Vec<&[u8]>) -> Vec<u8> {
        let msg = if let Some(arg) = message.get(0) {
            std::str::from_utf8(arg).ok()
        } else {
            None
        };

        match msg {
            Some(res) => res.as_bytes().to_vec(),
            None => String::from("PONG").into_bytes(),
        }
    }

    async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.fdb
            .set(key, value)
            .await
            .map_err(|e| format!("FoundationDB set error: {:?}", e))
    }

    async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.fdb.get(key).await {
            Ok(Some(val)) => {
                // Try to interpret as UTF-8, and quote if possible, else just return as-is
                match std::str::from_utf8(&val) {
                    Ok(s) => Some(format!("\"{}\"", s).into_bytes()),
                    Err(_) => Some(val),
                }
            }
            Ok(None) => None,
            Err(_) => None,
        }
    }

    async fn del(&self, key: &[u8]) -> Result<i64, String> {
        let db = &self.fdb;
        let res = db
            .database
            .run(move |trx, _| {
                let key = key.to_vec();
                async move {
                    let val = trx.get(&key, true).await?;
                    if val.is_some() {
                        trx.clear(&key);
                        Ok(1i64)
                    } else {
                        Ok(0i64)
                    }
                }
            })
            .await;
        res.map_err(|e| format!("FoundationDB del error: {:?}", e))
    }

    async fn getdel(&self, key: &[u8]) -> Option<Vec<u8>> {
        let db = &self.fdb;
        let res = db
            .database
            .run(move |trx, _| {
                let key = key.to_vec();
                async move {
                    let val = trx.get(&key, true).await?;
                    if val.is_some() {
                        trx.clear(&key);
                    }
                    // Convert Option<FdbSlice> to Option<Vec<u8>>
                    Ok(val.map(|v| v.to_vec()))
                }
            })
            .await;
        match res {
            Ok(Some(val)) => match std::str::from_utf8(&val) {
                Ok(s) => Some(format!("\"{}\"", s).into_bytes()),
                Err(_) => Some(val),
            },
            Ok(None) => None,
            Err(_) => None,
        }
    }

    async fn incr(&self, key: &[u8]) -> Result<i64, String> {
        let db = &self.fdb;
        let res = db
            .database
            .run(|trx, _| async move {
                let val = trx.get(key, true).await?;
                let mut n = 0i64;
                if let Some(bytes) = val {
                    let s = std::str::from_utf8(&bytes).map_err(|_| {
                        foundationdb::FdbBindingError::CustomError(
                            "Value is not valid UTF-8".into(),
                        )
                    })?;
                    n = s.parse::<i64>().map_err(|_| {
                        foundationdb::FdbBindingError::CustomError(
                            "Value is not a valid integer".into(),
                        )
                    })?;
                }
                n += 1;
                trx.set(key, n.to_string().as_bytes());
                Ok(n)
            })
            .await;
        res.map_err(|e| format!("FoundationDB incr error: {:?}", e))
    }

    async fn decr(&self, key: &[u8]) -> Result<i64, String> {
        let db = &self.fdb;
        let res = db
            .database
            .run(|trx, _| async move {
                let val = trx.get(key, true).await?;
                let mut n = 0i64;
                if let Some(bytes) = val {
                    let s = std::str::from_utf8(&bytes).map_err(|_| {
                        foundationdb::FdbBindingError::CustomError(
                            "Value is not valid UTF-8".into(),
                        )
                    })?;
                    n = s.parse::<i64>().map_err(|_| {
                        foundationdb::FdbBindingError::CustomError(
                            "Value is not a valid integer".into(),
                        )
                    })?;
                }
                n -= 1;
                trx.set(key, n.to_string().as_bytes());
                Ok(n)
            })
            .await;
        res.map_err(|e| format!("FoundationDB decr error: {:?}", e))
    }

    async fn incr_by(&self, key: &[u8], increment: i64) -> Result<i64, String> {
        let db = &self.fdb;
        let res = db
            .database
            .run(move |trx, _| {
                let key = key.to_vec();
                async move {
                    let val = trx.get(&key, true).await?;
                    let mut n = 0i64;
                    if let Some(bytes) = val {
                        let s = std::str::from_utf8(&bytes).map_err(|_| {
                            foundationdb::FdbBindingError::CustomError(
                                "Value is not valid UTF-8".into(),
                            )
                        })?;
                        n = s.parse::<i64>().map_err(|_| {
                            foundationdb::FdbBindingError::CustomError(
                                "Value is not a valid integer".into(),
                            )
                        })?;
                    }
                    n += increment;
                    trx.set(&key, n.to_string().as_bytes());
                    Ok(n)
                }
            })
            .await;
        res.map_err(|e| format!("FoundationDB incr_by error: {:?}", e))
    }

    async fn decr_by(&self, key: &[u8], decrement: i64) -> Result<i64, String> {
        let db = &self.fdb;
        let res = db
            .database
            .run(move |trx, _| {
                let key = key.to_vec();
                async move {
                    let val = trx.get(&key, true).await?;
                    let mut n = 0i64;
                    if let Some(bytes) = val {
                        let s = std::str::from_utf8(&bytes).map_err(|_| {
                            foundationdb::FdbBindingError::CustomError(
                                "Value is not valid UTF-8".into(),
                            )
                        })?;
                        n = s.parse::<i64>().map_err(|_| {
                            foundationdb::FdbBindingError::CustomError(
                                "Value is not a valid integer".into(),
                            )
                        })?;
                    }
                    n -= decrement;
                    trx.set(&key, n.to_string().as_bytes());
                    Ok(n)
                }
            })
            .await;
        res.map_err(|e| format!("FoundationDB decr_by error: {:?}", e))
    }

    async fn append(&self, key: &[u8], value: &[u8]) -> Result<usize, String> {
        let db = &self.fdb;
        let res = db
            .database
            .run(move |trx, _| {
                let key = key.to_vec();
                let value = value.to_vec();
                async move {
                    let current = trx.get(&key, true).await?;
                    let mut new_value = Vec::new();
                    if let Some(existing) = current {
                        new_value.extend_from_slice(&existing);
                    }
                    new_value.extend_from_slice(&value);
                    trx.set(&key, &new_value);
                    Ok(new_value.len())
                }
            })
            .await;
        res.map_err(|e| format!("FoundationDB append error: {:?}", e))
    }

    async fn lpush(&self, _key: &[u8], _values: &[&[u8]]) -> Result<usize, String> {
        unimplemented!()
    }

    async fn rpush(&self, _key: &[u8], _values: &[&[u8]]) -> Result<usize, String> {
        unimplemented!()
    }

    async fn lpop(&self, _key: &[u8]) -> Option<Vec<u8>> {
        unimplemented!()
    }

    async fn rpop(&self, _key: &[u8]) -> Option<Vec<u8>> {
        unimplemented!()
    }

    async fn lrange(&self, _key: &[u8], _start: isize, _stop: isize) -> Vec<Vec<u8>> {
        unimplemented!()
    }

    async fn lindex(&self, _key: &[u8], _index: isize) -> Option<Vec<u8>> {
        unimplemented!()
    }

    async fn llen(&self, _key: &[u8]) -> usize {
        unimplemented!()
    }

    async fn sadd(&self, _key: &[u8], _members: &[&[u8]]) -> Result<usize, String> {
        unimplemented!()
    }

    async fn srem(&self, _key: &[u8], _members: &[&[u8]]) -> Result<usize, String> {
        unimplemented!()
    }

    async fn smembers(&self, _key: &[u8]) -> Vec<Vec<u8>> {
        unimplemented!()
    }

    async fn sismember(&self, _key: &[u8], _member: &[u8]) -> bool {
        unimplemented!()
    }

    async fn sunion(&self, _keys: &[&[u8]]) -> Vec<Vec<u8>> {
        unimplemented!()
    }

    async fn sinter(&self, _keys: &[&[u8]]) -> Vec<Vec<u8>> {
        unimplemented!()
    }

    async fn sdiff(&self, _keys: &[&[u8]]) -> Vec<Vec<u8>> {
        unimplemented!()
    }
}
