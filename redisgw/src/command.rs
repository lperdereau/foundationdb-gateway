use crate::gateway::RedisGateway;
use redis_protocol::resp2::types::OwnedFrame as Frame;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

#[derive(Clone)]
pub struct CommandHandler {
    gateway: RedisGateway,
    map: Arc<std::collections::HashMap<String, Arc<CmdHandler>>>,
}

pub type CmdHandler = dyn Fn(RedisGateway, Vec<Vec<u8>>) -> BoxFuture + Send + Sync + 'static;

pub type CmdMap = HashMap<String, Arc<CmdHandler>>;

/// Boxed future alias to reduce repetition in handler types.
/// Use the `futures` crate generic boxed future specialized to our `Frame` type.
/// Boxed future alias returning (Frame, should_close)
pub type BoxFuture = futures::future::BoxFuture<'static, (Frame, bool)>;

/// Convenience boxed handler type (sized) used when constructing handlers.
pub type BoxedCmdHandler = Box<dyn Fn(RedisGateway, Vec<Vec<u8>>) -> BoxFuture + Send + Sync + 'static>;

/// Helper which accepts a closure returning a Future<Output = Frame> and adapts
/// it into a handler that returns (Frame, false).
pub fn boxed_handler<F, Fut>(f: F) -> Arc<CmdHandler>
where
    F: Fn(RedisGateway, Vec<Vec<u8>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Frame> + Send + 'static,
{
    Arc::from(Box::new(move |gw: RedisGateway, args: Vec<Vec<u8>>| {
        let fut = f(gw, args);
        Box::pin(async move { (fut.await, false) }) as BoxFuture
    }) as BoxedCmdHandler)
}

/// Helper which accepts a closure returning Future<Output = (Frame,bool)> and
/// returns it directly as a CmdHandler.
pub fn boxed_handler_with_signal<F, Fut>(f: F) -> Arc<CmdHandler>
where
    F: Fn(RedisGateway, Vec<Vec<u8>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = (Frame, bool)> + Send + 'static,
{
    Arc::from(Box::new(move |gw: RedisGateway, args: Vec<Vec<u8>>| {
        Box::pin(f(gw, args)) as BoxFuture
    }) as BoxedCmdHandler)
}

/// Macro helper to create a boxed, pinned, Arc-wrapped command handler from an
/// async block or expression that returns a `Frame`. The helper will wrap the
/// result into `(Frame, false)` so the server knows not to close the
/// connection.
#[macro_export]
macro_rules! command_handler {
    (|$gw:ident, $args:ident| $body:expr) => {{
        ::std::sync::Arc::from(::std::boxed::Box::new(move |$gw: $crate::gateway::RedisGateway, $args: Vec<Vec<u8>>| {
            ::std::boxed::Box::pin(async move {
                let __res = $body.await;
                (__res, false)
            }) as $crate::command::BoxFuture
        }) as $crate::command::BoxedCmdHandler)
    }};
}

/// Define a `static` Lazy Arc-wrapped command handler.
/// Usage: `crate::command_handler_static!(NAME, |gw, args| async move { ... });`
#[macro_export]
macro_rules! command_handler_static {
    ($name:ident, |$gw:ident, $args:ident| $body:expr) => {
        static $name: ::once_cell::sync::Lazy<::std::sync::Arc<$crate::command::CmdHandler>> =
            ::once_cell::sync::Lazy::new(|| { $crate::command_handler!(|$gw, $args| $body) });
    };
}

/// Create a handler from an async block that returns `(Frame, bool)` so the
/// handler itself can signal the server to close the connection.
#[macro_export]
macro_rules! command_handler_signal {
    (|$gw:ident, $args:ident| $body:expr) => {{
        ::std::sync::Arc::from(::std::boxed::Box::new(move |$gw: $crate::gateway::RedisGateway, $args: Vec<Vec<u8>>| {
            ::std::boxed::Box::pin($body) as $crate::command::BoxFuture
        }) as $crate::command::BoxedCmdHandler)
    }};
}

/// Define a static Lazy Arc-wrapped command handler that can return a signal.
#[macro_export]
macro_rules! command_handler_static_with_signal {
    ($name:ident, |$gw:ident, $args:ident| $body:expr) => {
        static $name: ::once_cell::sync::Lazy<::std::sync::Arc<$crate::command::CmdHandler>> =
            ::once_cell::sync::Lazy::new(|| { $crate::command_handler_signal!(|$gw, $args| $body) });
    };
}

fn build_command_map() -> CmdMap {
    let mut map: CmdMap = HashMap::new();

    // Register connection commands
    if let Ok(conn_map) = std::panic::catch_unwind(crate::connection::commands::commands) {
        for (k, h) in conn_map.into_iter() {
            map.insert(k.to_ascii_uppercase(), h);
        }
    }

    // Register string commands
    if let Ok(str_map) = std::panic::catch_unwind(crate::string::commands::commands) {
        for (k, h) in str_map.into_iter() {
            map.insert(k.to_ascii_uppercase(), h);
        }
    }

    // Register list commands
    if let Ok(list_map) = std::panic::catch_unwind(crate::list::commands::commands) {
        for (k, h) in list_map.into_iter() {
            map.insert(k.to_ascii_uppercase(), h);
        }
    }

    // Register set commands
    if let Ok(set_map) = std::panic::catch_unwind(crate::set::commands::commands) {
        for (k, h) in set_map.into_iter() {
            map.insert(k.to_ascii_uppercase(), h);
        }
    }

    map
}


impl CommandHandler {
    pub fn new(gateway: RedisGateway) -> Self {
        let map = build_command_map();
        Self { gateway, map: Arc::new(map) }
    }

    pub async fn handle(&self, cmd_str: &str, args: Vec<&[u8]>) -> (Frame, bool) {
        let key = cmd_str.to_ascii_uppercase();
        if let Some(h) = self.map.get(&key) {
            let owned_args = args.iter().map(|s| s.to_vec()).collect::<Vec<_>>();
            (h)(self.gateway.clone(), owned_args).await
        } else {
            let args_str: String = args
                .iter()
                .filter_map(|s| std::str::from_utf8(s).ok())
                .collect::<Vec<&str>>()
                .join(" ");
            (Frame::Error(format!(
                "ERR unknown command '{}', with args beginning with: '{}'",
                cmd_str, args_str
            )), false)
        }
    }
}
