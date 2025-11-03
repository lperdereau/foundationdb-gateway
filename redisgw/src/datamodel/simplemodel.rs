use crate::operations::{SetFlags, SetMethod, SetTTL};
use fdb::FoundationDB;
use foundationdb_tuple::{TupleDepth, TuplePack, VersionstampOffset, pack};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SimpleDataPrefix {
    Data = 11,
    Ttl = 12,
    Type = 13,
}

impl TuplePack for SimpleDataPrefix {
    fn pack<W: Write>(
        &self,
        w: &mut W,
        tuple_depth: TupleDepth,
    ) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}

pub struct SimpleDataModel {}

impl SimpleDataModel {
    pub async fn set(
        fdb: &FoundationDB,
        key: &[u8],
        value: &[u8],
        flags: SetFlags,
    ) -> Result<Option<Vec<u8>>, String> {
        let packed_key = pack(&(SimpleDataPrefix::Data, key));
        let mut old_val = None;

        let existing = if flags.get || flags.method.is_some() {
            SimpleDataModel::get(fdb, key)
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

        fdb.set(&packed_key, value)
            .await
            .map_err(|e| format!("FoundationDB set error: {:?}", e))?;

        if let Some(ttl_option) = &flags.ttl {
            let ttl = ttl_option
                .unix_epoch_in_ms()
                .map_err(|e| format!("FoundationDB set_ttl error: {:?}", e))?;
            if *ttl_option != SetTTL::KEPPTTL {
                SimpleDataModel::set_ttl(fdb, key, ttl)
                    .await
                    .map_err(|e| format!("FoundationDB set_ttl error: {:?}", e))?;
            }
        }

        Ok(old_val)
    }

    pub async fn set_ttl(fdb: &FoundationDB, key: &[u8], ttl: u128) -> Result<(), String> {
        let packed_key = pack(&(SimpleDataPrefix::Ttl, key));
        let ttl_bytes = ttl.to_be_bytes();
        fdb.set(&packed_key, &ttl_bytes)
            .await
            .map_err(|e| format!("FoundationDB set_ttl error: {:?}", e))
    }

    pub async fn get(fdb: &FoundationDB, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let packed_key = pack(&(SimpleDataPrefix::Data, key));
        // Read value and TTL (if any). If TTL exists and is expired, delete both and return None.
        let value = fdb
            .get(&packed_key)
            .await
            .map_err(|e| format!("FoundationDB get error: {:?}", e))?;

        // If there's no value, nothing to do.
        if value.is_none() {
            return Ok(None);
        }

        // Check TTL
        let packed_ttl_key = pack(&(SimpleDataPrefix::Ttl, key));
        let ttl_bytes_opt = fdb
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
                    Ok(d) => d.as_millis() as u128,
                    Err(_) => 0u128,
                };

                if ttl != 0 && ttl <= now {
                    // expired -> delete both keys
                    let _ = fdb
                        .delete(&packed_key)
                        .await
                        .map_err(|e| format!("FoundationDB delete expired error: {:?}", e))?;
                    let _ = fdb
                        .delete(&packed_ttl_key)
                        .await
                        .map_err(|e| format!("FoundationDB delete expired ttl error: {:?}", e))?;
                    return Ok(None);
                }
            }
        }

        Ok(value)
    }

    pub async fn delete(fdb: &FoundationDB, key: &[u8]) -> Result<i64, String> {
        let packed_key = pack(&(SimpleDataPrefix::Data, key));
        fdb.delete(&packed_key)
            .await
            .map_err(|e| format!("FoundationDB set error: {:?}", e))
    }
}
