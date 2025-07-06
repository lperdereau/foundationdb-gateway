use std::sync::Arc;

pub mod datamodel;

#[derive(Clone)]
pub struct FoundationDB {
    pub database: Arc<foundationdb::Database>,
}

impl FoundationDB {
    pub fn new(db: Arc<foundationdb::Database>) -> Self {
        Self { database: db }
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

    pub async fn get(
        &self,
        key: &[u8],
    ) -> std::result::Result<Option<Vec<u8>>, foundationdb::FdbBindingError> {
        let value = self
            .database
            .run(|trx, _| async move { Ok(trx.get(key, true).await?) })
            .await?;
        let value = value.map(|v| v.to_vec());
        Ok(value)
    }

    pub async fn delete(
        &self,
        key: &[u8],
    ) -> std::result::Result<i64, foundationdb::FdbBindingError> {
        self.database
            .run(|trx, _| async move {
                trx.clear(key);
                Ok(1i64)
            })
            .await?;
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fdb_testcontainer::get_db_once;

    #[tokio::test]
    async fn test_insert_record() {
        let _guard = get_db_once().await;
        let db = FoundationDB::new(_guard.clone());
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
