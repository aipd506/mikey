use super::*;
use std::io::Cursor;

#[test]
fn test_framing_roundtrip() {
    let frame = Frame::new(FrameType::Hello, b"{\"proto\":1}".to_vec());
    let mut buffer = Vec::new();
    write_frame(&mut buffer, &frame).unwrap();

    assert_eq!(buffer.len(), 5 + 11);
    assert_eq!(buffer[0], 0x00);
    assert_eq!(&buffer[1..5], &11u32.to_be_bytes());

    let mut cursor = Cursor::new(buffer);
    let parsed = read_frame(&mut cursor).unwrap();
    assert_eq!(parsed.frame_type, FrameType::Hello);
    assert_eq!(parsed.payload, b"{\"proto\":1}");
}

#[test]
fn test_frame_size_limit() {
    let mut evil = vec![0x00];
    evil.extend_from_slice(&(5 * 1024 * 1024u32).to_be_bytes());
    let mut cursor = Cursor::new(evil);
    let res = read_frame(&mut cursor);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().kind(), ErrorKind::InvalidData);
}

#[test]
fn test_media_header_roundtrip() {
    let header = MediaHeader {
        seq: 42,
        capture_ts: 1_700_000_000_123_456,
        codec: CODEC_PCM,
        reserved: 0,
    };
    let bytes = header.to_vec();
    assert_eq!(bytes.len(), MEDIA_HEADER_LEN);

    let parsed = MediaHeader::parse(&bytes).unwrap();
    assert_eq!(parsed, header);
}

#[test]
fn test_handshake_json() {
    let hello = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "test-device-uuid".into(),
        device_name: "Pixel 7".into(),
        level: 1,
        token: None,
        resume: None,
        caps: vec!["pcm".into()],
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: HelloPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.device_name, "Pixel 7");
    assert_eq!(parsed.level, 1);
}
