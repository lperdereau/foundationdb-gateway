use crate::command::CmdMap;
use redis_protocol::resp2::types::OwnedFrame as Frame;
use crate::server::operations::ServerOperations as _;
use crate::server::operations::parse::{parse_acl_command, ACLMethod};

crate::command_handler_static!(ACL_CMD, |gw, args| async move {
    if args.is_empty() {
        return Frame::Error("ERR wrong number of arguments for 'ACL' command".into());
    }

    let refs: Vec<&[u8]> = args.iter().map(|v| v.as_slice()).collect();
    match parse_acl_command(&refs) {
        Ok(ACLMethod::SetUser { user, password, rules }) => gw.set_user(user, password, rules).await,
        Ok(ACLMethod::GetUser { user }) => gw.get_user(user).await,
        Ok(ACLMethod::DelUser { user }) => gw.del_user(user).await,
        Ok(ACLMethod::List) => gw.list_users().await,
        Err(e) => Frame::Error(e),
    }
});

pub fn commands() -> CmdMap {
    let mut m: CmdMap = CmdMap::new();
    m.insert("ACL".to_string(), ACL_CMD.clone());
    m
}
