use crate::FoundationDB;
use foundationdb_tuple::Subspace;
use futures::StreamExt;

pub const MAX_VALUE_SIZE: usize = 100 * 1000; // 100KB
pub const MAX_TRANSACTION_SIZE: usize = 9 * 1000 * 1000; // 9MB
pub const MAX_RETRIES: usize = 3; // Maximum number of retries for a transaction

pub struct DataModel {}

impl DataModel {
    // Function to split data into chunks
    pub fn split_into_chunks(data: &[u8], chunk_size: Option<usize>) -> Vec<Vec<u8>> {
        let size = chunk_size.unwrap_or(MAX_VALUE_SIZE);
        data.chunks(size).map(|chunk| chunk.to_vec()).collect()
    }

    // Function to store chunks in FoundationDB using subspace and packing
    pub async fn store_chunks_in_fdb(
        fdb: &FoundationDB,
        key: &[u8],
        chunks: Vec<Vec<u8>>,
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let mut current_batch_size = 0;
        let mut batch = Vec::new();
        let mut batches = Vec::new();

        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk_size = chunk.len();
            if current_batch_size + chunk_size > MAX_TRANSACTION_SIZE {
                // Save the current batch
                batches.push(std::mem::take(&mut batch));
                current_batch_size = 0;
            }
            batch.push((i, chunk));
            current_batch_size += chunk_size;
        }

        // Add any remaining batch
        if !batch.is_empty() {
            batches.push(batch);
        }

        // Spawn all batch futures
        let futures = batches
            .into_iter()
            .map(|b| Self::store_batch_with_retry(fdb, key, b))
            .collect::<Vec<_>>();

        // Await all in parallel
        let results = futures::future::join_all(futures).await;

        // Check for errors
        for res in results {
            res?; // propagate the first error found
        }

        Ok(())
    }

    // Helper function to store a batch of chunks with retry logic
    async fn store_batch_with_retry(
        fdb: &FoundationDB,
        key: &[u8],
        batch: Vec<(usize, Vec<u8>)>,
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let mut retries = 0;
        // Clone the batch for each retry since Vec is moved into the async closure
        while retries < MAX_RETRIES {
            match Self::store_batch_in_fdb(fdb, key, batch.clone()).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    retries += 1;
                    if retries == MAX_RETRIES {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    // Function to store a batch of chunks
    async fn store_batch_in_fdb(
        fdb: &FoundationDB,
        key: &[u8],
        batch: Vec<(usize, Vec<u8>)>,
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let subspace = Subspace::from_bytes(key);
        fdb.database
            .run(move |trx, _| {
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
    pub async fn clean_chunks(
        fdb: &FoundationDB,
        key: &[u8],
    ) -> std::result::Result<(), foundationdb::FdbBindingError> {
        let mut end = key.to_vec();
        end.push(0xFF);

        let kvs = fdb.full_scan(key, &end).await.collect::<Vec<_>>().await;
        let mut keys_to_clear = Vec::new();
        for res in kvs {
            let (k, _) = res?;
            keys_to_clear.push(k);
        }

        fdb.database
            .run(move |trx, _| {
                let keys_to_clear = keys_to_clear.clone();
                async move {
                    for k in keys_to_clear {
                        trx.clear(&k);
                    }
                    Ok(())
                }
            })
            .await
    }

    // Function to retrieve and combine chunks from FoundationDB
    pub async fn retrieve_chunks_from_fdb(
        fdb: &FoundationDB,
        key: &[u8],
        num_chunks: usize,
    ) -> std::result::Result<Vec<u8>, foundationdb::FdbBindingError> {
        let subspace = Subspace::from_bytes(key);
        fdb.database
            .run(move |trx, _| {
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

    pub async fn reconstruct_bloc(
        fdb: &FoundationDB,
        key_prefix: &[u8],
    ) -> Result<Vec<u8>, foundationdb::FdbBindingError> {
        let mut end = key_prefix.to_vec();
        end.push(0xFF);

        let mut bloc = Vec::new();
        let result = fdb
            .full_scan(key_prefix, &end)
            .await
            .collect::<Vec<_>>()
            .await;
        for chunk in result {
            let (_key, value) = chunk?;
            bloc.extend_from_slice(&value);
        }
        Ok(bloc)
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
        let db = FoundationDB::new(_guard.clone());
        let subspace = Subspace::from_bytes("subspace");
        let key = subspace.pack(&("my_large_key"));
        let large_data = vec![0; 512 * 10 * MAX_VALUE_SIZE];
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
        let db = FoundationDB::new(_guard.clone());
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
    async fn test_reconstruct_bloc() {
        let _guard = get_db_once().await;
        let db = FoundationDB::new(_guard.clone());
        let subspace = Subspace::from_bytes("subspace_reconstruct");
        let key = subspace.pack(&("my_bloc_key"));

        // Create some data and chunk it
        let mut data = vec![0; 512 * 3 * MAX_VALUE_SIZE];
        let mut rng = rand::thread_rng();
        rng.fill(&mut data[..]);

        let chunks = DataModel::split_into_chunks(&data, None);

        // Store the chunks
        let res = DataModel::store_chunks_in_fdb(&db, &key, chunks).await;
        assert!(res.is_ok());

        // Reconstruct using the new method
        let reconstructed = DataModel::reconstruct_bloc(&db, &key)
            .await
            .expect("reconstruct_bloc failed");
        assert_eq!(reconstructed, data);
    }
}
