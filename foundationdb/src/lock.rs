use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct LockManager {
    pub db: Arc<foundationdb::Database>,
    pub lock_ttl_ms: u128,
    pub default_timeout_ms: u64,
    pub lock_size_threshold: usize,
}

impl LockManager {
    pub fn new(
        db: Arc<foundationdb::Database>,
        lock_size_threshold: usize,
        lock_ttl_ms: u128,
        default_timeout_ms: u64,
    ) -> Self {
        Self {
            db,
            lock_ttl_ms,
            default_timeout_ms,
            lock_size_threshold,
        }
    }

    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn should_lock_for_size(&self, size: usize) -> bool {
        size >= self.lock_size_threshold
    }

    /// Acquire a lock for `key`. Returns a token string to be used for release.
    pub async fn acquire(&self, key: &[u8], timeout_ms: Option<u64>) -> Result<String, String> {
        use tokio::time::{sleep, Instant};

        let timeout_ms = timeout_ms.unwrap_or(self.default_timeout_ms);
        let start = Instant::now();
        let mut backoff = 10u64;
    let lock_key = foundationdb_tuple::pack(&(crate::Prefix::Lock.as_u64(), key));

        while start.elapsed().as_millis() as u64 <= timeout_ms {
            let lk = lock_key.clone();
            let db = self.db.clone();
            // generate an uuid v7 token (time-ordered, contains ms timestamp)
            let token_uuid = Uuid::now_v7();
            let token = token_uuid.to_string();
            // extract timestamp (ms) from uuidv7: top 48 bits contain unix millis
            let u128v = token_uuid.as_u128();
            let now_ms = (u128v >> 80) & ((1u128 << 48) - 1u128);
            let token_value = token.clone();

            let res = db
                .run(move |trx, _| {
                    let lk = lk.clone();
                    let token_value = token_value.clone();
                    async move {
                        let existing = trx.get(&lk, false).await?;
                        if let Some(val) = existing {
                            if let Ok(s) = std::str::from_utf8(&val) {
                                if let Ok(existing_uuid) = Uuid::parse_str(s) {
                                    let ev = existing_uuid.as_u128();
                                    let owner_ts = (ev >> 80) & ((1u128 << 48) - 1u128);
                                    if now_ms.saturating_sub(owner_ts) > self.lock_ttl_ms {
                                        trx.set(&lk, token_value.as_bytes());
                                        return Ok(true);
                                    }
                                }
                            }
                            return Ok(false);
                        }
                        trx.set(&lk, token_value.as_bytes());
                        Ok(true)
                    }
                })
                .await;

            match res {
                Ok(true) => return Ok(token),
                Ok(false) => {
                    sleep(std::time::Duration::from_millis(backoff)).await;
                    backoff = (backoff * 2).min(500);
                    continue;
                }
                Err(e) => {
                    eprintln!("LockManager transient error while acquiring lock: {:?}", e);
                    sleep(std::time::Duration::from_millis(backoff)).await;
                    backoff = (backoff * 2).min(500);
                    continue;
                }
            }
        }

        Err("timeout acquiring key lock".to_string())
    }

    /// Release lock only if token matches.
    pub async fn release(&self, key: &[u8], token: &str) -> Result<(), String> {
    let lock_key = foundationdb_tuple::pack(&(crate::Prefix::Lock.as_u64(), key));
        let lk = lock_key.clone();
        let db = self.db.clone();
        let token = token.to_string();
        let res = db
            .run(move |trx, _| {
                let lk = lk.clone();
                let token = token.clone();
                async move {
                    let existing = trx.get(&lk, false).await?;
                    if let Some(val) = existing {
                        if let Ok(s) = std::str::from_utf8(&val) {
                            if s.starts_with(&token) {
                                trx.clear(&lk);
                            }
                        }
                    }
                    Ok(())
                }
            })
            .await;

        res.map_err(|e| format!("FoundationDB release_lock error: {:?}", e))
    }

    /// Return true if the lock key currently exists.
    pub async fn is_locked(&self, key: &[u8]) -> Result<bool, String> {
    let lock_key = foundationdb_tuple::pack(&(crate::Prefix::Lock.as_u64(), key));
        let lk = lock_key.clone();
        let db = self.db.clone();
        let res = db
            .run(move |trx, _| {
                let lk = lk.clone();
                async move {
                    let existing = trx.get(&lk, false).await?;
                    Ok(existing.is_some())
                }
            })
            .await;

        res.map_err(|e| format!("FoundationDB is_locked error: {:?}", e))
    }

    /// Wait until the lock is released or timeout is reached.
    /// Returns Ok(()) if unlocked, Err on timeout or underlying error.
    pub async fn wait_for_unlock(&self, key: &[u8], timeout_ms: Option<u64>) -> Result<(), String> {
        use tokio::time::{sleep, Instant};

        let timeout_ms = timeout_ms.unwrap_or(self.default_timeout_ms);
        let start = Instant::now();
        let mut backoff = 5u64;

        while start.elapsed().as_millis() as u64 <= timeout_ms {
            match self.is_locked(key).await {
                Ok(true) => {
                    sleep(std::time::Duration::from_millis(backoff)).await;
                    backoff = (backoff * 2).min(200);
                    continue;
                }
                Ok(false) => return Ok(()),
                Err(e) => {
                    // on error, retry a few times until timeout
                    eprintln!("wait_for_unlock: transient error: {:?}", e);
                    sleep(std::time::Duration::from_millis(backoff)).await;
                    backoff = (backoff * 2).min(200);
                    continue;
                }
            }
        }

        Err("timeout waiting for unlock".to_string())
    }
}
