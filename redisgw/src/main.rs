use fdb::FoundationDB;
use foundationdb::Database;
use redisgw::gateway::RedisGateway;
use redisgw::server::Server;
use std::sync::Arc;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let _network = unsafe { foundationdb::boot() };
    let db = Database::new(None).expect("Failed to run Database");
    let fdb = FoundationDB::new(Arc::new(db));
    let gw = RedisGateway::new(fdb);
    let server = Server::new("127.0.0.1:6379", gw);
    let res = server.start().await;
    return res;
}
