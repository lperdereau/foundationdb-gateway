use fdb::FoundationDB;
use std::sync::Arc;

use crate::config::{ServerConfig, SharedServerConfig, SharedSocketConfig};

#[derive(Clone)]
pub struct RedisGateway {
    pub(crate) fdb: FoundationDB,
    /// Shared server-wide configuration
    pub server_cfg: SharedServerConfig,
    /// Optional per-connection socket configuration
    pub socket_cfg: Option<SharedSocketConfig>,
}

impl RedisGateway {
    pub fn new(fdb: FoundationDB) -> Self {
        Self {
            fdb,
            server_cfg: Arc::new(ServerConfig::default()),
            socket_cfg: None,
        }
    }

    /// Create a clone of this gateway that is bound to a per-connection socket config.
    pub fn with_socket_config(&self, sc: SharedSocketConfig) -> Self {
        let mut cloned = self.clone();
        cloned.socket_cfg = Some(sc);
        cloned
    }
}
