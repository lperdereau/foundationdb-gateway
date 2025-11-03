use fdb_testcontainer::get_db_once;
use fdb::FoundationDB;
use crate::gateway::RedisGateway;
use crate::server::Server;
use tokio::time::{sleep, Duration};

/// Start a test FDB instance and spawn a RedisGateway Server bound to an ephemeral port.
/// Returns (server_handle, stream). The testcontainer guard is captured by the
/// spawned server task to keep the DB alive while the server runs.
pub async fn spawn_test_server() -> (tokio::task::JoinHandle<()>, Option<tokio::net::TcpStream>) {
    let guard = get_db_once().await;
    let db = FoundationDB::new(guard.clone());
    let gw = RedisGateway::new(db);

    // Reserve an ephemeral port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);
    let bind_addr = format!("127.0.0.1:{}", addr.port());

    // Start server in background; move `guard` into the task so it remains alive.
    let server = Server::new(bind_addr.clone(), gw);
    let srv = tokio::spawn(async move {
        let _guard = guard; // keep guard alive while server runs
        let _ = server.start().await;
    });

    // Wait a little for server to start accepting connections
    let mut connected = false;
    let mut stream = None;
    for _ in 0..10 {
        match tokio::net::TcpStream::connect(&bind_addr).await {
            Ok(s) => { stream = Some(s); connected = true; break; }
            Err(_) => sleep(Duration::from_millis(50)).await,
        }
    }
    assert!(connected, "could not connect to server");

    (srv, stream)
}

// Macro is defined in the parent `tests` module so it is available to child test modules.
