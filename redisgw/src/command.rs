use crate::gateway::RedisGateway;
use crate::operations::RedisOperations;
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
            Some(Command::Ping) => Frame::SimpleString(self.gateway.ping(args).await.into()),
            Some(Command::Set) => match self.gateway.set(args[0], args[1]).await {
                Ok(_) => Frame::SimpleString("OK".into()),
                Err(err) => Frame::SimpleString(err.into()),
            },
            Some(Command::Get) => match self.gateway.get(args[0]).await {
                Some(result) => Frame::SimpleString(result.into()),
                None => Frame::Null,
            },
            Some(Command::Del) => match self.gateway.del(args[0]).await {
                Ok(int) => Frame::Integer(int),
                Err(err) => Frame::SimpleString(err.into()),
            },
            Some(Command::GetDel) => match self.gateway.getdel(args[0]).await {
                Some(result) => Frame::SimpleString(result.into()),
                None => Frame::Null,
            },
            Some(Command::Incr) => match self.gateway.incr(args[0]).await {
                Ok(int) => Frame::Integer(int),
                Err(err) => Frame::SimpleString(err.into()),
            },
            Some(Command::Decr) => match self.gateway.decr(args[0]).await {
                Ok(int) => Frame::Integer(int),
                Err(err) => Frame::SimpleString(err.into()),
            },
            Some(Command::IncrBy) => {
                match std::str::from_utf8(args[1])
                    .ok()
                    .and_then(|s| s.parse::<i64>().ok())
                {
                    Some(by) => match self.gateway.incr_by(args[0], by).await {
                        Ok(int) => Frame::Integer(int),
                        Err(err) => Frame::SimpleString(err.into()),
                    },
                    None => Frame::Error("ERR value is not an integer or out of range".into()),
                }
            }
            Some(Command::DecrBy) => {
                match std::str::from_utf8(args[1])
                    .ok()
                    .and_then(|s| s.parse::<i64>().ok())
                {
                    Some(by) => match self.gateway.decr_by(args[0], by).await {
                        Ok(int) => Frame::Integer(int),
                        Err(err) => Frame::SimpleString(err.into()),
                    },
                    None => Frame::Error("ERR value is not an integer or out of range".into()),
                }
            }

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
