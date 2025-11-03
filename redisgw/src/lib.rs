pub mod command;
pub mod datamodel;
pub mod gateway;
pub(crate) mod operations;
pub mod server;


#[cfg(test)]
mod tests {

	// Helper macro to bootstrap an e2e server in tests. Defined here so child test
	// modules can use it directly. It expands to a `let` binding that awaits the
	// async helper `crate::tests::util::spawn_test_server()` and yields
	// `(guard, addr, handle)`.
	macro_rules! with_e2e_server {
		($addr:ident, $handle:ident) => {
			let ($addr, $handle) = crate::tests::util::spawn_test_server().await;
		};
	}
	pub(crate) mod util;
	pub mod e2e;
	pub mod unit;
}
