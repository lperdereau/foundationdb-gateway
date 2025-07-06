use foundationdb::future::FdbValue;
use foundationdb::{FdbBindingError, RangeOption};
use futures::Stream;
use futures_util::TryStreamExt;
use futures_util::stream::StreamExt;
use std::result::Result;
use std::sync::Arc;

pub(crate) mod datamodel;
use datamodel::DataModel;

const MAX_SCAN_SIZE: usize = 20;

#[derive(Clone)]
pub struct FoundationDB {
    pub database: Arc<foundationdb::Database>,
}

impl FoundationDB {
    pub fn new(db: Arc<foundationdb::Database>) -> Self {
        Self { database: db }
    }

    pub async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), FdbBindingError> {
        let chunks = DataModel::split_into_chunks(&value, None);
        if let Some(_) = self.get(key).await? {
            self.delete(key).await?;
        }
        DataModel::store_chunks_in_fdb(&self, key, chunks).await?;
        Ok(())
    }

    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, FdbBindingError> {
        let result = DataModel::reconstruct_bloc(self, key).await?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    pub async fn delete(&self, key: &[u8]) -> Result<i64, FdbBindingError> {
        DataModel::clean_chunks(self, key).await?;
        Ok(1)
    }

    pub async fn scan(
        &self,
        start: &[u8],
        end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, FdbBindingError> {
        let kvs = self
            .database
            .run(|trx, _| async move {
                let range = RangeOption::from((start, end));
                let stream = trx.get_ranges_keyvalues(range, false);
                collect_stream(stream).await
            })
            .await?;

        Ok(kvs)
    }

    pub async fn full_scan(
        &self,
        start: &[u8],
        end: &[u8],
    ) -> impl Stream<Item = Result<(Vec<u8>, Vec<u8>), FdbBindingError>> {
        let mut start = start.to_vec();
        async_stream::try_stream! {
            loop {
                let kvs = self.scan(&start, end).await?;
                if kvs.is_empty() {
                    break;
                }
                if let Some(kv) = kvs.last() {
                    start = kv.0.to_vec();
                    start.push(0xff);
                }
                for kv in kvs {
                    yield kv;
                }
            }
        }
    }
}

async fn collect_stream<S>(stream: S) -> Result<Vec<(Vec<u8>, Vec<u8>)>, FdbBindingError>
where
    S: futures_util::Stream<Item = foundationdb::FdbResult<FdbValue>>,
{
    let records = stream
        .map(|x| match x {
            Ok(value) => {
                let data = (value.key().to_vec(), value.value().to_vec());
                Ok(data)
            }
            Err(err) => Err(err),
        })
        .take(20)
        .try_collect::<Vec<(Vec<u8>, Vec<u8>)>>()
        .await?;
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fdb_testcontainer::get_db_once;
    use foundationdb_tuple::pack;

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

    #[tokio::test]
    async fn test_scan() {
        let _guard = get_db_once().await;
        let db = FoundationDB::new(_guard.clone());

        db.set(b"key1", b"value1")
            .await
            .expect("Unable to set key1");
        db.set(b"key2", b"value2")
            .await
            .expect("Unable to set key2");
        db.set(b"key3", b"value3")
            .await
            .expect("Unable to set key3");
        let result = db.scan(b"key1", b"key3").await.expect("Unable to scan");
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_bigger_than_max_scan() {
        let _guard = get_db_once().await;
        let db = FoundationDB::new(_guard.clone());

        for i in 0..MAX_SCAN_SIZE + 1 {
            let key = pack(&("key", &i));
            db.set(&key, format!("value{}", i).as_bytes())
                .await
                .expect("Unable to set key");
        }

        let start = pack(&("key", &0));
        let end = pack(&("key", &100));

        let result = db.scan(&start, &end).await.expect("Unable to scan");
        assert_eq!(result.len(), MAX_SCAN_SIZE);
    }

    #[tokio::test]
    async fn test_full_scan() {
        let _guard = get_db_once().await;
        let db = FoundationDB::new(_guard.clone());

        for i in 0..100 {
            let key = pack(&("key", &i));
            db.set(&key, format!("value{}", i).as_bytes())
                .await
                .expect("Unable to set key");
        }

        let start = pack(&("key", &0));
        let end = pack(&("key", &100));

        let result = db.full_scan(&start, &end).await.collect::<Vec<_>>().await;
        assert_eq!(result.len(), 100);
    }
}
