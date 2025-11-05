use redis_protocol::resp2::types::OwnedFrame as Frame;

/// Server-side operations trait (ACL, user management, server control)
pub trait ServerOperations {
    fn get_user(&self, username: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    fn verify_user(&self, username: &[u8], password: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    fn set_user(&self, username: &[u8], password: &[u8], rules: Option<&[u8]>) -> impl std::future::Future<Output = Frame> + Send;

    fn del_user(&self, username: &[u8]) -> impl std::future::Future<Output = Frame> + Send;

    fn list_users(&self) -> impl std::future::Future<Output = Frame> + Send;

    fn whoami(&self) -> impl std::future::Future<Output = Frame> + Send;
}
