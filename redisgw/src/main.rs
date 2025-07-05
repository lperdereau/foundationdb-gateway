use foundationdb::Database;
use foundationdb_tuple::Subspace;
use redisgw::foundationdb::FoundationDB;
use redisgw::gateway::RedisGateway;
use redisgw::server::Server;
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let _network = unsafe { foundationdb::boot() };
    let db = Database::new(None).expect("Failed to run Database");
    let fdb = FoundationDB::new(Subspace::all(), Arc::new(db));
    let gw = RedisGateway::new(fdb);
    let server = Server::new("127.0.0.1:6379", gw);
    server.start().await
}
