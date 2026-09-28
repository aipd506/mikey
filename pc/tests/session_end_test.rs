//! How a session ends: the phone stopping, the link dropping, and the PC's user disconnecting.

use mikey::audio::pipeline::JitterBuffer;
use mikey::config::generate_random_hex;
use mikey::protocol::{read_frame, write_frame, Frame, FrameType, HelloPayload, PROTO_VERSION};
use mikey::session::SessionManager;
use mikey::transport::tcp::{configure_stream, handle_client};
use mikey::video::VideoPipeline;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// A PC with one connected, accepted phone.
fn connected_phone() -> (SessionManager, TcpStream, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let config = std::env::temp_dir().join(format!("mikey_end_{}.toml", generate_random_hex(8)));
    let sm = SessionManager::new(config);
    let server_sm = sm.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let jb = Arc::new(JitterBuffer::new());
        handle_client(stream, jb, Arc::new(VideoPipeline::new()), server_sm);
    });

    let mut phone = TcpStream::connect(addr).expect("connect");
    configure_stream(&phone).expect("configure");
    let hello = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "phone-end".into(),
        device_name: "Pixel".into(),
        level: 1,
        token: None,
        resume: None,
        caps: vec!["audio".into()],
    };
    let hello = Frame::new(FrameType::Hello, serde_json::to_vec(&hello).unwrap());
    write_frame(&mut phone, &hello).expect("hello");
    let welcome = read_frame(&mut phone).expect("welcome");
    assert_eq!(welcome.frame_type, FrameType::Welcome);
    (sm, phone, server)
}

fn send(phone: &mut TcpStream, frame_type: FrameType, payload: &[u8]) {
    write_frame(phone, &Frame::new(frame_type, payload.to_vec())).expect("send");
}

#[test]
fn a_phone_that_says_stop_ends_its_session_at_once() {
    let (sm, mut phone, server) = connected_phone();
    send(&mut phone, FrameType::Bye, br#"{"reason":"stop"}"#);
    server.join().unwrap();
    assert!(sm.active_session().is_none());
}

#[test]
fn a_dropped_link_holds_the_session_for_the_phone_to_come_back() {
    let (sm, phone, server) = connected_phone();
    drop(phone);
    server.join().unwrap();
    assert!(sm.active_session().expect("held").is_held());
}

#[test]
fn disconnecting_on_the_pc_tells_the_phone_to_stop_and_closes_the_link() {
    let (sm, mut phone, server) = connected_phone();
    sm.close_session("user disconnected");
    send(&mut phone, FrameType::Heartbeat, &[0; 8]);

    let bye = loop {
        let frame = read_frame(&mut phone).expect("the PC answers before closing");
        if frame.frame_type == FrameType::Bye {
            break frame;
        }
    };
    assert_eq!(bye.payload, br#"{"reason":"disconnect"}"#);
    assert!(read_frame(&mut phone).is_err(), "the PC closed the link");
    server.join().unwrap();
    assert!(sm.active_session().is_none());
}
