use crate::command::CommandHandler;
use crate::gateway::RedisGateway;
use redis_protocol::resp2::{
    decode::decode,
    encode::encode,
    types::{OwnedFrame as Frame, Resp2Frame},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    addr: String,
    handler: CommandHandler,
}

impl Server {
    pub fn new(addr: impl Into<String>, gateway: RedisGateway) -> Self {
        Self {
            addr: addr.into(),
            handler: CommandHandler::new(gateway),
        }
    }

    pub async fn start(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Listening on {}", self.addr);

        loop {
            let (socket, _) = listener.accept().await?;
            tokio::spawn(Self::handle_connection(socket, self.handler.clone()));
        }
    }

    async fn handle_connection(mut socket: TcpStream, handler: CommandHandler) {
        let mut buf = vec![0u8; 4096];
        let mut offset = 0;

        loop {
            let n = match socket.read(&mut buf[offset..]).await {
                Ok(0) => return, // connection closed
                Ok(n) => n,
                Err(_) => return,
            };

            let mut consumed = 0;
            let mut frames = Vec::new();

            // Parse all complete frames from the buffer
            while let Ok(Some((frame, used))) = decode(&buf[consumed..offset + n]) {
                frames.push(frame);
                consumed += used;
            }

            // Move any leftover bytes to the front of the buffer
            if consumed < offset + n {
                buf.copy_within(consumed..offset + n, 0);
                offset = offset + n - consumed;
            } else {
                offset = 0;
            }

            for frame in frames {
                let response = Self::process_command(&frame, &handler).await;
                let mut out = vec![0u8; response.encode_len(false)];
                let _ = encode(&mut out, &response, false);
                let _ = socket.write_all(&out).await;
            }
        }
    }

    async fn process_command(frame: &Frame, handler: &CommandHandler) -> Frame {
        match frame {
            Frame::Array(arr) if !arr.is_empty() => {
                if let Frame::BulkString(cmd) = &arr[0] {
                    // Collect arguments as Vec<&[u8]>
                    let args: Vec<&[u8]> = arr[1..]
                        .iter()
                        .filter_map(|f| match f {
                            Frame::BulkString(arg) => Some(arg.as_slice()),
                            Frame::SimpleString(arg) => Some(arg.as_slice()),
                            _ => None,
                        })
                        .collect();

                    handler
                        .handle(std::str::from_utf8(cmd).unwrap_or(""), args)
                        .await
                } else {
                    Frame::Error("ERR invalid command".into())
                }
            }
            _ => Frame::Error("ERR invalid command".into()),
        }
    }
}
