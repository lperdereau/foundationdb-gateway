use crate::datamodel::SimpleDataModel;
use crate::operations::RedisOperations;
use fdb::FoundationDB;
use redis_protocol::resp2::types::OwnedFrame as Frame;

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
    async fn ping(&self, message: Vec<&[u8]>) -> Frame {
        let msg = if let Some(arg) = message.get(0) {
            std::str::from_utf8(arg).ok()
        } else {
            None
        };

        match msg {
            Some(res) => Frame::SimpleString(res.into()),
            None => Frame::SimpleString(b"PONG".to_vec()),
        }
    }

    async fn set(&self, key: &[u8], value: &[u8]) -> Frame {
        match SimpleDataModel::set(&self.fdb, key, value).await {
            Ok(_) => Frame::SimpleString(b"OK".to_vec()),
            Err(e) => Frame::Error(e.into()),
        }
    }

    async fn get(&self, key: &[u8]) -> Frame {
        match self.fdb.get(key).await {
            Ok(Some(val)) => {
                // Try to interpret as UTF-8, and quote if possible, else just return as-is
                match std::str::from_utf8(&val) {
                    Ok(s) => Frame::SimpleString(format!("\"{}\"", s).into()),
                    Err(e) => Frame::Error(e.to_string().into()),
                }
            }
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e.to_string().into()),
        }
    }

    async fn del(&self, key: &[u8]) -> Frame {
        let db = &self.fdb;
        let val = match db.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        if val.is_none() {
            return Frame::Integer(0i64);
        }
        match db.delete(key).await {
            Ok(val) => Frame::Integer(val),
            Err(e) => Frame::Error(e.to_string().into()),
        }
    }

    async fn getdel(&self, key: &[u8]) -> Frame {
        let db = &self.fdb;
        let val = match db.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        match db.delete(key).await {
            Err(e) => return Frame::Error(e.to_string().into()),
            Ok(_) => (),
        }
        match val {
            Some(val) => match std::str::from_utf8(&val) {
                Ok(s) => Frame::SimpleString(format!("\"{}\"", s).into_bytes()),
                Err(e) => Frame::Error(e.to_string().into()),
            },
            None => Frame::Null,
        }
    }

    async fn incr(&self, key: &[u8]) -> Frame {
        let db = &self.fdb;
        let val = match db.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        let mut n = 0i64;
        if let Some(bytes) = val {
            let s = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => return Frame::Error("Value is not valid UTF-8".into()),
            };
            n = match s.parse::<i64>() {
                Ok(num) => num,
                Err(_) => return Frame::Error("Value is not a valid integer".into()),
            };
        }
        n += 1;
        if let Err(e) = db.set(key, n.to_string().as_bytes()).await {
            return Frame::Error(e.to_string().into());
        }
        Frame::Integer(n)
    }

    async fn decr(&self, key: &[u8]) -> Frame {
        let db = &self.fdb;
        let val = match db.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        let mut n = 0i64;
        if let Some(bytes) = val {
            let s = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => return Frame::Error("Value is not valid UTF-8".into()),
            };
            n = match s.parse::<i64>() {
                Ok(num) => num,
                Err(_) => return Frame::Error("Value is not a valid integer".into()),
            };
        }
        n -= 1;
        if let Err(e) = db.set(key, n.to_string().as_bytes()).await {
            return Frame::Error(e.to_string().into());
        }
        Frame::Integer(n)
    }

    async fn incr_by(&self, key: &[u8], increment: &[u8]) -> Frame {
        let int = match std::str::from_utf8(increment)
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
        {
            None => return Frame::Error("ERR value is not an integer or out of range".into()),
            Some(i) => i,
        };

        let db = &self.fdb;
        let val = match db.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        let mut n = 0i64;
        if let Some(bytes) = val {
            let s = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => return Frame::Error("Value is not valid UTF-8".into()),
            };
            n = match s.parse::<i64>() {
                Ok(num) => num,
                Err(_) => return Frame::Error("Value is not a valid integer".into()),
            };
        }
        n += int;
        if let Err(e) = db.set(key, n.to_string().as_bytes()).await {
            return Frame::Error(e.to_string().into());
        }
        Frame::Integer(n)
    }

    async fn decr_by(&self, key: &[u8], decrement: &[u8]) -> Frame {
        let int = match std::str::from_utf8(decrement)
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
        {
            None => return Frame::Error("ERR value is not an integer or out of range".into()),
            Some(i) => i,
        };

        let db = &self.fdb;
        let val = match db.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        let mut n = 0i64;
        if let Some(bytes) = val {
            let s = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => return Frame::Error("Value is not valid UTF-8".into()),
            };
            n = match s.parse::<i64>() {
                Ok(num) => num,
                Err(_) => return Frame::Error("Value is not a valid integer".into()),
            };
        }
        n -= int;
        if let Err(e) = db.set(key, n.to_string().as_bytes()).await {
            return Frame::Error(e.to_string().into());
        }
        Frame::Integer(n)
    }

    async fn append(&self, key: &[u8], value: &[u8]) -> Frame {
        let db = &self.fdb;
        let current = match db.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        let mut new_value = Vec::new();
        if let Some(existing) = current {
            new_value.extend_from_slice(&existing);
        }
        new_value.extend_from_slice(value);
        let len = new_value.len();
        if let Err(e) = db.set(key, &new_value).await {
            return Frame::Error(e.to_string().into());
        }
        Frame::Integer(len as i64)
    }

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
