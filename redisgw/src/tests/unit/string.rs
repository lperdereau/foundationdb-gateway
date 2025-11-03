use crate::gateway::RedisGateway;
use crate::operations::{Flags, SetFlags, StringOperations, SetTTL};
use fdb::FoundationDB;
use fdb_testcontainer::get_db_once;
use redis_protocol::resp2::types::OwnedFrame as Frame;
use tokio::time::{sleep, Duration};
use futures::future::join_all;

#[tokio::test]
async fn test_insert_record() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);

    let _ = gw.set(b"key", b"value", Flags::None).await;
    let result = gw.get(b"key").await;
    assert_eq!(result, Frame::BulkString(b"value".to_vec()));
    let _ = gw.del(b"key").await;
    let result = gw.get(b"key").await;
    assert_eq!(result, Frame::Null);

    let _ = gw.set(b"key", b"value", Flags::None).await;
    let result = gw.getdel(b"key").await;
    assert_eq!(result, Frame::BulkString(b"value".to_vec()));
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
    assert_eq!(res, Frame::BulkString(b"v2".to_vec()));
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
    assert_eq!(res, Frame::BulkString(b"v1".to_vec()));

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
    assert_eq!(res, Frame::BulkString(b"v1".to_vec()));
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
    assert_eq!(res, Frame::BulkString(b"200".to_vec()));
    let _ = gw.del(key).await;
}

#[tokio::test]
async fn test_set_with_ttl_ex_px() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);

    let flags = SetFlags { method: None, ttl: Some(SetTTL::PX(100)), get: false };
    let _ = gw.set(b"ttl_key", b"vttl", Flags::Set(flags)).await;
    // immediately available
    let res = gw.get(b"ttl_key").await;
    assert_eq!(res, Frame::BulkString(b"vttl".to_vec()));

    // wait for expiry (>100ms)
    sleep(Duration::from_millis(200)).await;
    let res = gw.get(b"ttl_key").await;
    assert_eq!(res, Frame::Null);
}

#[tokio::test]
async fn test_keep_ttl_preserved_on_set() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);

    // set initial key with 300ms TTL
    let flags_init = SetFlags { method: None, ttl: Some(SetTTL::PX(300)), get: false };
    let _ = gw.set(b"keep_ttl", b"v1", Flags::Set(flags_init)).await;

    // replace value but keep TTL
    let flags_keep = SetFlags { method: None, ttl: Some(SetTTL::KEPPTTL), get: false };
    let _ = gw.set(b"keep_ttl", b"v2", Flags::Set(flags_keep)).await;

    // after short wait (<300ms) key still exists
    sleep(Duration::from_millis(150)).await;
    let res = gw.get(b"keep_ttl").await;
    assert_eq!(res, Frame::BulkString(b"v2".to_vec()));

    // after TTL passes, key should be gone
    sleep(Duration::from_millis(200)).await;
    let res = gw.get(b"keep_ttl").await;
    assert_eq!(res, Frame::Null);
}

#[tokio::test]
async fn test_append_empty_and_nonempty() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);

    // append to missing key creates it
    let r = gw.append(b"app_key", b"hello").await;
    // append returns Integer with length; accept Integer
    assert!(matches!(r, Frame::Integer(_)));
    let res = gw.get(b"app_key").await;
    assert_eq!(res, Frame::BulkString(b"hello".to_vec()));

    // append more
    let r = gw.append(b"app_key", b", you").await;
    assert!(matches!(r, Frame::Integer(_)));
    let res = gw.get(b"app_key").await;
    assert_eq!(res, Frame::BulkString(b"hello, you".to_vec()));
}

#[tokio::test]
async fn test_set_get_with_getflag_returns_old_value() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);

    // initial set
    let _ = gw.set(b"getflag", b"old", Flags::None).await;

    // set with GET should return previous value
    let flags = SetFlags { method: None, ttl: None, get: true };
    let res = gw.set(b"getflag", b"new", Flags::Set(flags)).await;
    assert_eq!(res, Frame::BulkString(b"old".to_vec()));

    // confirm new value stored
    let res = gw.get(b"getflag").await;
    assert_eq!(res, Frame::BulkString(b"new".to_vec()));
}

// Concurrent large writes to the same key should not produce corrupted values.
// The SimpleDataModel uses a per-key lock to serialize large writes; this test
// ensures that after many concurrent writers, the stored value is exactly one
// of the writers' payloads.
#[tokio::test]
async fn test_concurrent_large_writes() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db.clone());

    let key = b"concurrent_big_key";

    // Prepare distinct ASCII payloads (valid UTF-8) of ~120KB each.
    let mut payloads: Vec<Vec<u8>> = Vec::new();
    for i in 0..8 {
        let ch = (b'a' + (i as u8 % 26)) as u8;
        payloads.push(vec![ch; 120_000]);
    }

    // Spawn concurrent tasks that perform a single large SET each.
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let gw = RedisGateway::new(db.clone());
            let payload = payloads[i].clone();
            let key = key.to_vec();
            tokio::spawn(async move {
                let _ = gw.set(&key, &payload, Flags::None).await;
            })
        })
        .collect();

    let _ = join_all(handles).await;

    // Read back the stored value and ensure it matches exactly one of the payloads.
    let res = gw.get(key).await;
    match res {
        Frame::BulkString(bytes) => {
            let inner = &bytes[..];
            // Check equality with any payload
            let mut matched = false;
            for p in payloads.iter() {
                if inner == p.as_slice() {
                    matched = true;
                    break;
                }
            }
            assert!(matched, "Final value does not match any writer payload");
        }
        other => panic!("unexpected frame returned: {:?}", other),
    }
}
