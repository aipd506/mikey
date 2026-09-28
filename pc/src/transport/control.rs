//! What the phone tells the PC mid-session: its settings (CONTROL) and how it leaves (BYE).

use crate::audio::pipeline::JitterBuffer;
use crate::protocol::{ByePayload, ControlPayload, Frame, FrameType};
use crate::session::SessionManager;
use crate::video::VideoPipeline;

/// Applies the phone's settings. The phone owns them and the PC follows (wire-protocol.md).
pub fn apply_phone_control(
    payload: &[u8],
    jitter_buffer: &JitterBuffer,
    video: Option<&VideoPipeline>,
    session_manager: &SessionManager,
) {
    let Ok(ctrl) = serde_json::from_slice::<ControlPayload>(payload) else {
        return;
    };
    if let Some(audio) = ctrl.audio {
        if let Some(ns) = audio.ns {
            jitter_buffer.set_ns_enabled(ns);
        }
        if let Some(strength) = audio.ns_strength {
            jitter_buffer.set_ns_strength((strength * 100.0) as u32);
        }
        if let Some(aec) = audio.aec {
            jitter_buffer.set_aec_enabled(aec);
        }
        if let Some(gate_db) = audio.gate_db {
            jitter_buffer.set_gate_db(gate_db);
        }
        if let Some(muted) = audio.muted {
            session_manager.set_phone_muted(muted);
        }
    }
    if let (Some(video), Some(on)) = (video, ctrl.video.and_then(|v| v.on)) {
        video.set_camera_active(on);
    }
}

/// How a phone's connection ended, which decides what happens to its session.
#[derive(Debug, PartialEq, Eq)]
pub enum Ending {
    /// The link failed: hold the session so the phone can come back on any level.
    Dropped,
    /// The phone's user stopped: the session ends now.
    Stopped,
    /// The phone moved to a better link, which already carries the session.
    Switched,
    /// The PC's user disconnected the phone, or another phone took over.
    Disconnected,
}

/// Reads the reason in the phone's BYE.
pub fn ending_for_bye(payload: &[u8]) -> Ending {
    match serde_json::from_slice::<ByePayload>(payload) {
        Ok(bye) if bye.reason == "stop" => Ending::Stopped,
        Ok(bye) if bye.reason == "switch" => Ending::Switched,
        _ => Ending::Dropped,
    }
}

pub fn finish_session(session_manager: &SessionManager, ending: &Ending) {
    match ending {
        Ending::Dropped => session_manager.notify_transport_dropped(),
        Ending::Stopped => session_manager.close_session("phone stopped"),
        Ending::Switched | Ending::Disconnected => {}
    }
}

/// Tells the phone its session is over: it stops instead of reconnecting.
pub fn disconnect_frame() -> Frame {
    let bye = ByePayload {
        reason: "disconnect".to_string(),
    };
    Frame::new(FrameType::Bye, serde_json::to_vec(&bye).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bye_reason_decides_what_happens_to_the_session() {
        assert_eq!(ending_for_bye(br#"{"reason":"stop"}"#), Ending::Stopped);
        assert_eq!(ending_for_bye(br#"{"reason":"switch"}"#), Ending::Switched);
        assert_eq!(ending_for_bye(br#"{"reason":"later"}"#), Ending::Dropped);
        assert_eq!(ending_for_bye(b"not json"), Ending::Dropped);
    }

    #[test]
    fn the_disconnect_bye_carries_its_reason() {
        let frame = disconnect_frame();
        assert_eq!(frame.frame_type, FrameType::Bye);
        assert_eq!(frame.payload, br#"{"reason":"disconnect"}"#);
    }

    #[test]
    fn the_phone_settings_drive_the_processing() {
        let jb = JitterBuffer::new();
        let sm = SessionManager::new(std::env::temp_dir().join("mikey-control-test.toml"));
        let full =
            br#"{"audio":{"ns":false,"ns_strength":0.8,"aec":false,"gate_db":-40.0,"muted":true}}"#;
        apply_phone_control(full, &jb, None, &sm);
        assert!(!jb.is_ns_enabled());
        assert_eq!(jb.get_ns_strength(), 80);
        assert!(!jb.is_aec_enabled());
        assert_eq!(jb.gate_db(), Some(-40.0));
        assert!(sm.is_phone_muted());

        // A partial update changes only what it names; an explicit null switches the gate off.
        apply_phone_control(br#"{"audio":{"muted":false}}"#, &jb, None, &sm);
        assert_eq!(jb.gate_db(), Some(-40.0));
        assert!(!sm.is_phone_muted());
        apply_phone_control(br#"{"audio":{"gate_db":null}}"#, &jb, None, &sm);
        assert_eq!(jb.gate_db(), None);
    }
}
