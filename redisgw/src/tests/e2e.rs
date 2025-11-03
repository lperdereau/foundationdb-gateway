use redis_protocol::resp2::{encode::encode, decode::decode, types::{OwnedFrame as Frame, Resp2Frame}};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_e2e_set_get_via_tcp() {
    // Bootstrap test server (FDB + RedisGateway) via helper macro
    with_e2e_server!(bind_addr, srv_handle);

    // Wait a little for server to start accepting connections
    let mut connected = false;
    let mut stream = None;
    for _ in 0..10 {
        match tokio::net::TcpStream::connect(&bind_addr).await {
            Ok(s) => { stream = Some(s); connected = true; break; }
            Err(_) => sleep(Duration::from_millis(50)).await,
        }
    }
    assert!(connected, "could not connect to server");
    let mut stream = stream.expect("stream");

    // Build SET command: ["SET", "e2e_key", "hello"]
    let req_set = Frame::Array(vec![
        Frame::BulkString(b"SET".to_vec()),
        Frame::BulkString(b"e2e_key".to_vec()),
        Frame::BulkString(b"hello".to_vec()),
    ]);
    let mut out = vec![0u8; req_set.encode_len(false)];
    let _ = encode(&mut out, &req_set, false);
    stream.write_all(&out).await.expect("write set");

    // Read response
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.expect("read");
    let opt = decode(&buf[..n]).expect("decode");
    let (frame, _used) = opt.expect("frame");
    // Expect OK
    assert!(matches!(frame, Frame::SimpleString(s) if s == b"OK".to_vec()));

    // Send GET
    let req_get = Frame::Array(vec![
        Frame::BulkString(b"GET".to_vec()),
        Frame::BulkString(b"e2e_key".to_vec()),
    ]);
    let mut out = vec![0u8; req_get.encode_len(false)];
    let _ = encode(&mut out, &req_get, false);
    stream.write_all(&out).await.expect("write get");

    let n = stream.read(&mut buf).await.expect("read2");
    let opt = decode(&buf[..n]).expect("decode2");
    let (frame, _used) = opt.expect("frame2");
    // Gateway returns SimpleString with quotes: "hello"
    assert!(matches!(frame, Frame::SimpleString(s) if s == b"\"hello\"".to_vec()));

    // Tear down: drop stream and abort server
    drop(stream);
    srv_handle.abort();
}
