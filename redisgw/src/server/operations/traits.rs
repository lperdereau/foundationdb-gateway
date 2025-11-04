use crate::server::datamodel::UserInfo;

/// Server-side operations trait (ACL, user management, server control)
pub trait ServerOperations {
    fn get_user(&self, username: &[u8]) -> impl std::future::Future<Output = Result<Option<UserInfo>, String>> + Send;

    fn verify_user(&self, username: &[u8], password: &[u8]) -> impl std::future::Future<Output = Result<bool, String>> + Send;

    fn set_user(&self, username: &[u8], password: &[u8], rules: Option<&[u8]>) -> impl std::future::Future<Output = Result<(), String>> + Send;

    fn del_user(&self, username: &[u8]) -> impl std::future::Future<Output = Result<(), String>> + Send;

    fn list_users(&self) -> impl std::future::Future<Output = Result<Vec<String>, String>> + Send;
}
