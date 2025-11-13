use crate::string::operations::{SetFlags, SetMethod, SetTTL};
use fdb::FoundationDB;
use foundationdb_tuple::{TupleDepth, TuplePack, VersionstampOffset, pack};
use foundationdb::RangeOption;
use foundationdb_tuple::Subspace;
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StringPrefix {
    Data = 11,
    Ttl = 12,
}

impl TuplePack for StringPrefix {
    fn pack<W: Write>(
        &self,
        w: &mut W,
        tuple_depth: TupleDepth,
    ) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}

pub struct StringDataModel {
    pub fdb: FoundationDB,
}

impl StringDataModel {
    pub fn new(fdb: FoundationDB) -> Self {
        Self { fdb }
    }

    pub async fn set(
        &self,
        key: &[u8],
        value: &[u8],
        flags: SetFlags,
    ) -> Result<Option<Vec<u8>>, String> {
        let packed_key = pack(&(StringPrefix::Data, key));
        let mut old_val = None;

        let existing = if flags.get || flags.method.is_some() {
            self.get(key)
                .await
                .map_err(|e| format!("FoundationDB set error: {:?}", e))?
        } else {
            None
        };

        if flags.get {
            old_val = existing.clone();
        }

        if let Some(method) = &flags.method {
            match method {
                SetMethod::NX => {
                    if existing.is_some() {
                        return Ok(None);
                    }
                }
                SetMethod::XX => {
                    if existing.is_none() {
                        return Ok(None);
                    }
                }
            }
        }

        let set_res = self.fdb.set(&packed_key, value).await;
        if let Err(e) = set_res {
            return Err(format!("FoundationDB set error: {:?}", e));
        }

        if let Some(ttl_option) = &flags.ttl {
            let ttl = ttl_option
                .unix_epoch_in_ms()
                .map_err(|e| format!("FoundationDB set_ttl error: {:?}", e))?;
            if *ttl_option != SetTTL::KeepTTL {
                self.set_ttl(key, ttl)
                    .await
                    .map_err(|e| format!("FoundationDB set_ttl error: {:?}", e))?;
            }
        }

        Ok(old_val)
    }

    pub async fn set_ttl(&self, key: &[u8], ttl: u128) -> Result<(), String> {
        let packed_key = pack(&(StringPrefix::Ttl, key));
        let ttl_bytes = ttl.to_be_bytes();
        self.fdb
            .set(&packed_key, &ttl_bytes)
            .await
            .map_err(|e| format!("FoundationDB set_ttl error: {:?}", e))
    }


    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let packed_key = pack(&(StringPrefix::Data, key));
        // Read value and TTL (if any). If TTL exists and is expired, delete both and return None.
        let value = self
            .fdb
            .get(&packed_key)
            .await
            .map_err(|e| format!("FoundationDB get error: {:?}", e))?;

        // If there's no value, nothing to do.
        if value.is_none() {
            return Ok(None);
        }

        // Check TTL
        let packed_ttl_key = pack(&(StringPrefix::Ttl, key));
        let ttl_bytes_opt = self
            .fdb
            .get(&packed_ttl_key)
            .await
            .map_err(|e| format!("FoundationDB get ttl error: {:?}", e))?;

        if let Some(ttl_bytes) = ttl_bytes_opt {
            // TTL stored as big-endian u128
            if ttl_bytes.len() == 16 {
                let mut arr = [0u8; 16];
                arr.copy_from_slice(&ttl_bytes[..16]);
                let ttl = u128::from_be_bytes(arr);

                let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(d) => d.as_millis(),
                    Err(_) => 0u128,
                };

                if ttl != 0 && ttl <= now {
                    // expired -> delete both keys
                    let _ = self
                        .fdb
                        .delete(&packed_key)
                        .await
                        .map_err(|e| format!("FoundationDB delete expired error: {:?}", e))?;
                    let _ = self
                        .fdb
                        .delete(&packed_ttl_key)
                        .await
                        .map_err(|e| format!("FoundationDB delete expired ttl error: {:?}", e))?;
                    return Ok(None);
                }
            }
        }

        Ok(value)
    }

    pub async fn delete(&self, key: &[u8]) -> Result<i64, String> {
        let packed_key = pack(&(StringPrefix::Data, key));
        let packed_ttl_key = pack(&(StringPrefix::Ttl, key));
        let r1 = self.fdb.delete(&packed_key).await;
        let r2 = self.fdb.delete(&packed_ttl_key).await;


        if let Err(e) = r1 {
            return Err(format!("FoundationDB delete error: {:?}", e));
        }
        if let Err(e) = r2 {
            return Err(format!("FoundationDB delete ttl error: {:?}", e));
        }

        Ok(1)
    }

    /// Atomically add `delta` to integer value stored at `key`.
    /// Returns new value on success, or Err(String) on parse/FDB error.
    pub async fn atomic_add(&self, key: &[u8], delta: i64) -> Result<i64, String> {
        let packed_key = pack(&(StringPrefix::Data, key));
        let pk = packed_key.clone();
        let db = self.fdb.clone();

        let res = db
            .database
            .run(move |trx, _| {
                let key = pk.clone();
                async move {
                    // range scan for chunks under packed key
                    let mut end = key.clone();
                    end.push(0xFF);
                    let range = RangeOption::from((key.clone(), end));
                    let stream = trx.get_ranges_keyvalues(range, false);
                    let records = stream
                        .map(|r| match r {
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
                        match std::str::from_utf8(&current) {
                            Ok(s) => match s.parse::<i64>() {
                                Ok(num) => n = num,
                                Err(_) => {
                                    // signal parse failure by returning (false, 0)
                                    return Ok((false, 0i64));
                                }
                            },
                            Err(_) => {
                                return Ok((false, 0i64));
                            }
                        }
                    }

                    let new_n = n.wrapping_add(delta);

                    // clear old chunks
                    for (k, _) in &records {
                        trx.clear(k);
                    }

                    // write new single chunk at index 0
                    let subspace = Subspace::from_bytes(key.clone());
                    let chunk_key = subspace.pack(&(0,));
                    trx.set(&chunk_key, new_n.to_string().as_bytes());

                    Ok((true, new_n))
                }
            })
            .await;

        match res {
            Ok((true, v)) => Ok(v),
            Ok((false, _)) => Err("Value is not a valid integer".to_string()),
            Err(e) => Err(format!("FoundationDB error: {:?}", e)),
        }
    }
}
