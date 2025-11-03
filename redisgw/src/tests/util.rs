use fdb_testcontainer::get_db_once;
use fdb::FoundationDB;
use crate::gateway::RedisGateway;
use crate::server::Server;
/// Start a test FDB instance and spawn a RedisGateway Server bound to an ephemeral port.
/// Returns (bind_addr, server_handle). The testcontainer guard is captured by the
/// spawned server task to keep the DB alive while the server runs.
pub async fn spawn_test_server() -> (String, tokio::task::JoinHandle<()>) {
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

    (bind_addr, srv)
}

// Macro is defined in the parent `tests` module so it is available to child test modules.
