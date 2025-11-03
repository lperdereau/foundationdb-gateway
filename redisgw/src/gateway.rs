use crate::datamodel::SimpleDataModel;
use crate::operations::{
    Flags,
    SetFlags,
    ConnectionOperations,
    StringOperations,
    ListOperations,
    SetOperations,
};
use fdb::FoundationDB;
// transactional logic moved to SimpleDataModel
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

impl ConnectionOperations for RedisGateway {
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
}

impl StringOperations for RedisGateway {
    async fn set(&self, key: &[u8], value: &[u8], extra_args: Flags) -> Frame {
        let args = match extra_args {
            Flags::Set(set_flags) => set_flags,
            Flags::None => SetFlags::default(),
        };
        match SimpleDataModel::set(&self.fdb, key, value, args).await {
            Ok(Some(val)) => Frame::BulkString(val),
            Ok(None) => Frame::SimpleString("OK".to_string().into_bytes()),
            Err(e) => Frame::Error(e.to_string()),
        }
    }

    async fn get(&self, key: &[u8]) -> Frame {
        match SimpleDataModel::get(&self.fdb, key).await {
            Ok(Some(val)) => Frame::BulkString(val),
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e.to_string().into()),
        }
    }

    async fn del(&self, key: &[u8]) -> Frame {
        let val = match SimpleDataModel::get(&self.fdb, key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        if val.is_none() {
            return Frame::Integer(0i64);
        }
        match SimpleDataModel::delete(&self.fdb, key).await {
            Ok(val) => Frame::Integer(val),
            Err(e) => Frame::Error(e.to_string().into()),
        }
    }

    async fn getdel(&self, key: &[u8]) -> Frame {
        let val = match SimpleDataModel::get(&self.fdb, key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        match SimpleDataModel::delete(&self.fdb, key).await {
            Err(e) => return Frame::Error(e.to_string().into()),
            Ok(_) => (),
        }
        match val {
            Some(val) => Frame::BulkString(val),
            None => Frame::Null,
        }
    }

    async fn incr(&self, key: &[u8]) -> Frame {
        match SimpleDataModel::atomic_add(&self.fdb, key, 1).await {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e),
        }
    }

    async fn decr(&self, key: &[u8]) -> Frame {
        match SimpleDataModel::atomic_add(&self.fdb, key, -1).await {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e),
        }
    }

    async fn incr_by(&self, key: &[u8], increment: &[u8]) -> Frame {
        let int = match std::str::from_utf8(increment)
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
        {
            None => return Frame::Error("ERR value is not an integer or out of range".into()),
            Some(i) => i,
        };
        match SimpleDataModel::atomic_add(&self.fdb, key, int).await {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e),
        }
    }

    async fn decr_by(&self, key: &[u8], decrement: &[u8]) -> Frame {
        let int = match std::str::from_utf8(decrement)
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
        {
            None => return Frame::Error("ERR value is not an integer or out of range".into()),
            Some(i) => i,
        };
        match SimpleDataModel::atomic_add(&self.fdb, key, -int).await {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e),
        }
    }

    async fn append(&self, key: &[u8], value: &[u8]) -> Frame {
        let current = match SimpleDataModel::get(&self.fdb, key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string().into()),
        };
        let mut new_value = Vec::new();
        if let Some(existing) = current {
            new_value.extend_from_slice(&existing);
        }
        new_value.extend_from_slice(value);
        let len = new_value.len();
        if let Err(e) = SimpleDataModel::set(&self.fdb, key, &new_value, SetFlags::default()).await
        {
            return Frame::Error(e.to_string().into());
        }
        Frame::Integer(len as i64)
    }
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
