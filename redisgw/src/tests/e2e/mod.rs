// Helper macro to bootstrap an e2e server in tests. Defined here so child test
// modules can use it directly. It expands to a `let` binding that awaits the
// async helper `crate::tests::e2e::util::spawn_test_server()` and yields
// `(server_handle, stream)`.
macro_rules! with_e2e_server {
    ($handle:ident, $stream:ident) => {
        let ($handle, $stream) = crate::tests::e2e::util::spawn_test_server().await;
    };
}

mod util;
pub mod string;
