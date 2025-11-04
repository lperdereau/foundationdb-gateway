use tokio::io::{AsyncWriteExt, AsyncReadExt};
use redis_protocol::resp2::{decode::decode, encode::encode, types::{OwnedFrame as Frame, Resp2Frame}};

#[tokio::test]
async fn test_e2e_acl_set_get_del() {
    // Start server and connect
    crate::with_e2e_server!(srv_handle, stream);
    let mut stream = stream.expect("stream");

    // ACL SETUSER testuser secret on
    let req = Frame::Array(vec![
        Frame::BulkString(b"ACL".to_vec()),
        Frame::BulkString(b"SETUSER".to_vec()),
        Frame::BulkString(b"testuser".to_vec()),
        Frame::BulkString(b"secret".to_vec()),
        Frame::BulkString(b"on".to_vec()),
    ]);
    let mut out = vec![0u8; req.encode_len(false)];
    let _ = encode(&mut out, &req, false);
    stream.write_all(&out).await.expect("write setuser");

    // Read reply
    let mut buf = vec![0u8; 2048];
    let n = stream.read(&mut buf).await.expect("read reply");
    let opt = decode(&buf[..n]).expect("decode");
    let (frame, _used) = opt.expect("frame");
    assert!(matches!(frame, Frame::SimpleString(s) if s == b"OK".to_vec()));

    // ACL GETUSER testuser
    let req2 = Frame::Array(vec![
        Frame::BulkString(b"ACL".to_vec()),
        Frame::BulkString(b"GETUSER".to_vec()),
        Frame::BulkString(b"testuser".to_vec()),
    ]);
    let mut out2 = vec![0u8; req2.encode_len(false)];
    let _ = encode(&mut out2, &req2, false);
    stream.write_all(&out2).await.expect("write getuser");

    let n2 = stream.read(&mut buf).await.expect("read getuser reply");
    let opt2 = decode(&buf[..n2]).expect("decode2");
    let (frame2, _used2) = opt2.expect("frame2");
    // Expect array reply
    match frame2 {
        Frame::Array(arr) => {
            assert!(!arr.is_empty());
        }
        other => panic!("unexpected reply: {:?}", other),
    }

    // ACL DELUSER testuser
    let req3 = Frame::Array(vec![
        Frame::BulkString(b"ACL".to_vec()),
        Frame::BulkString(b"DELUSER".to_vec()),
        Frame::BulkString(b"testuser".to_vec()),
    ]);
    let mut out3 = vec![0u8; req3.encode_len(false)];
    let _ = encode(&mut out3, &req3, false);
    stream.write_all(&out3).await.expect("write deluser");

    let n3 = stream.read(&mut buf).await.expect("read deluser reply");
    let opt3 = decode(&buf[..n3]).expect("decode3");
    let (frame3, _used3) = opt3.expect("frame3");
    assert!(matches!(frame3, Frame::SimpleString(s) if s == b"OK".to_vec()));

    // Tear down server
    srv_handle.abort();
}
