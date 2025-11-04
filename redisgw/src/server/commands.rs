use crate::command::CmdMap;
use redis_protocol::resp2::types::OwnedFrame as Frame;
use crate::server::operations::ServerOperations as _;
use crate::server::operations::parse::{parse_acl_command, ACLMethod};

crate::command_handler_static!(ACL_CMD, |gw, args| async move {
    if args.is_empty() {
        return Frame::Error("ERR wrong number of arguments for 'ACL' command".into());
    }
    // build borrowed refs for parsing
    let refs: Vec<&[u8]> = args.iter().map(|v| v.as_slice()).collect();
    match parse_acl_command(&refs) {
        Ok(ACLMethod::SetUser { user, password, rules }) => match gw.set_user(user, password, rules).await {
            Ok(()) => Frame::SimpleString(b"OK".to_vec()),
            Err(e) => Frame::Error(format!("ERR acl setuser failed: {}", e)),
        },
        Ok(ACLMethod::GetUser { user }) => match gw.get_user(user).await {
            Ok(Some(u)) => {
                let mut arr = Vec::new();
                arr.push(Frame::BulkString(user.to_vec()));
                if let Some(r) = u.rules { arr.push(Frame::BulkString(r.into_bytes())); } else { arr.push(Frame::Null); }
                Frame::Array(arr)
            }
            Ok(None) => Frame::Error("ERR no such user".into()),
            Err(e) => Frame::Error(format!("ERR acl backend error: {}", e)),
        },
        Ok(ACLMethod::DelUser { user }) => match gw.del_user(user).await {
            Ok(()) => Frame::SimpleString(b"OK".to_vec()),
            Err(e) => Frame::Error(format!("ERR acl deluser failed: {}", e)),
        },
        Ok(ACLMethod::List) => match gw.list_users().await {
            Ok(list) => {
                let arr = list.into_iter().map(|s| Frame::BulkString(s.into_bytes())).collect();
                Frame::Array(arr)
            }
            Err(e) => Frame::Error(format!("ERR acl list failed: {}", e)),
        },
        Err(e) => Frame::Error(e),
    }
});

pub fn commands() -> CmdMap {
    let mut m: CmdMap = CmdMap::new();
    m.insert("ACL".to_string(), ACL_CMD.clone());
    m
}
