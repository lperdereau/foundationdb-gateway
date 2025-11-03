use crate::gateway::RedisGateway;
use crate::operations::{Flags, StringOperations};
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

#[tokio::test]
async fn test_set_overwrite() {
    let _guard = fdb_testcontainer::get_db_once().await;
    let db = fdb::FoundationDB::new(_guard.clone());
    let gw = crate::gateway::RedisGateway::new(db);

    let _ = gw.set(b"ow", b"v1", crate::operations::Flags::None).await;
    let _ = gw.set(b"ow", b"v2", crate::operations::Flags::None).await;
    let res = gw.get(b"ow").await;
    assert_eq!(res, Frame::SimpleString(b"\"v2\"".to_vec()));
    let _ = gw.del(b"ow").await;
}

#[tokio::test]
async fn test_set_nx_xx() {
    use crate::operations::{SetFlags, SetMethod};
    let _guard = fdb_testcontainer::get_db_once().await;
    let db = fdb::FoundationDB::new(_guard.clone());
    let gw = crate::gateway::RedisGateway::new(db);

    // NX: only set when not exists
    let flags = SetFlags { method: Some(SetMethod::NX), ttl: None, get: false };
    let _ = gw.set(b"nxkey", b"v1", crate::operations::Flags::Set(flags.clone())).await;
    // second NX should not overwrite
    let _ = gw.set(b"nxkey", b"v2", crate::operations::Flags::Set(flags)).await;
    let res = gw.get(b"nxkey").await;
    assert_eq!(res, Frame::SimpleString(b"\"v1\"".to_vec()));

    let _ = gw.del(b"nxkey").await;

    // XX: only set when exists
    let flags_xx = SetFlags { method: Some(SetMethod::XX), ttl: None, get: false };
    // should not set because key absent
    let _ = gw.set(b"xxkey", b"v1", crate::operations::Flags::Set(flags_xx.clone())).await;
    let res = gw.get(b"xxkey").await;
    assert_eq!(res, Frame::Null);

    // create then XX should succeed
    let _ = gw.set(b"xxkey", b"v0", crate::operations::Flags::None).await;
    let _ = gw.set(b"xxkey", b"v1", crate::operations::Flags::Set(flags_xx)).await;
    let res = gw.get(b"xxkey").await;
    assert_eq!(res, Frame::SimpleString(b"\"v1\"".to_vec()));
    let _ = gw.del(b"xxkey").await;
}

#[tokio::test]
async fn test_del_multiple() {
    let _guard = fdb_testcontainer::get_db_once().await;
    let db = fdb::FoundationDB::new(_guard.clone());
    let gw = crate::gateway::RedisGateway::new(db);

    let _ = gw.set(b"d1", b"a", crate::operations::Flags::None).await;
    let _ = gw.set(b"d2", b"b", crate::operations::Flags::None).await;
    let _ = gw.set(b"d3", b"c", crate::operations::Flags::None).await;

    // del returns integer frame with number of deleted keys
    let res = gw.del(b"d1").await;
    // del currently implemented to return Integer(1) on success
    assert!(matches!(res, Frame::Integer(_)));
    let _ = gw.del(b"d2").await;
    let _ = gw.del(b"d3").await;
}

#[tokio::test]
async fn test_concurrent_incr() {
    use futures::future::join_all;
    let _guard = fdb_testcontainer::get_db_once().await;
    let db = fdb::FoundationDB::new(_guard.clone());
    let gw = crate::gateway::RedisGateway::new(db.clone());
    let key = b"conc_counter";
    let _ = gw.set(key, b"0", crate::operations::Flags::None).await;

    let tasks: Vec<_> = (0..20)
        .map(|_| {
            let gw = crate::gateway::RedisGateway::new(db.clone());
            let key = key.to_vec();
            tokio::spawn(async move {
                for _ in 0..10 {
                    let _ = gw.incr(&key,).await;
                }
            })
        })
        .collect();

    let _ = join_all(tasks).await;

    let res = gw.get(key).await;
    // expect 200 increments
    assert_eq!(res, Frame::SimpleString(b"\"200\"".to_vec()));
    let _ = gw.del(key).await;
}
