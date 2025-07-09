use crate::gateway::RedisGateway;
use crate::operations::{Flags, RedisOperations};
use fdb::FoundationDB;
use fdb_testcontainer::get_db_once;
use redis_protocol::resp2::types::OwnedFrame as Frame;

#[tokio::test]
async fn test_insert_record() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);
    let _ = gw.set(b"key", b"value", Flags::None).await;
    let result = gw.get(b"key").await;
    assert_eq!(result, Frame::SimpleString(b"\"value\"".to_vec()));
    let _ = gw.del(b"key").await;
    let result = gw.get(b"key").await;
    assert_eq!(result, Frame::Null);

    let _ = gw.set(b"key", b"value", Flags::None).await;
    let result = gw.getdel(b"key").await;
    assert_eq!(result, Frame::SimpleString(b"\"value\"".to_vec()));
    let result = gw.getdel(b"key").await;
    assert_eq!(result, Frame::Null);
}

#[tokio::test]
async fn test_increment_decrement_record() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gateway = RedisGateway::new(db);
    let key = b"counter";
    gateway.set(key, b"0", Flags::None).await;
    let val = gateway.incr(key).await;
    assert_eq!(val, Frame::Integer(1));
    let val = gateway.incr(key).await;
    assert_eq!(val, Frame::Integer(2));
    let val = gateway.decr(key).await;
    assert_eq!(val, Frame::Integer(1));
    let val = gateway.decr(key).await;
    assert_eq!(val, Frame::Integer(0));
    let val = gateway.decr(key).await;
    assert_eq!(val, Frame::Integer(-1));

    gateway.set(key, b"10", Flags::None).await;
    let val = gateway.incr(key).await;
    assert_eq!(val, Frame::Integer(11));

    gateway.set(key, b"100", Flags::None).await;
    let val = gateway.incr(key).await;
    assert_eq!(val, Frame::Integer(101));
    let _ = gateway.decr(key).await;
    let val = gateway.decr(key).await;
    assert_eq!(val, Frame::Integer(99));

    gateway.del(key).await;
}

#[tokio::test]
async fn test_increment_decrement_by_record() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gateway = RedisGateway::new(db);
    let key = b"counter";
    gateway.set(key, b"0", Flags::None).await;
    let val = gateway.incr_by(key, b"10").await;
    assert_eq!(val, Frame::Integer(10));
    let val = gateway.incr_by(key, b"100").await;
    assert_eq!(val, Frame::Integer(110));
    let val = gateway.decr_by(key, b"109").await;
    assert_eq!(val, Frame::Integer(1));
    let val = gateway.decr_by(key, b"10").await;
    assert_eq!(val, Frame::Integer(-9));
    let val = gateway.incr_by(key, b"8").await;
    assert_eq!(val, Frame::Integer(-1));
}
