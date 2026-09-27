mod client;
#[cfg(windows)]
mod server;

pub use client::handle_bt_client;

use crate::audio::pipeline::JitterBuffer;
use crate::session::SessionManager;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

pub const MIKEY_BT_SERVICE_UUID: &str = "6d696b65-7900-4000-8000-00805f9b34fb";

#[cfg(windows)]
pub fn start_bt_listener(
    session_manager: SessionManager,
    jitter_buffer: Arc<JitterBuffer>,
    running: Arc<AtomicBool>,
) -> Option<thread::JoinHandle<()>> {
    let handle = thread::spawn(move || {
        if let Err(e) = server::run_windows_rfcomm_listener(session_manager, jitter_buffer, running)
        {
            eprintln!("[bt] Bluetooth RFCOMM listener stopped: {}", e);
        }
    });
    Some(handle)
}

#[cfg(not(windows))]
pub fn start_bt_listener(
    _session_manager: SessionManager,
    _jitter_buffer: Arc<JitterBuffer>,
    _running: Arc<AtomicBool>,
) -> Option<thread::JoinHandle<()>> {
    println!("[bt] Bluetooth RFCOMM on Linux requires BlueZ runtime (stubbed for non-Windows)");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bt_service_uuid_format() {
        assert_eq!(MIKEY_BT_SERVICE_UUID.len(), 36);
        assert!(MIKEY_BT_SERVICE_UUID.starts_with("6d696b65"));
    }
}
