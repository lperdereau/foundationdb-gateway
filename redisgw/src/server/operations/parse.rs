use std::str;

/// Parsed ACL method and associated args (borrows slices from the original args array).
pub enum ACLMethod<'a> {
    SetUser { user: &'a [u8], password: &'a [u8], rules: Option<&'a [u8]> },
    GetUser { user: &'a [u8] },
    DelUser { user: &'a [u8] },
    WhoAmI,
    List,
}

/// Parse an `ACL` command (args: &[&[u8]] where args[0] is the subcommand).
/// Returns an `ACLMethod` describing the subcommand and its parsed arguments.
pub fn parse_acl_command<'a>(args: &'a [&'a [u8]]) -> Result<ACLMethod<'a>, String> {
    if args.is_empty() {
        return Err("ERR wrong number of arguments for 'ACL' command".into());
    }
    let sub = match str::from_utf8(args[0]) {
        Ok(s) => s.to_ascii_uppercase(),
        Err(_) => return Err("ERR invalid ACL subcommand".into()),
    };

    match sub.as_str() {
        "SETUSER" => {
            if args.len() < 3 {
                return Err("ERR wrong number of arguments for 'ACL SETUSER'".into());
            }
            let user = args[1];
            let pw = args[2];
            let rules = if args.len() >= 4 { Some(args[3]) } else { None };
            Ok(ACLMethod::SetUser { user, password: pw, rules })
        }
        "GETUSER" => {
            if args.len() < 2 { return Err("ERR wrong number of arguments for 'ACL GETUSER'".into()); }
            Ok(ACLMethod::GetUser { user: args[1] })
        }
        "DELUSER" => {
            if args.len() < 2 { return Err("ERR wrong number of arguments for 'ACL DELUSER'".into()); }
            Ok(ACLMethod::DelUser { user: args[1] })
        }
        "LIST" => Ok(ACLMethod::List),
        "WHOAMI" => Ok(ACLMethod::WhoAmI),
        other => Err(format!("ERR unknown ACL subcommand: {}", other)),
    }
}
