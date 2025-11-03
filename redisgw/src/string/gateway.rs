use crate::string::datamodel::StringDataModel;
use crate::string::operations::{SetFlags, StringOperations};
use redis_protocol::resp2::types::OwnedFrame as Frame;

// Implement StringOperations for RedisGateway in the feature-local module
use crate::gateway::RedisGateway;

impl StringOperations for RedisGateway {
    async fn set(&self, key: &[u8], value: &[u8], extra_args: SetFlags) -> Frame {
        let dm = StringDataModel::new(self.fdb.clone());
        match dm.set(key, value, extra_args).await {
            Ok(Some(val)) => Frame::BulkString(val),
            Ok(None) => Frame::SimpleString("OK".to_string().into_bytes()),
            Err(e) => Frame::Error(e.to_string()),
        }
    }

    async fn get(&self, key: &[u8]) -> Frame {
        let dm = StringDataModel::new(self.fdb.clone());
        match dm.get(key).await {
            Ok(Some(val)) => Frame::BulkString(val),
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e.to_string()),
        }
    }

    async fn del(&self, key: &[u8]) -> Frame {
        let dm = StringDataModel::new(self.fdb.clone());
        let val = match dm.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string()),
        };
        if val.is_none() {
            return Frame::Integer(0i64);
        }
        let dm = StringDataModel::new(self.fdb.clone());
        match dm.delete(key).await {
            Ok(val) => Frame::Integer(val),
            Err(e) => Frame::Error(e.to_string()),
        }
    }

    async fn getdel(&self, key: &[u8]) -> Frame {
        let dm = StringDataModel::new(self.fdb.clone());
        let val = match dm.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string()),
        };
        if let Err(e) = dm.delete(key).await { return Frame::Error(e.to_string()) }
        match val {
            Some(val) => Frame::BulkString(val),
            None => Frame::Null,
        }
    }

    async fn incr(&self, key: &[u8]) -> Frame {
        let dm = StringDataModel::new(self.fdb.clone());
        match dm.atomic_add(key, 1).await {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e),
        }
    }

    async fn decr(&self, key: &[u8]) -> Frame {
        let dm = StringDataModel::new(self.fdb.clone());
        match dm.atomic_add(key, -1).await {
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
        let dm = StringDataModel::new(self.fdb.clone());
        match dm.atomic_add(key, int).await {
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
        let dm = StringDataModel::new(self.fdb.clone());
        match dm.atomic_add(key, -int).await {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e),
        }
    }

    async fn append(&self, key: &[u8], value: &[u8]) -> Frame {
        let dm = StringDataModel::new(self.fdb.clone());
        let current = match dm.get(key).await {
            Ok(v) => v,
            Err(e) => return Frame::Error(e.to_string()),
        };
        let mut new_value = Vec::new();
        if let Some(existing) = current {
            new_value.extend_from_slice(&existing);
        }
        new_value.extend_from_slice(value);
        let len = new_value.len();
        let dm2 = StringDataModel::new(self.fdb.clone());
        if let Err(e) = dm2.set(key, &new_value, SetFlags::default()).await
        {
            return Frame::Error(e.to_string());
        }
        Frame::Integer(len as i64)
    }
}
