use crate::operations::{SetFlags, SetMethod, SetTTL};
use fdb::FoundationDB;
use foundationdb_tuple::{TupleDepth, TuplePack, VersionstampOffset, pack};
use std::io::Write;

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
        let mut old_val = Some("Ok".pack_to_vec());

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
        fdb.set(&packed_key, &ttl.to_string().into_bytes())
            .await
            .map_err(|e| format!("FoundationDB set_ttl error: {:?}", e))
    }

    pub async fn get(fdb: &FoundationDB, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let packed_key = pack(&(SimpleDataPrefix::Data, key));
        fdb.get(&packed_key)
            .await
            .map_err(|e| format!("FoundationDB set error: {:?}", e))
    }

    pub async fn delete(fdb: &FoundationDB, key: &[u8]) -> Result<i64, String> {
        let packed_key = pack(&(SimpleDataPrefix::Data, key));
        fdb.delete(&packed_key)
            .await
            .map_err(|e| format!("FoundationDB set error: {:?}", e))
    }
}
