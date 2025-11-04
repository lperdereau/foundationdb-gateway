use crate::gateway::RedisGateway;
use crate::server::datamodel::AuthDataModel;
use crate::server::operations::ServerOperations;
use redis_protocol::resp2::types::OwnedFrame as Frame;

impl ServerOperations for RedisGateway {
    async fn get_user(&self, username: &[u8]) -> Frame {
        let dm = AuthDataModel::new(self.fdb.clone());
        match dm.get_user(username).await {
            Ok(Some(u)) => {
                let mut arr = Vec::new();
                arr.push(Frame::BulkString(username.to_vec()));
                if let Some(r) = u.rules { arr.push(Frame::BulkString(r.into_bytes())); } else { arr.push(Frame::Null); }
                Frame::Array(arr)
            }
            Ok(None) => Frame::Error("ERR no such user".into()),
            Err(e) => Frame::Error(format!("ERR acl backend error: {}", e)),
        }
    }

    async fn verify_user(&self, username: &[u8], password: &[u8]) -> Frame {
        let dm = AuthDataModel::new(self.fdb.clone());
        match dm.get_user(username).await {
            Ok(Some(u)) => {
                if u.hash.starts_with(b"$2") {
                    let stored_s = String::from_utf8_lossy(&u.hash);
                    let pw_s = String::from_utf8_lossy(password);
                    match bcrypt::verify(pw_s.as_ref(), stored_s.as_ref()) {
                        Ok(true) => Frame::SimpleString(b"OK".to_vec()),
                        Ok(false) => Frame::Error("ERR invalid password".into()),
                        Err(e) => Frame::Error(format!("bcrypt verify error: {:?}", e)),
                    }
                } else if u.hash.as_slice() == password {
                    Frame::SimpleString(b"OK".to_vec())
                } else {
                    Frame::Error("ERR invalid password".into())
                }
            }
            Ok(None) => Frame::Error("ERR no such user".into()),
            Err(e) => Frame::Error(format!("ERR auth backend error: {}", e)),
        }
    }

    async fn set_user(&self, username: &[u8], password: &[u8], rules: Option<&[u8]>) -> Frame {
        let dm = AuthDataModel::new(self.fdb.clone());
        match dm.set_user(username, password, rules).await {
            Ok(()) => Frame::SimpleString(b"OK".to_vec()),
            Err(e) => Frame::Error(format!("ERR acl setuser failed: {}", e)),
        }
    }

    async fn del_user(&self, username: &[u8]) -> Frame {
        let dm = AuthDataModel::new(self.fdb.clone());
        match dm.del_user(username).await {
            Ok(()) => Frame::SimpleString(b"OK".to_vec()),
            Err(e) => Frame::Error(format!("ERR acl deluser failed: {}", e)),
        }
    }

    async fn list_users(&self) -> Frame {
        let dm = AuthDataModel::new(self.fdb.clone());
        match dm.list_users().await {
            Ok(list) => {
                let arr = list.into_iter().map(|s| Frame::BulkString(s.into_bytes())).collect();
                Frame::Array(arr)
            }
            Err(e) => Frame::Error(format!("ERR acl list failed: {}", e)),
        }
    }
}
