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
    pub async fn set(fdb: &FoundationDB, key: &[u8], value: &[u8]) -> Result<(), String> {
        let packed_key = pack(&(SimpleDataPrefix::Data, key));
        fdb.set(&packed_key, value)
            .await
            .map_err(|e| format!("FoundationDB set error: {:?}", e))
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
