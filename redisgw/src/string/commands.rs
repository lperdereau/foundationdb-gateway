use redis_protocol::resp2::types::OwnedFrame as Frame;
use crate::command::CmdMap;
use crate::string::operations::{StringOperations, parse_set_extra_args};

crate::command_handler_static!(SET, |gw, args| async move {
    // expecting at least key and value
    if args.len() < 2 {
        return Frame::Error("ERR wrong number of arguments for 'SET' command".into());
    }
    let key = args[0].as_slice();
    let val = args[1].as_slice();

    let extra_refs: Vec<&[u8]> = args.iter().skip(2).map(|v| v.as_slice()).collect();
    let flags = parse_set_extra_args(&extra_refs);
    gw.set(key, val, flags).await
});

crate::command_handler_static!(GET, |gw, args| async move {
    if args.is_empty() {
        return Frame::Error("ERR wrong number of arguments for 'GET' command".into());
    }
    let key = args[0].as_slice();
    gw.get(key).await
});

crate::command_handler_static!(DEL, |gw, args| async move {
    if args.is_empty() {
        return Frame::Error("ERR wrong number of arguments for 'DEL' command".into());
    }
    let key = args[0].as_slice();
    gw.del(key).await
});

crate::command_handler_static!(GETDEL, |gw, args| async move {
    if args.is_empty() {
        return Frame::Error("ERR wrong number of arguments for 'GETDEL' command".into());
    }
    let key = args[0].as_slice();
    gw.getdel(key).await
});

crate::command_handler_static!(INCR, |gw, args| async move {
    if args.is_empty() {
        return Frame::Error("ERR wrong number of arguments for 'INCR' command".into());
    }
    let key = args[0].as_slice();
    gw.incr(key).await
});

crate::command_handler_static!(DECR, |gw, args| async move {
    if args.is_empty() {
        return Frame::Error("ERR wrong number of arguments for 'DECR' command".into());
    }
    let key = args[0].as_slice();
    gw.decr(key).await
});

crate::command_handler_static!(INCRBY, |gw, args| async move {
    if args.len() < 2 {
        return Frame::Error("ERR wrong number of arguments for 'INCRBY' command".into());
    }
    let key = args[0].as_slice();
    let inc = args[1].as_slice();
    gw.incr_by(key, inc).await
});

crate::command_handler_static!(DECRBY, |gw, args| async move {
    if args.len() < 2 {
        return Frame::Error("ERR wrong number of arguments for 'DECRBY' command".into());
    }
    let key = args[0].as_slice();
    let dec = args[1].as_slice();
    gw.decr_by(key, dec).await
});

/// Return handlers for string-related commands (SET, GET)
pub fn commands() -> CmdMap {
    let mut m: CmdMap = CmdMap::new();

    m.insert("SET".to_string(), SET.clone());

    m.insert("GET".to_string(), GET.clone());

    m.insert("DEL".to_string(), DEL.clone());

    m.insert("GETDEL".to_string(), GETDEL.clone());

    m.insert("INCR".to_string(), INCR.clone());

    m.insert("DECR".to_string(), DECR.clone());

    m.insert("INCRBY".to_string(), INCRBY.clone());

    m.insert("DECRBY".to_string(), DECRBY.clone());

    m
}
