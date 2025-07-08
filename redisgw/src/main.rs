use clap::Parser;
use fdb::FoundationDB;
use foundationdb::Database;
use redisgw::gateway::RedisGateway;
use redisgw::server::Server;
use std::sync::Arc;

const APP_TITLE: &str = "redisgw";

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version, name = APP_TITLE)]
pub struct Config {
    /// IP address on which to start the server
    #[clap(long, env, default_value = "127.0.0.1")]
    ip: String,

    /// Port on which to start the server
    #[clap(long, env, default_value = "6379")]
    port: String,

    /// Port on which to start the server
    #[clap(long = "fdb-path", env)]
    fdb_path: Option<String>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = Config::parse();

    let _network = unsafe { foundationdb::boot() };
    let db = Database::new(config.fdb_path.as_deref()).expect("Failed to run Database");
    let fdb = FoundationDB::new(Arc::new(db));
    let gw = RedisGateway::new(fdb);
    let server = Server::new(format!("{}:{}", config.ip, config.port), gw);
    let res = server.start().await;
    return res;
}
