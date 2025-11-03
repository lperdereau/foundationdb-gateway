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
use foundationdb::RangeOption;
use foundationdb_tuple::Subspace;
use futures_util::TryStreamExt;
use futures_util::stream::StreamExt;
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
            Ok(Some(val)) => Frame::SimpleString(val),
            Ok(None) => Frame::SimpleString("OK".to_string().into_bytes()),
            Err(e) => Frame::Error(e.to_string()),
        }
    }

    async fn get(&self, key: &[u8]) -> Frame {
        match SimpleDataModel::get(&self.fdb, key).await {
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
            Some(val) => match std::str::from_utf8(&val) {
                Ok(s) => Frame::SimpleString(format!("\"{}\"", s).into_bytes()),
                Err(e) => Frame::Error(e.to_string().into()),
            },
            None => Frame::Null,
        }
    }

    async fn incr(&self, key: &[u8]) -> Frame {
        // Perform read-modify-write in a single FoundationDB transaction to avoid lost updates
        let key_vec = key.to_vec();
        let db = self.fdb.clone();
        let result = db
            .database
            .run(move |trx, _| {
                let key = key_vec.clone();
                async move {
                    // range scan for chunks stored under the key prefix
                    let mut end = key.clone();
                    end.push(0xFF);
                    let range = RangeOption::from((key.clone(), end));
                    let stream = trx.get_ranges_keyvalues(range, false);
                    // collect keyvalues
                    let records = stream
                        .map(|res| match res {
                            Ok(v) => Ok((v.key().to_vec(), v.value().to_vec())),
                            Err(e) => Err(e),
                        })
                        .try_collect::<Vec<(Vec<u8>, Vec<u8>)>>()
                        .await?;

                    // reconstruct current value
                    let mut current = Vec::new();
                    for (_k, v) in &records {
                        current.extend_from_slice(v);
                    }

                    let mut n: i64 = 0;
                    if !current.is_empty() {
                        if let Ok(s) = std::str::from_utf8(&current) {
                            if let Ok(num) = s.parse::<i64>() {
                                n = num;
                            }
                        }
                    }

                    n += 1;

                    // clear old chunks
                    for (k, _) in &records {
                        trx.clear(k);
                    }

                    // write new single chunk at index 0
                    let subspace = Subspace::from_bytes(key.clone());
                    let chunk_key = subspace.pack(&(0,));
                    trx.set(&chunk_key, n.to_string().as_bytes());

                    Ok(n)
                }
            })
            .await;

        match result {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e.to_string().into()),
        }
    }

    async fn decr(&self, key: &[u8]) -> Frame {
        // Transactional decrement to avoid lost updates under concurrency
        let key_vec = key.to_vec();
        let db = self.fdb.clone();
        let result = db
            .database
            .run(move |trx, _| {
                let key = key_vec.clone();
                async move {
                    let mut end = key.clone();
                    end.push(0xFF);
                    let range = RangeOption::from((key.clone(), end));
                    let stream = trx.get_ranges_keyvalues(range, false);
                    let records = stream
                        .map(|res| match res {
                            Ok(v) => Ok((v.key().to_vec(), v.value().to_vec())),
                            Err(e) => Err(e),
                        })
                        .try_collect::<Vec<(Vec<u8>, Vec<u8>)>>()
                        .await?;

                    let mut current = Vec::new();
                    for (_k, v) in &records {
                        current.extend_from_slice(v);
                    }
                    let mut n: i64 = 0;
                    if !current.is_empty() {
                        if let Ok(s) = std::str::from_utf8(&current) {
                            if let Ok(num) = s.parse::<i64>() {
                                n = num;
                            }
                        }
                    }

                    n -= 1;

                    for (k, _) in &records {
                        trx.clear(k);
                    }
                    let subspace = Subspace::from_bytes(key.clone());
                    let chunk_key = subspace.pack(&(0,));
                    trx.set(&chunk_key, n.to_string().as_bytes());
                    Ok(n)
                }
            })
            .await;

        match result {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e.to_string().into()),
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
        // Transactional increment-by
        let key_vec = key.to_vec();
        let db = self.fdb.clone();
        let inc = int;
        let result = db
            .database
            .run(move |trx, _| {
                let key = key_vec.clone();
                async move {
                    let mut end = key.clone();
                    end.push(0xFF);
                    let range = RangeOption::from((key.clone(), end));
                    let stream = trx.get_ranges_keyvalues(range, false);
                    let records = stream
                        .map(|res| match res {
                            Ok(v) => Ok((v.key().to_vec(), v.value().to_vec())),
                            Err(e) => Err(e),
                        })
                        .try_collect::<Vec<(Vec<u8>, Vec<u8>)>>()
                        .await?;

                    let mut current = Vec::new();
                    for (_k, v) in &records {
                        current.extend_from_slice(v);
                    }
                    let mut n: i64 = 0;
                    if !current.is_empty() {
                        if let Ok(s) = std::str::from_utf8(&current) {
                            if let Ok(num) = s.parse::<i64>() {
                                n = num;
                            }
                        }
                    }

                    n += inc;

                    for (k, _) in &records {
                        trx.clear(k);
                    }
                    let subspace = Subspace::from_bytes(key.clone());
                    let chunk_key = subspace.pack(&(0,));
                    trx.set(&chunk_key, n.to_string().as_bytes());
                    Ok(n)
                }
            })
            .await;

        match result {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e.to_string().into()),
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
        // Transactional decrement-by
        let key_vec = key.to_vec();
        let db = self.fdb.clone();
        let dec = int;
        let result = db
            .database
            .run(move |trx, _| {
                let key = key_vec.clone();
                async move {
                    let mut end = key.clone();
                    end.push(0xFF);
                    let range = RangeOption::from((key.clone(), end));
                    let stream = trx.get_ranges_keyvalues(range, false);
                    let records = stream
                        .map(|res| match res {
                            Ok(v) => Ok((v.key().to_vec(), v.value().to_vec())),
                            Err(e) => Err(e),
                        })
                        .try_collect::<Vec<(Vec<u8>, Vec<u8>)>>()
                        .await?;

                    let mut current = Vec::new();
                    for (_k, v) in &records {
                        current.extend_from_slice(v);
                    }
                    let mut n: i64 = 0;
                    if !current.is_empty() {
                        if let Ok(s) = std::str::from_utf8(&current) {
                            if let Ok(num) = s.parse::<i64>() {
                                n = num;
                            }
                        }
                    }

                    n -= dec;

                    for (k, _) in &records {
                        trx.clear(k);
                    }
                    let subspace = Subspace::from_bytes(key.clone());
                    let chunk_key = subspace.pack(&(0,));
                    trx.set(&chunk_key, n.to_string().as_bytes());
                    Ok(n)
                }
            })
            .await;

        match result {
            Ok(n) => Frame::Integer(n),
            Err(e) => Frame::Error(e.to_string().into()),
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
