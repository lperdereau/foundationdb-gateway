use crate::gateway::RedisGateway;
use crate::connection::operations::ConnectionOperations;
use crate::server::operations::ServerOperations;
use redis_protocol::resp2::types::OwnedFrame as Frame;

impl ConnectionOperations for RedisGateway {
    async fn ping(&self, message: Vec<&[u8]>) -> Frame {
        let msg = if let Some(arg) = message.first() {
            std::str::from_utf8(arg).ok()
        } else {
            None
        };

        match msg {
            Some(res) => Frame::BulkString(res.into()),
            None => Frame::SimpleString(b"PONG".to_vec()),
        }
    }

    async fn echo(&self, message: Vec<&[u8]>) -> Frame {
        if let Some(arg) = message.first() {
            Frame::BulkString(arg.to_vec())
        } else {
            Frame::Error("ERR wrong number of arguments for 'ECHO' command".into())
        }
    }

    async fn hello(&self, _args: Vec<&[u8]>) -> Frame {
        // Minimal HELLO implementation: acknowledge handshake
        Frame::SimpleString(b"OK".to_vec())
    }

    async fn reset(&self) -> Frame {
        Frame::SimpleString(b"OK".to_vec())
    }

    async fn select(&self, index: &[u8]) -> Frame {
        match std::str::from_utf8(index).ok().and_then(|s| s.parse::<i64>().ok()) {
            Some(_) => Frame::SimpleString(b"OK".to_vec()),
            None => Frame::Error("ERR invalid DB index".into()),
        }
    }

    async fn auth(&self, username: Option<&[u8]>, password: &[u8]) -> Frame {
        let user = username.unwrap_or(b"default");
        let reply = self.verify_user(user, password).await;
        // if verification returned OK, mark socket as authenticated
        if let Frame::SimpleString(s) = &reply {
            if s == b"OK" {
                if let Some(sc) = &self.socket_cfg {
                    if let Ok(mut w) = sc.write() {
                        w.authenticated_user = Some(String::from_utf8_lossy(user).into_owned());
                    }
                }
            }
        }
        reply
    }

    async fn client_getname(&self) -> Frame {
        // No per-connection state available here; return Null
        Frame::Null
    }

    async fn client_setname(&self, _name: &[u8]) -> Frame {
        // No per-connection state; accept but do nothing
        Frame::SimpleString(b"OK".to_vec())
    }

    async fn quit(&self) -> Frame {
        if let Some(sc) = &self.socket_cfg {
            if let Ok(mut w) = sc.write() {
                w.mark_close();
            }
        }
        Frame::SimpleString(b"OK".to_vec())
    }
}
