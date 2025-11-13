use foundationdb::future::FdbValue;
use foundationdb::{FdbBindingError, RangeOption};
use futures::Stream;
use futures_util::TryStreamExt;
use futures_util::stream::StreamExt;
use std::result::Result;
use std::sync::Arc;
pub(crate) mod datamodel;
use datamodel::{DataModel, MAX_VALUE_SIZE};
pub mod lock;
pub use lock::LockManager;

/// High-level prefixes used to partition top-level keys in FoundationDB.
/// Use these values when packing top-level tuple keys to keep prefix constants in one place.
#[repr(u64)]
#[derive(Clone, Copy, Debug)]
pub enum Prefix {
    Data = 11,
    Lock = 12,
}

impl Prefix {
    pub fn as_u64(self) -> u64 {
        self as u64
    }
}

#[derive(Clone)]
pub struct FoundationDB {
    pub database: Arc<foundationdb::Database>,
    pub lock_manager: Arc<lock::LockManager>,
}

impl FoundationDB {
    pub fn new(db: Arc<foundationdb::Database>) -> Self {
        let lm = LockManager::new(db.clone(), MAX_VALUE_SIZE, 10_000u128, 30_000u64).into_arc();
        Self { database: db, lock_manager: lm }
    }

    pub async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), FdbBindingError> {
        let chunks = DataModel::split_into_chunks(value, None);

        // If the previous value was large, wait briefly for any in-flight modification to finish
        // before proceeding. This avoids reading/writing over a partially-updated set.
        let existing = DataModel::reconstruct_bloc(self, key).await?;
        if self.lock_manager.should_lock_for_size(existing.len()) {
            // best-effort wait; don't fail the set on timeout, but prefer to wait a short time
            let _ = self.lock_manager.wait_for_unlock(key, Some(5000)).await;
        }

        // Acquire lock if new value is large enough
        let mut token: Option<String> = None;
        if self.lock_manager.should_lock_for_size(value.len()) {
            match self.lock_manager.acquire(key, None).await {
                Ok(t) => token = Some(t),
                Err(e) => eprintln!("Lock acquire failed, proceeding without lock: {}", e),
            }
        }

        if !existing.is_empty() {
            self.delete(key).await?;
        }

        let res = DataModel::store_chunks_in_fdb(self, key, chunks).await;

        // best-effort release
        if let Some(t) = token {
            let _ = self.lock_manager.release(key, &t).await;
        }

        res?;
        Ok(())
    }

    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, FdbBindingError> {
        // If there's a concurrent write underway, wait a short time for it to finish
        // to avoid returning a partially updated set. If wait_for_unlock times out,
        // proceed anyway to avoid blocking reads indefinitely.
        let _ = self.lock_manager.wait_for_unlock(key, Some(2000)).await;

        let result = DataModel::reconstruct_bloc(self, key).await?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    pub async fn delete(&self, key: &[u8]) -> Result<i64, FdbBindingError> {
        // Acquire lock before deleting if likely large
        let mut token: Option<String> = None;
        // We can't easily know the value size here; assume delete may be large and try acquire with best-effort
        match self.lock_manager.acquire(key, None).await {
            Ok(t) => token = Some(t),
            Err(e) => eprintln!("Lock acquire failed for delete, proceeding: {}", e),
        }

        let res = DataModel::clean_chunks(self, key).await;

        if let Some(t) = token {
            let _ = self.lock_manager.release(key, &t).await;
        }

        res?;
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

    const MAX_SCAN_SIZE: usize = 20;

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
