use tokio::io::{AsyncWriteExt, AsyncReadExt};
use redis_protocol::resp2::{decode::decode, encode::encode, types::{OwnedFrame as Frame, Resp2Frame}};

#[tokio::test]
async fn test_e2e_acl_whoami() {
    // Start server and connect
    crate::with_e2e_server!(srv_handle, stream);
    let mut stream = stream.expect("stream");

    // ACL SETUSER testuser secret on
    let setreq = Frame::Array(vec![
        Frame::BulkString(b"ACL".to_vec()),
        Frame::BulkString(b"SETUSER".to_vec()),
        Frame::BulkString(b"testuser".to_vec()),
        Frame::BulkString(b"secret".to_vec()),
        Frame::BulkString(b"on".to_vec()),
    ]);
    let mut out = vec![0u8; setreq.encode_len(false)];
    let _ = encode(&mut out, &setreq, false);
    stream.write_all(&out).await.expect("write setuser");

    // Read reply
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.expect("read reply");
    let opt = decode(&buf[..n]).expect("decode");
    let (frame, _used) = opt.expect("frame");
    assert!(matches!(frame, Frame::SimpleString(s) if s == b"OK".to_vec()));

    // ACL WHOAMI before auth => should be "default"
    let who_req = Frame::Array(vec![
        Frame::BulkString(b"ACL".to_vec()),
        Frame::BulkString(b"WHOAMI".to_vec()),
    ]);
    let mut out2 = vec![0u8; who_req.encode_len(false)];
    let _ = encode(&mut out2, &who_req, false);
    stream.write_all(&out2).await.expect("write whoami");

    let n2 = stream.read(&mut buf).await.expect("read whoami reply");
    let opt2 = decode(&buf[..n2]).expect("decode2");
    let (frame2, _used2) = opt2.expect("frame2");
    match frame2 {
        Frame::BulkString(bs) => assert_eq!(bs, b"default".to_vec()),
        other => panic!("unexpected whoami reply before auth: {:?}", other),
    }

    // Now AUTH as testuser -> should authenticate this connection
    let auth_req = Frame::Array(vec![
        Frame::BulkString(b"AUTH".to_vec()),
        Frame::BulkString(b"testuser".to_vec()),
        Frame::BulkString(b"secret".to_vec()),
    ]);
    let mut out3 = vec![0u8; auth_req.encode_len(false)];
    let _ = encode(&mut out3, &auth_req, false);
    stream.write_all(&out3).await.expect("write auth");

    let n3 = stream.read(&mut buf).await.expect("read auth reply");
    let opt3 = decode(&buf[..n3]).expect("decode3");
    let (frame3, _used3) = opt3.expect("frame3");
    assert!(matches!(frame3, Frame::SimpleString(s) if s == b"OK".to_vec()));

    // ACL WHOAMI after auth -> should be "testuser"
    let who_req2 = Frame::Array(vec![
        Frame::BulkString(b"ACL".to_vec()),
        Frame::BulkString(b"whoami".to_vec()), // lowercased subcommand
    ]);
    let mut out4 = vec![0u8; who_req2.encode_len(false)];
    let _ = encode(&mut out4, &who_req2, false);
    stream.write_all(&out4).await.expect("write whoami2");

    let n4 = stream.read(&mut buf).await.expect("read whoami2 reply");
    let opt4 = decode(&buf[..n4]).expect("decode4");
    let (frame4, _used4) = opt4.expect("frame4");
    match frame4 {
        Frame::BulkString(bs) => assert_eq!(bs, b"testuser".to_vec()),
        other => panic!("unexpected whoami reply after auth: {:?}", other),
    }

    // Tear down server
    srv_handle.abort();
}
