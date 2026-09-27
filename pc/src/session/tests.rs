use super::*;
use crate::protocol::PROTO_VERSION;
use std::thread;

fn create_test_manager() -> SessionManager {
    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("mikey_test_{}.toml", generate_random_hex(8)));
    SessionManager::new(temp_path)
}

#[test]
fn test_usb_new_device_silent_accept() {
    let sm = create_test_manager();
    let hello = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "phone-123".into(),
        device_name: "Pixel".into(),
        level: 1, // USB
        token: None,
        resume: None,
        caps: vec![],
    };

    match sm.handle_hello(&hello) {
        HandshakeOutcome::Accept {
            token,
            resumed,
            session_token,
            ..
        } => {
            assert!(!resumed);
            assert!(!token.is_empty());
            assert!(!session_token.is_empty());
        }
        other => panic!("expected Accept for USB, got {:?}", other),
    }

    assert!(sm.is_active());
}

#[test]
fn test_wifi_new_device_requires_approval() {
    let sm = create_test_manager();
    let hello = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "phone-456".into(),
        device_name: "Galaxy".into(),
        level: 4, // Wi-Fi
        token: None,
        resume: None,
        caps: vec![],
    };

    let req_id = match sm.handle_hello(&hello) {
        HandshakeOutcome::Pending { request_id } => request_id,
        other => panic!("expected Pending for Wi-Fi, got {:?}", other),
    };

    assert!(!sm.is_active());
    assert_eq!(sm.list_pending().len(), 1);

    // Spawn resolver thread
    let sm_clone = sm.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        assert!(sm_clone.resolve_pending(req_id, true));
    });

    match sm.wait_for_decision(req_id, Duration::from_secs(1)) {
        HandshakeOutcome::Accept { .. } => {}
        other => panic!("expected Accept after approval, got {:?}", other),
    }

    assert!(sm.is_active());
}

#[test]
fn test_one_active_busy_rejection() {
    let sm = create_test_manager();
    let dev1 = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "phone-1".into(),
        device_name: "Dev1".into(),
        level: 1,
        token: None,
        resume: None,
        caps: vec![],
    };
    let dev2 = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "phone-2".into(),
        device_name: "Dev2".into(),
        level: 1,
        token: None,
        resume: None,
        caps: vec![],
    };

    match sm.handle_hello(&dev1) {
        HandshakeOutcome::Accept { .. } => {}
        _ => panic!("dev1 failed"),
    }

    match sm.handle_hello(&dev2) {
        HandshakeOutcome::Pending { request_id } => {
            let pending = sm.list_pending();
            assert_eq!(pending.len(), 1);
            assert!(pending[0].is_switch);

            let sm_clone = sm.clone();
            thread::spawn(move || {
                sm_clone.resolve_pending(request_id, false);
            });

            match sm.wait_for_decision(request_id, Duration::from_secs(1)) {
                HandshakeOutcome::Reject { reason } => assert_eq!(reason, "denied"),
                _ => panic!("expected denied"),
            }
        }
        _ => panic!("expected pending switch"),
    }
}

#[test]
fn test_session_hold_and_handover() {
    let sm = create_test_manager();
    let hello_usb = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "phone-1".into(),
        device_name: "Dev1".into(),
        level: 1,
        token: None,
        resume: None,
        caps: vec![],
    };

    let token = match sm.handle_hello(&hello_usb) {
        HandshakeOutcome::Accept { token, .. } => token,
        _ => panic!("usb failed"),
    };

    sm.notify_transport_dropped();
    assert!(!sm.is_active()); // held

    let hello_wifi = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "phone-1".into(),
        device_name: "Dev1".into(),
        level: 4, // handover to Wi-Fi
        token: Some(token),
        resume: Some(true),
        caps: vec![],
    };

    match sm.handle_hello(&hello_wifi) {
        HandshakeOutcome::Accept { resumed, token, .. } => {
            assert!(resumed);
            assert!(!token.is_empty());
        }
        _ => panic!("handover failed"),
    }

    assert!(sm.is_active());
    assert_eq!(sm.current_level(), Some(4));
}
