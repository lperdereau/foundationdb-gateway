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
/// Boxed future alias returning Frame
pub type BoxFuture = futures::future::BoxFuture<'static, Frame>;

/// Convenience boxed handler type (sized) used when constructing handlers.
pub type BoxedCmdHandler = Box<dyn Fn(RedisGateway, Vec<Vec<u8>>) -> BoxFuture + Send + Sync + 'static>;

/// Helper which accepts a closure returning a Future<Output = Frame> and adapts
/// it into a handler returning that Frame.
pub fn boxed_handler<F, Fut>(f: F) -> Arc<CmdHandler>
where
    F: Fn(RedisGateway, Vec<Vec<u8>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Frame> + Send + 'static,
{
    Arc::from(Box::new(move |gw: RedisGateway, args: Vec<Vec<u8>>| {
        let fut = f(gw, args);
        Box::pin(fut) as BoxFuture
    }) as BoxedCmdHandler)
}


/// Macro helper to create a boxed, pinned, Arc-wrapped command handler from an
/// async block or expression that returns a `Frame`.
#[macro_export]
macro_rules! command_handler {
    (|$gw:ident, $args:ident| $body:expr) => {{
        ::std::sync::Arc::from(::std::boxed::Box::new(move |$gw: $crate::gateway::RedisGateway, $args: Vec<Vec<u8>>| {
            ::std::boxed::Box::pin(async move {
                $body.await
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

// signal-style handlers were removed in favor of using per-socket `SocketConfig`
// to request connection closure.

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

    /// Access the base gateway used when the CommandHandler was constructed.
    /// This can be cloned and augmented with a per-connection SocketConfig.
    pub fn base_gateway(&self) -> RedisGateway {
        self.gateway.clone()
    }

    /// Handle a command using the provided per-connection `gateway` instance.
    /// Handlers return a `Frame`; any side-effect on connection lifecycle must
    /// be performed by mutating the per-connection `SocketConfig` available
    /// via `gateway.socket_cfg`.
    pub async fn handle(&self, gateway: crate::gateway::RedisGateway, cmd_str: &str, args: Vec<&[u8]>) -> Frame {
        let key = cmd_str.to_ascii_uppercase();
        if let Some(h) = self.map.get(&key) {
            let owned_args = args.iter().map(|s| s.to_vec()).collect::<Vec<_>>();
            (h)(gateway, owned_args).await
        } else {
            let args_str: String = args
                .iter()
                .filter_map(|s| std::str::from_utf8(s).ok())
                .collect::<Vec<&str>>()
                .join(" ");
            Frame::Error(format!(
                "ERR unknown command '{}', with args beginning with: '{}'",
                cmd_str, args_str
            ))
        }
    }
}
