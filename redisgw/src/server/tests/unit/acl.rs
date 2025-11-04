use fdb_testcontainer::get_db_once;
use fdb::FoundationDB;

#[tokio::test]
async fn test_datamodel_set_get_del() {
    let guard = get_db_once().await;
    let db = FoundationDB::new(guard.clone());
    let dm = crate::server::datamodel::AuthDataModel::new(db);

    // ensure clean
    let _ = dm.del_user(b"testuser").await;

    // set user with plaintext password (will be bcrypt-hashed)
    dm.set_user(b"testuser", b"secret", Some(b"on")).await.expect("set_user");

    // get user
    let got = dm.get_user(b"testuser").await.expect("get_user");
    assert!(got.is_some(), "user must exist");
    let info = got.unwrap();
    // stored hash should start with bcrypt prefix
    assert!(info.hash.starts_with(b"$2"), "hash should be bcrypt");
    assert_eq!(info.rules.unwrap_or_default(), "on");

    // delete user
    dm.del_user(b"testuser").await.expect("del_user");
    let got2 = dm.get_user(b"testuser").await.expect("get_user after del");
    assert!(got2.is_none(), "user must be deleted");
}
