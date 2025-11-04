use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address the server binds to (informational here)
    pub bind: String,
    /// Default keyspace / DB index
    pub default_db: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { bind: "127.0.0.1:6379".to_string(), default_db: 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SocketConfig {
    /// If true, the server loop will close the connection after sending reply
    pub should_close: bool,
    /// Selected DB index for this connection
    pub selected_db: usize,
    /// Authenticated user (if any)
    pub authenticated_user: Option<String>,
    /// Client name if set by CLIENT SETNAME
    pub client_name: Option<String>,
}

impl SocketConfig {
    pub fn mark_close(&mut self) {
        self.should_close = true;
    }
}

pub type SharedServerConfig = Arc<ServerConfig>;
pub type SharedSocketConfig = Arc<std::sync::RwLock<SocketConfig>>;
