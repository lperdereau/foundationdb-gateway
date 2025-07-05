use crate::operations::{SetMethod, SetTTL};
use foundationdb::{Database, TransactOption};
use foundationdb_tuple::{Subspace, TupleDepth, TuplePack, VersionstampOffset};
use std::io::Write;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DataPrefix {
    Table = 1,
    ExpiryTable = 2,
}

impl TuplePack for DataPrefix {
    fn pack<W: Write>(
        &self,
        w: &mut W,
        tuple_depth: TupleDepth,
    ) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}

#[derive(Clone)]
pub struct FoundationDB {
    root_subspace: Subspace,
    pub database: Arc<Database>,
}

impl FoundationDB {
    pub fn new(root_subspace: Subspace, db: Arc<Database>) -> Self {
        Self {
            root_subspace: root_subspace,
            database: db,
        }
    }

    pub async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.set_opt(key, value, None, None, None).await.map(|_| ())
    }

    pub async fn set_opt(
        &self,
        key: &[u8],
        value: &[u8],
        method: Option<SetMethod>,
        get: Option<bool>,
        ttl: Option<SetTTL>,
    ) -> Result<Option<Vec<u8>>, String> {
        let data_subspace = self.root_subspace.subspace(&DataPrefix::Table);
        let ttl_subspace = self.root_subspace.subspace(&DataPrefix::ExpiryTable);
        let get = get.unwrap_or(false);
        let ts = match ttl {
            Some(val) => val.unix_epoch_in_ms()?,
            None => 0,
        };

        let old_val = if get {
            match self.get(key).await {
                Ok(val) => val,
                Err(e) => return Err(format!("FoundationDB get error: {:?}", e)),
            }
        } else {
            None
        };

        let should_set = match method {
            Some(SetMethod::NX) => !old_val.is_some(),
            Some(SetMethod::XX) => old_val.is_some(),
            None => true,
        };

        let data_key = data_subspace.pack(&key);
        let data_key_slice: &[u8] = &data_key;
        let ttl_key = ttl_subspace.pack(&key);
        let ttl_key_slice: &[u8] = &ttl_key;

        let res = self
            .database
            .run(|trx, _| async move {
                if should_set {
                    trx.set(&data_key_slice, value);
                }
                if ts > 0 {
                    let ts_bytes = ts.to_be_bytes();
                    trx.set(&ttl_key_slice, &ts_bytes);
                }
                Ok(())
            })
            .await;
        if res.is_err() {
            Err(format!("FoundationDB set_opt error: {:?}", res))
        } else {
            Ok(old_val)
        }
    }

    pub async fn get(
        &self,
        key: &[u8],
    ) -> std::result::Result<Option<Vec<u8>>, foundationdb::FdbBindingError> {
        let data_subspace = self.root_subspace.subspace(&DataPrefix::Table);
        let data_key = data_subspace.pack(&key.to_vec());
        let value = self
            .database
            .run(|trx, _| {
                let data_key = data_key.clone();
                async move { Ok(trx.get(&data_key, true).await?) }
            })
            .await?;
        let value = value.map(|v| v.to_vec());
        Ok(value)
    }

    pub async fn delete(
        &self,
        key: &[u8],
    ) -> std::result::Result<i64, foundationdb::FdbBindingError> {
        let data_subspace = self.root_subspace.subspace(&DataPrefix::Table);
        let data_key = data_subspace.pack(&key.to_vec());
        self.database
            .run(|trx, _| {
                let data_key = data_key.clone();
                async move {
                    trx.clear(&data_key);
                    Ok(1i64)
                }
            })
            .await?;
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fdb_testcontainer::get_db_once;
    use foundationdb_tuple::Subspace;

    #[tokio::test]
    async fn test_fdb_insert_record() {
        let _guard = get_db_once().await;
        let root_subspace = Subspace::from_bytes(vec![]);
        let db = FoundationDB::new(root_subspace, _guard.clone());
        db.set(b"key", b"value").await.expect("Unable to set key");
        let result = db.get(b"key").await.expect("Unable to get key");
        assert_eq!(result, Some(b"value".to_vec()));
        db.delete(b"key").await.expect("Unable to delete key");
        let result = db
            .get(b"key")
            .await
            .expect("Unable to get key after delete");
        assert!(result.is_none());
    }
}
