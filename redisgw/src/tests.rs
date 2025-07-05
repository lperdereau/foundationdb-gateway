use crate::gateway::RedisGateway;
use crate::operations::RedisOperations;
use fdb::FoundationDB;
use fdb_testcontainer::get_db_once;

#[tokio::test]
async fn test_insert_record() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gw = RedisGateway::new(db);
    let _ = gw.set(b"key", b"value").await;
    let result = gw.get(b"key").await;
    assert_eq!(result, Some(b"\"value\"".to_vec()));
    let _ = gw.del(b"key").await;
    let result = gw.get(b"key").await;
    assert!(result.is_none());

    let _ = gw.set(b"key", b"value").await;
    let result = gw.getdel(b"key").await;
    assert_eq!(result, Some(b"\"value\"".to_vec()));
    let result = gw.getdel(b"key").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_increment_decrement_record() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gateway = RedisGateway::new(db);
    let key = b"counter";
    gateway.set(key, b"0").await.unwrap();
    let val = gateway.incr(key).await.unwrap();
    assert_eq!(val, 1);
    let val = gateway.incr(key).await.unwrap();
    assert_eq!(val, 2);
    let val = gateway.decr(key).await.unwrap();
    assert_eq!(val, 1);
    let val = gateway.decr(key).await.unwrap();
    assert_eq!(val, 0);
    let val = gateway.decr(key).await.unwrap();
    assert_eq!(val, -1);

    gateway.set(key, b"10").await.unwrap();
    let val = gateway.incr(key).await.unwrap();
    assert_eq!(val, 11);

    gateway.set(key, b"100").await.unwrap();
    let val = gateway.incr(key).await.unwrap();
    assert_eq!(val, 101);
    let _ = gateway.decr(key).await.unwrap();
    let val = gateway.decr(key).await.unwrap();
    assert_eq!(val, 99);

    gateway.del(key).await.unwrap();
}

#[tokio::test]
async fn test_increment_decrement_by_record() {
    let _guard = get_db_once().await;
    let db = FoundationDB::new(_guard.clone());
    let gateway = RedisGateway::new(db);
    let key = b"counter";
    gateway.set(key, b"0").await.unwrap();
    let val = gateway.incr_by(key, 10).await.unwrap();
    assert_eq!(val, 10);
    let val = gateway.incr_by(key, 100).await.unwrap();
    assert_eq!(val, 110);
    let val = gateway.decr_by(key, 109).await.unwrap();
    assert_eq!(val, 1);
    let val = gateway.decr_by(key, 10).await.unwrap();
    assert_eq!(val, -9);
    let val = gateway.incr_by(key, 8).await.unwrap();
    assert_eq!(val, -1);
}
