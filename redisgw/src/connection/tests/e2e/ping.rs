use tokio::io::{AsyncWriteExt, AsyncReadExt};
use redis_protocol::resp2::{encode::encode, decode::decode, types::{OwnedFrame as Frame, Resp2Frame}};

#[tokio::test]
async fn test_e2e_ping() {
    // Bootstrap test server (FDB + RedisGateway) via helper function
    crate::with_e2e_server!(srv_handle, stream);
    let mut stream = stream.expect("stream");

    // PING without args -> PONG (SimpleString)
    let req_ping = Frame::Array(vec![Frame::BulkString(b"PING".to_vec())]);
    let mut out = vec![0u8; req_ping.encode_len(false)];
    let _ = encode(&mut out, &req_ping, false);
    stream.write_all(&out).await.expect("write ping");

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.expect("read");
    let opt = decode(&buf[..n]).expect("decode");
    let (frame, _used) = opt.expect("frame");
    assert!(matches!(frame, Frame::SimpleString(s) if s == b"PONG".to_vec()));

    // PING with message -> return message
    let req_ping2 = Frame::Array(vec![
        Frame::BulkString(b"PING".to_vec()),
        Frame::BulkString(b"hello".to_vec()),
    ]);
    let mut out = vec![0u8; req_ping2.encode_len(false)];
    let _ = encode(&mut out, &req_ping2, false);
    stream.write_all(&out).await.expect("write ping2");

    let n = stream.read(&mut buf).await.expect("read2");
    let opt = decode(&buf[..n]).expect("decode2");
    let (frame, _used) = opt.expect("frame2");
    assert!(matches!(frame, Frame::SimpleString(s) if s == b"hello".to_vec()));

    // Tear down
    drop(stream);
    srv_handle.abort();
}
