use crate::command::CmdMap;
use crate::connection::operations::ConnectionOperations;
use redis_protocol::resp2::types::OwnedFrame as Frame;

crate::command_handler_static!(PING, |gw, args| async move {
    let refs: Vec<&[u8]> = args.iter().map(|v| v.as_slice()).collect();
    gw.ping(refs).await
});

crate::command_handler_static!(ECHO, |gw, args| async move {
    let refs: Vec<&[u8]> = args.iter().map(|v| v.as_slice()).collect();
    gw.echo(refs).await
});

crate::command_handler_static!(HELLO, |gw, args| async move {
    let refs: Vec<&[u8]> = args.iter().map(|v| v.as_slice()).collect();
    gw.hello(refs).await
});

crate::command_handler_static!(RESET, |gw, _args| async move {
    gw.reset().await
});

crate::command_handler_static!(SELECT, |gw, args| async move {
    if args.is_empty() { return Frame::Error("ERR wrong number of arguments for 'SELECT' command".into()); }
    gw.select(args[0].as_slice()).await
});

crate::command_handler_static!(AUTH, |gw, args| async move {
    if args.is_empty() { return Frame::Error("ERR wrong number of arguments for 'AUTH' command".into()); }
    if args.len() == 1 {
        gw.auth(None, args[0].as_slice()).await
    } else {
        gw.auth(Some(args[0].as_slice()), args[1].as_slice()).await
    }
});

crate::command_handler_static!(CLIENT_CMD, |gw, args| async move {
    // CLIENT subcommands are handled here
    if args.is_empty() { return Frame::Error("ERR wrong number of arguments for 'CLIENT' command".into()); }
    // interpret first arg as subcommand
    let sub = args[0].as_slice();
    match std::str::from_utf8(sub).unwrap_or("").to_ascii_uppercase().as_str() {
        "GETNAME" => gw.client_getname().await,
        "SETNAME" => {
            if args.len() < 2 { Frame::Error("ERR wrong number of arguments for 'CLIENT SETNAME'".into()) }
            else { gw.client_setname(args[1].as_slice()).await }
        }
        _ => Frame::Error("ERR CLIENT subcommand not implemented".into()),
    }
});

crate::command_handler_static!(QUIT, |gw, _args| async move {
    // mark connection to be closed after reply
    if let Some(sc) = &gw.socket_cfg {
        if let Ok(mut w) = sc.write() {
            w.mark_close();
        }
    }
    gw.quit().await
});

/// Return a map of command name -> handler for connection-related commands.
pub fn commands() -> CmdMap {
    let mut m: CmdMap = CmdMap::new();
    m.insert("PING".to_string(), PING.clone());
    m.insert("ECHO".to_string(), ECHO.clone());
    m.insert("HELLO".to_string(), HELLO.clone());
    m.insert("RESET".to_string(), RESET.clone());
    m.insert("SELECT".to_string(), SELECT.clone());
    m.insert("AUTH".to_string(), AUTH.clone());
    m.insert("CLIENT".to_string(), CLIENT_CMD.clone());
    m.insert("QUIT".to_string(), QUIT.clone());
    m
}
