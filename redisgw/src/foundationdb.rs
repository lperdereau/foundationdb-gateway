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

    pub async fn set(
        &self,
        key: &[u8],
        value: &[u8],
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        self.database
            .run(|trx, _| async move {
                trx.set(key, value);
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn set_opt(
        &self,
        key: &[u8],
        value: &[u8],
        method: Option<SetMethod>,
        get: Option<bool>,
        ttl: Option<SetTTL>,
    ) -> Result<Option<Vec<u8>>, String> {
        let data_key = self.root_subspace.subspace(&DataPrefix::Table).pack(&key);
        let ttl_key = self
            .root_subspace
            .subspace(&DataPrefix::ExpiryTable)
            .pack(&key);

        let get = get.unwrap_or(false);
        let ts_vec;
        let ts: &[u8] = match ttl {
            Some(val) => {
                let ts_string = val.unix_epoch_in_ms()?.to_string();
                ts_vec = ts_string.as_bytes().to_vec();
                &ts_vec
            }
            None => &[],
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

        if should_set {
            let res = self.set(&data_key, value).await;
            if let Err(e) = res {
                return Err(format!("FoundationDB set_opt error (value): {:?}", e));
            }

            if ts.len() > 0 {
                let res_ttl = self.set(&ttl_key, ts).await;
                if let Err(e) = res_ttl {
                    return Err(format!("FoundationDB set_opt error (ttl): {:?}", e));
                }
            }
        }
        Ok(old_val)
    }

    pub async fn get(
        &self,
        key: &[u8],
    ) -> std::result::Result<Option<Vec<u8>>, foundationdb::FdbBindingError> {
        let data_subspace = self.root_subspace.subspace(&DataPrefix::Table);
        let data_key = data_subspace.pack(&key);
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
