use crate::gateway::RedisGateway;
use crate::server::datamodel::AuthDataModel;
use crate::server::operations::ServerOperations;

impl ServerOperations for RedisGateway {
    async fn get_user(&self, username: &[u8]) -> Result<Option<crate::server::datamodel::UserInfo>, String> {
        let dm = AuthDataModel::new(self.fdb.clone());
        dm.get_user(username).await
    }

    async fn verify_user(&self, username: &[u8], password: &[u8]) -> Result<bool, String> {
        let dm = AuthDataModel::new(self.fdb.clone());
        match dm.get_user(username).await {
            Ok(Some(u)) => {
                if u.hash.starts_with(b"$2") {
                    let stored_s = String::from_utf8_lossy(&u.hash);
                    let pw_s = String::from_utf8_lossy(password);
                    match bcrypt::verify(pw_s.as_ref(), stored_s.as_ref()) {
                        Ok(v) => Ok(v),
                        Err(e) => Err(format!("bcrypt verify error: {:?}", e)),
                    }
                } else {
                    Ok(u.hash.as_slice() == password)
                }
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn set_user(&self, username: &[u8], password: &[u8], rules: Option<&[u8]>) -> Result<(), String> {
        let dm = AuthDataModel::new(self.fdb.clone());
        dm.set_user(username, password, rules).await
    }

    async fn del_user(&self, username: &[u8]) -> Result<(), String> {
        let dm = AuthDataModel::new(self.fdb.clone());
        dm.del_user(username).await
    }

    async fn list_users(&self) -> Result<Vec<String>, String> {
        let dm = AuthDataModel::new(self.fdb.clone());
        dm.list_users().await
    }
}
