use mikey::audio::pipeline::JitterBuffer;
use mikey::config::generate_random_hex;
use mikey::protocol::{
    read_frame, write_frame, Frame, FrameType, HelloPayload, MediaHeader, WelcomePayload,
    CODEC_PCM, PROTO_VERSION,
};
use mikey::session::SessionManager;
use mikey::transport::tcp::{configure_stream, handle_client};
use mikey::video::VideoPipeline;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

#[test]
fn test_end_to_end_streaming_and_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");

    let jb = Arc::new(JitterBuffer::new());
    let jb_clone = Arc::clone(&jb);
    let vp = Arc::new(VideoPipeline::new());
    let vp_clone = Arc::clone(&vp);

    let mut temp_cfg = std::env::temp_dir();
    temp_cfg.push(format!("mikey_it_{}.toml", generate_random_hex(8)));
    let sm = SessionManager::new(temp_cfg);
    let sm_clone = sm.clone();
    let expected_pc_id = sm.config().pc_id.clone();

    let server_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept connection");
        handle_client(stream, jb_clone, vp_clone, sm_clone);
    });

    // Client connection
    let mut client = TcpStream::connect(addr).expect("connect to server");
    configure_stream(&client).expect("configure client stream");

    // 1. Send HELLO
    let hello = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "test-device-uuid".to_string(),
        device_name: "Pixel 7".to_string(),
        level: 1,
        token: None,
        resume: None,
        caps: vec!["pcm".to_string()],
    };
    let hello_bytes = serde_json::to_vec(&hello).unwrap();
    let hello_frame = Frame::new(FrameType::Hello, hello_bytes);
    write_frame(&mut client, &hello_frame).expect("send hello");

    // 2. Read WELCOME
    let welcome_frame = read_frame(&mut client).expect("read welcome");
    assert_eq!(welcome_frame.frame_type, FrameType::Welcome);
    let welcome: WelcomePayload = serde_json::from_slice(&welcome_frame.payload).unwrap();
    assert_eq!(welcome.pc_id, expected_pc_id);
    assert!(!welcome.token.is_empty());

    // 3. Send AUDIO frame (10 ms PCM = 480 samples = 960 bytes)
    let header = MediaHeader {
        seq: 1,
        capture_ts: 1_700_000_000,
        codec: CODEC_PCM,
        reserved: 0,
    };
    let mut audio_payload = header.to_vec();
    let pcm_samples = vec![1234i16; 480];
    for s in &pcm_samples {
        audio_payload.extend_from_slice(&s.to_le_bytes());
    }
    let audio_frame = Frame::new(FrameType::Audio, audio_payload);
    write_frame(&mut client, &audio_frame).expect("send audio");

    // 4. Send HEARTBEAT
    let ts: u64 = 987_654_321;
    let heartbeat_frame = Frame::new(FrameType::Heartbeat, ts.to_be_bytes().to_vec());
    write_frame(&mut client, &heartbeat_frame).expect("send heartbeat");

    // 5. Read echoed HEARTBEAT
    let echoed_hb = read_frame(&mut client).expect("read echoed heartbeat");
    assert_eq!(echoed_hb.frame_type, FrameType::Heartbeat);
    assert_eq!(echoed_hb.payload, ts.to_be_bytes());

    // 6. Send BYE
    let bye_frame = Frame::new(FrameType::Bye, b"{\"reason\":\"user_stop\"}".to_vec());
    write_frame(&mut client, &bye_frame).expect("send bye");

    server_thread
        .join()
        .expect("server thread terminates cleanly");
}

#[test]
fn test_end_to_end_handover_with_session_hold() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");

    let jb = Arc::new(JitterBuffer::new());
    let jb_clone = Arc::clone(&jb);
    let vp = Arc::new(VideoPipeline::new());
    let vp_clone = Arc::clone(&vp);

    let mut temp_cfg = std::env::temp_dir();
    temp_cfg.push(format!("mikey_it_ho_{}.toml", generate_random_hex(8)));
    let sm = SessionManager::new(temp_cfg);
    let sm_clone = sm.clone();

    // Spawn server accept loop for two connections (Level 1 then Level 3)
    let server_thread = thread::spawn(move || {
        for _ in 0..2 {
            let (stream, _) = listener.accept().expect("accept connection");
            let jb_inst = Arc::clone(&jb_clone);
            let vp_inst = Arc::clone(&vp_clone);
            let sm_inst = sm_clone.clone();
            handle_client(stream, jb_inst, vp_inst, sm_inst);
        }
    });

    // 1. Initial connection via Level 1 (USB)
    let mut client1 = TcpStream::connect(addr).expect("connect client 1");
    configure_stream(&client1).expect("configure stream");

    let hello1 = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "handover-phone-id".to_string(),
        device_name: "Pixel 7".to_string(),
        level: 1,
        token: None,
        resume: None,
        caps: vec!["pcm".to_string()],
    };
    write_frame(
        &mut client1,
        &Frame::new(FrameType::Hello, serde_json::to_vec(&hello1).unwrap()),
    )
    .expect("send hello 1");

    let welcome_frame1 = read_frame(&mut client1).expect("read welcome 1");
    let welcome1: WelcomePayload = serde_json::from_slice(&welcome_frame1.payload).unwrap();
    assert!(!welcome1.resumed);
    let token = welcome1.token;

    // Simulate abrupt cable disconnect (drop socket without sending BYE)
    drop(client1);
    thread::sleep(std::time::Duration::from_millis(100));

    // Verify session is held on PC
    let session = sm.active_session().expect("session exists");
    assert!(session.is_held());
    assert_eq!(session.current_level, 1);

    // 2. Reconnect via Level 3 (Wi-Fi) within 30 seconds
    let mut client2 = TcpStream::connect(addr).expect("connect client 2");
    configure_stream(&client2).expect("configure stream");

    let hello2 = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "handover-phone-id".to_string(),
        device_name: "Pixel 7".to_string(),
        level: 3,
        token: Some(token),
        resume: Some(true),
        caps: vec!["pcm".to_string()],
    };
    write_frame(
        &mut client2,
        &Frame::new(FrameType::Hello, serde_json::to_vec(&hello2).unwrap()),
    )
    .expect("send hello 2");

    let welcome_frame2 = read_frame(&mut client2).expect("read welcome 2");
    let welcome2: WelcomePayload = serde_json::from_slice(&welcome_frame2.payload).unwrap();
    // Handover succeeded: session resumed!
    assert!(welcome2.resumed);

    let updated_session = sm.active_session().expect("resumed session");
    assert_eq!(updated_session.current_level, 3);
    assert!(!updated_session.is_held());

    // Clean shutdown
    let _ = write_frame(
        &mut client2,
        &Frame::new(FrameType::Bye, b"{\"reason\":\"stop\"}".to_vec()),
    );
    server_thread.join().expect("server joins cleanly");
}
