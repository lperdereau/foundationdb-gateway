use crate::gateway::RedisGateway;
use crate::operations::{
    Flags,
    SetFlags,
    ConnectionOperations,
    StringOperations,
};
use redis_protocol::resp2::types::OwnedFrame as Frame;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Ping,
    Set,
    Get,
    Del,
    GetDel,
    Incr,
    Decr,
    IncrBy,
    DecrBy,
    // Add more commands as needed
}

impl Command {
    pub fn from_str(cmd: &str) -> Option<Self> {
        match cmd.to_ascii_uppercase().as_str() {
            "PING" => Some(Command::Ping),
            "SET" => Some(Command::Set),
            "GET" => Some(Command::Get),
            "DEL" => Some(Command::Del),
            "GETDEL" => Some(Command::GetDel),
            "INCR" => Some(Command::Incr),
            "DECR" => Some(Command::Decr),
            "INCRBY" => Some(Command::Incr),
            "DECRBY" => Some(Command::Decr),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct CommandHandler {
    gateway: RedisGateway,
}

impl CommandHandler {
    pub fn new(gateway: RedisGateway) -> Self {
        Self { gateway }
    }

    pub async fn handle(&self, cmd_str: &str, args: Vec<&[u8]>) -> Frame {
        match Command::from_str(cmd_str) {
            Some(Command::Ping) => self.gateway.ping(args).await,
            Some(Command::Set) => {
                self.gateway
                    .set(
                        args[0],
                        args[1],
                        parse_extra_args(Command::Set, args.get(2..).unwrap_or(&[])),
                    )
                    .await
            }
            Some(Command::Get) => self.gateway.get(args[0]).await,
            Some(Command::Del) => self.gateway.del(args[0]).await,
            Some(Command::GetDel) => self.gateway.getdel(args[0]).await,
            Some(Command::Incr) => self.gateway.incr(args[0]).await,
            Some(Command::Decr) => self.gateway.decr(args[0]).await,
            Some(Command::IncrBy) => self.gateway.incr_by(args[0], args[1]).await,
            Some(Command::DecrBy) => self.gateway.decr_by(args[0], args[1]).await,
            None => {
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
}

fn parse_extra_args(cmd: Command, extra_args: &[&[u8]]) -> Flags {
    if extra_args.len() < 1 {
        return Flags::None;
    }

    match cmd {
        Command::Set => Flags::Set(parse_set_extra_args(extra_args)),
        _ => Flags::None,
    }
}

fn parse_set_extra_args(extra_args: &[&[u8]]) -> SetFlags {
    // Assume SetFlags, SetMethod, SetTTL are defined in crate::operations
    use crate::operations::{SetFlags, SetMethod, SetTTL};

    let mut method: Option<SetMethod> = None;
    let mut ttl: Option<SetTTL> = None;
    let mut get: bool = false;

    let mut args = extra_args.iter().peekable();
    while let Some(arg) = args.next() {
        let s = match std::str::from_utf8(arg) {
            Ok(s) => s.to_ascii_uppercase(),
            Err(_) => continue, // skip invalid utf-8
        };
        match s.as_str() {
            "NX" => method = Some(SetMethod::NX),
            "XX" => method = Some(SetMethod::XX),
            "GET" => get = true,
            "EX" | "PX" | "EXAT" | "PXAT" => {
                let next = match args.next() {
                    Some(n) => n,
                    None => continue, // skip if missing argument
                };
                let n = match std::str::from_utf8(next)
                    .ok()
                    .and_then(|x| x.parse::<u64>().ok())
                {
                    Some(val) => val,
                    None => continue, // skip if invalid value
                };
                ttl = Some(match s.as_str() {
                    "EX" => SetTTL::EX(n),
                    "PX" => SetTTL::PX(n),
                    "EXAT" => SetTTL::EXAT(n),
                    "PXAT" => SetTTL::PXAT(n),
                    _ => unreachable!(),
                });
            }
            "KEPPTTL" => ttl = Some(SetTTL::KEPPTTL),
            _ => {} // ignore unknown options
        }
    }

    SetFlags { method, ttl, get }
}
