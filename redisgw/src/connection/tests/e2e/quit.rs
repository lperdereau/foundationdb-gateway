use tokio::io::{AsyncWriteExt, AsyncReadExt};
use redis_protocol::resp2::{encode::encode, decode::decode, types::{OwnedFrame as Frame, Resp2Frame}};

#[tokio::test]
async fn test_e2e_quit() {
    // Start test server and connect
    crate::with_e2e_server!(srv_handle, stream);
    let mut stream = stream.expect("stream");

    // Send QUIT
    let req_quit = Frame::Array(vec![Frame::BulkString(b"QUIT".to_vec())]);
    let mut out = vec![0u8; req_quit.encode_len(false)];
    let _ = encode(&mut out, &req_quit, false);
    stream.write_all(&out).await.expect("write quit");

    // Read reply
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.expect("read reply");
    let opt = decode(&buf[..n]).expect("decode");
    let (frame, _used) = opt.expect("frame");
    // Expect OK SimpleString
    assert!(matches!(frame, Frame::SimpleString(s) if s == b"OK".to_vec()));

    // After reply the server should close the connection; subsequent read returns 0
    let n2 = stream.read(&mut buf).await.expect("read after close");
    assert_eq!(n2, 0);

    // Tear down server
    srv_handle.abort();
}
