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

#[test]
fn test_skip_unknown_frame_type() {
    let mut buffer = Vec::new();
    // Unknown frame type 0x99 with 9-byte payload
    buffer.push(0x99);
    buffer.extend_from_slice(&9u32.to_be_bytes());
    buffer.extend_from_slice(b"discardme");

    // Followed by valid frame
    let hello_frame = Frame::new(FrameType::Hello, b"{\"proto\":1}".to_vec());
    write_frame(&mut buffer, &hello_frame).unwrap();

    let mut cursor = Cursor::new(buffer);
    let parsed = read_frame(&mut cursor).unwrap();
    assert_eq!(parsed.frame_type, FrameType::Hello);
    assert_eq!(parsed.payload, b"{\"proto\":1}");
}

#[test]
fn test_control_payload_roundtrip() {
    let ctrl = ControlPayload {
        audio: Some(ControlAudioPayload {
            ns: Some(true),
            ns_strength: Some(0.85),
            aec: Some(true),
            gate_db: Some(Some(-45.0)),
            muted: Some(false),
        }),
        video: Some(ControlVideoPayload {
            on: Some(true),
            lens: Some("front".to_string()),
            preview: Some(true),
            aspect: Some("16:9".to_string()),
            fps: Some(30),
        }),
    };
    let json = serde_json::to_string(&ctrl).unwrap();
    let parsed: ControlPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, ctrl);
}

#[test]
fn test_gate_off_is_an_explicit_null() {
    let off = ControlAudioPayload {
        gate_db: Some(None),
        ..Default::default()
    };
    assert_eq!(serde_json::to_string(&off).unwrap(), r#"{"gate_db":null}"#);
    let parsed: ControlAudioPayload = serde_json::from_str(r#"{"gate_db":null}"#).unwrap();
    assert_eq!(parsed.gate_db, Some(None));
    let untouched: ControlAudioPayload = serde_json::from_str("{}").unwrap();
    assert_eq!(untouched.gate_db, None);
}
