use crate::gateway::RedisGateway;
use crate::operations::{Flags, SetFlags, SetTTL, StringOperations};
use std::time::Duration;
use fdb::FoundationDB;
use fdb_testcontainer::get_db_once;
use redis_protocol::resp2::types::OwnedFrame as Frame;

#[tokio::test]
async fn test_set_with_ttl() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);

    let flags = SetFlags {
        method: None,
        ttl: Some(SetTTL::EX(1)),
        get: false,
    };

    let _ = gw.set(b"ttlkey", b"value", Flags::Set(flags)).await;
    let result = gw.get(b"ttlkey").await;
    assert_eq!(result, Frame::SimpleString(b"\"value\"".to_vec()));

    tokio::time::sleep(Duration::from_millis(1200)).await;

    let result = gw.get(b"ttlkey").await;
    assert_eq!(result, Frame::Null);
}
