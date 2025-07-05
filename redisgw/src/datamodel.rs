use foundationdb::Database;
use foundationdb_tuple::Subspace;
use std::collections::HashSet;

pub const MAX_VALUE_SIZE: usize = 100 * 1000; // 100KB
pub const MAX_TRANSACTION_SIZE: usize = 9 * 1000 * 1000; // 9MB
pub const MAX_RETRIES: usize = 3; // Maximum number of retries for a transaction

struct DataModel {}

impl DataModel {
    // Function to split data into chunks
    pub fn split_into_chunks(data: &[u8], chunk_size: Option<usize>) -> Vec<Vec<u8>> {
        let size = chunk_size.unwrap_or(MAX_VALUE_SIZE);
        data.chunks(size).map(|chunk| chunk.to_vec()).collect()
    }

    // Function to store chunks in FoundationDB using subspace and packing
    pub async fn store_chunks_in_fdb(
        db: &Database,
        key: &Vec<u8>,
        chunks: Vec<Vec<u8>>,
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let mut current_batch_size = 0;
        let mut batch = Vec::new();
        let mut successfully_stored = HashSet::new();

        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk_size = chunk.len();
            if current_batch_size + chunk_size > MAX_TRANSACTION_SIZE {
                // Process the current batch
                Self::store_batch_with_retry(db, key, &batch, &mut successfully_stored).await?;
                batch.clear();
                current_batch_size = 0;
            }
            batch.push((i, chunk));
            current_batch_size += chunk_size;
        }

        // Process any remaining chunks in the batch
        if !batch.is_empty() {
            Self::store_batch_with_retry(db, key, &batch, &mut successfully_stored).await?;
        }

        Ok(())
    }

    // Helper function to store a batch of chunks with retry logic
    async fn store_batch_with_retry(
        db: &Database,
        key: &Vec<u8>,
        batch: &[(usize, Vec<u8>)],
        successfully_stored: &mut HashSet<usize>,
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let mut retries = 0;
        while retries < MAX_RETRIES {
            match Self::store_batch_in_fdb(db, key, batch).await {
                Ok(_) => {
                    // Mark chunks as successfully stored
                    for &(i, _) in batch {
                        successfully_stored.insert(i);
                    }
                    return Ok(());
                }
                Err(e) => {
                    retries += 1;
                    if retries == MAX_RETRIES {
                        // If all retries fail, clean up any successfully stored chunks
                        Self::cleanup_failed_chunks(db, key, successfully_stored).await?;
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    // Function to store a batch of chunks
    async fn store_batch_in_fdb(
        db: &Database,
        key: &Vec<u8>,
        batch: &[(usize, Vec<u8>)],
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let subspace = Subspace::from_bytes(key.clone());
        db.run(move |trx, _| {
            let batch = batch.to_vec();
            let subspace = subspace.clone();
            async move {
                for (i, chunk) in batch {
                    let chunk_key = subspace.pack(&(i));
                    trx.set(&chunk_key, &chunk);
                }
                Ok(())
            }
        })
        .await
    }

    // Function to clean up failed chunks
    async fn cleanup_failed_chunks(
        db: &Database,
        key: &Vec<u8>,
        successfully_stored: &HashSet<usize>,
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let subspace = Subspace::from_bytes(key.clone());
        db.run(move |trx, _| {
            let successfully_stored = successfully_stored.clone();
            let subspace = subspace.clone();
            async move {
                for i in successfully_stored {
                    let chunk_key = subspace.pack(&(i));
                    trx.clear(&chunk_key);
                }
                Ok(())
            }
        })
        .await
    }

    // Function to retrieve and combine chunks from FoundationDB
    pub async fn retrieve_chunks_from_fdb(
        db: &Database,
        key: &Vec<u8>,
        num_chunks: usize,
    ) -> std::result::Result<Vec<u8>, foundationdb::FdbBindingError> {
        let subspace = Subspace::from_bytes(key.clone());
        db.run(move |trx, _| {
            let mut futs = Vec::with_capacity(num_chunks);
            let subspace = subspace.clone();
            for i in 0..num_chunks {
                let chunk_key = subspace.pack(&(i,));
                futs.push(trx.get(&chunk_key, false));
            }
            async move {
                let mut result = Vec::new();
                for fut in futs {
                    if let Some(chunk) = fut.await? {
                        result.extend_from_slice(&chunk);
                    }
                }
                Ok(result)
            }
        })
        .await
    }

    pub async fn count_chunks_in_fdb(
        db: &Database,
        key: &Vec<u8>,
    ) -> std::result::Result<usize, foundationdb::FdbBindingError> {
        let subspace = Subspace::from_bytes(key.clone());
        db.run(move |trx, _| {
            let range = subspace.range();
            async move {
                let kvs = trx
                    .get_range(&foundationdb::RangeOption::from(range), 0, true)
                    .await?;
                Ok(kvs.len())
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    use fdb_testcontainer::get_db_once;

    #[tokio::test]
    async fn test_long_chunk() {
        let _guard = get_db_once().await;
        let db = _guard.clone();
        let subspace = Subspace::from_bytes("subspace");
        let large_data = vec![0; 512 * 10 * MAX_VALUE_SIZE];
        let key = subspace.pack(&("my_large_key"));
        let chunks = DataModel::split_into_chunks(&large_data, None);
        let num_chunks = chunks.len();

        let res = DataModel::store_chunks_in_fdb(&db, &key, chunks).await;
        assert!(res.is_ok());

        let res = DataModel::retrieve_chunks_from_fdb(&db, &key, num_chunks).await;
        assert!(res.is_ok());
        let retrieved = res.unwrap();
        assert_eq!(retrieved, large_data);
    }

    #[tokio::test]
    async fn test_ordering() {
        let _guard = get_db_once().await;
        let db = _guard.clone();
        let subspace = Subspace::from_bytes("subspace");

        let mut large_data = vec![0; 100_000_000];
        let mut rng = rand::thread_rng();
        rng.fill(&mut large_data[..]);

        let key = subspace.pack(&("my_rand_key"));

        let chunks = DataModel::split_into_chunks(&large_data, None);
        let num_chunks = chunks.len();

        let res = DataModel::store_chunks_in_fdb(&db, &key, chunks).await;
        assert!(res.is_ok());

        let res = DataModel::retrieve_chunks_from_fdb(&db, &key, num_chunks).await;
        assert!(res.is_ok());
        let retrieved = res.unwrap();
        assert_eq!(retrieved, large_data);
    }

    #[tokio::test]
    async fn test_count_chunks_in_fdb() {
        let _guard = get_db_once().await;
        let db = _guard.clone();
        let subspace = Subspace::from_bytes("subspace_count_chunks");

        // Create data that will be split into multiple chunks
        let data = vec![42u8; 3 * MAX_VALUE_SIZE + 123];
        let key = subspace.pack(&("my_count_chunks_key"));
        let chunks = DataModel::split_into_chunks(&data, None);
        let num_chunks = chunks.len();

        // Store the chunks
        let res = DataModel::store_chunks_in_fdb(&db, &key, chunks.clone()).await;
        assert!(res.is_ok());

        // Count the chunks using the method
        let count_res = DataModel::count_chunks_in_fdb(&db, &key).await;
        assert!(count_res.is_ok());
        let count = count_res.unwrap();
        assert_eq!(count, num_chunks);
    }
}
