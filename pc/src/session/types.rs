use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub const SESSION_HOLD_DURATION: Duration = Duration::from_secs(30);
pub const PENDING_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSession {
    pub session_token: String,
    pub device_id: String,
    pub device_name: String,
    pub current_level: u8,
    pub started_at: Instant,
    pub last_frame_at: Instant,
    pub transport_dropped_at: Option<Instant>,
}

impl ActiveSession {
    pub fn is_held(&self) -> bool {
        self.transport_dropped_at.is_some()
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        if let Some(dropped_at) = self.transport_dropped_at {
            now.duration_since(dropped_at) >= SESSION_HOLD_DURATION
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub request_id: u64,
    pub device_id: String,
    pub device_name: String,
    pub level: u8,
    pub token: String,
    pub is_switch: bool,
    pub requested_at: Instant,
}

#[derive(Debug, Clone)]
pub enum HandshakeOutcome {
    Accept {
        token: String,
        resumed: bool,
        pc_id: String,
        pc_name: String,
        session_token: String,
    },
    Pending {
        request_id: u64,
    },
    Reject {
        reason: String,
    },
}

pub(crate) struct PendingEntry {
    pub(crate) request: PendingRequest,
    pub(crate) decision: Option<bool>,
}

pub(crate) struct SessionInner {
    pub(crate) config: Config,
    pub(crate) config_path: PathBuf,
    pub(crate) active_session: Option<ActiveSession>,
    pub(crate) pending_requests: HashMap<u64, PendingEntry>,
}
