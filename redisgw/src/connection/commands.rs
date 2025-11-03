use crate::command::CmdMap;
use crate::connection::operations::ConnectionOperations;

crate::command_handler_static!(PING, |gw, args| async move {
    let refs: Vec<&[u8]> = args.iter().map(|v| v.as_slice()).collect();
    gw.ping(refs).await
});

/// Return a map of command name -> handler for connection-related commands.
pub fn commands() -> CmdMap {
    let mut m: CmdMap = CmdMap::new();
    m.insert("PING".to_string(), PING.clone());
    m
}
