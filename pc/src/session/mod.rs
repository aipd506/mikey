mod decision;
mod handshake;
#[cfg(test)]
mod tests;
mod types;

pub use types::{
    ActiveSession, HandshakeOutcome, PendingRequest, PENDING_TIMEOUT, SESSION_HOLD_DURATION,
};

use crate::config::Config;
use crate::protocol::{ControlPayload, HelloPayload};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use types::*;

#[derive(Clone)]
pub struct SessionManager {
    inner: Arc<Mutex<SessionInner>>,
    condvar: Arc<Condvar>,
    next_request_id: Arc<AtomicU64>,
}

impl SessionManager {
    pub fn new(config_path: PathBuf) -> Self {
        let config = Config::load_or_default(&config_path);
        Self {
            inner: Arc::new(Mutex::new(SessionInner {
                config,
                config_path,
                active_session: None,
                pending_requests: std::collections::HashMap::new(),
                pending_controls: Vec::new(),
                phone_muted: false,
            })),
            condvar: Arc::new(Condvar::new()),
            next_request_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn config(&self) -> Config {
        let inner = self.inner.lock().unwrap();
        inner.config.clone()
    }

    pub fn active_session(&self) -> Option<ActiveSession> {
        let inner = self.inner.lock().unwrap();
        inner.active_session.clone()
    }

    pub fn is_active(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner
            .active_session
            .as_ref()
            .map(|s| !s.is_held())
            .unwrap_or(false)
    }

    pub fn current_level(&self) -> Option<u8> {
        let inner = self.inner.lock().unwrap();
        inner.active_session.as_ref().map(|s| s.current_level)
    }

    /// False once this phone's session is gone: the PC's user disconnected it, or another
    /// phone took over. Its connection then tells it to stop and ends.
    pub fn is_active_device(&self, device_id: &str) -> bool {
        let inner = self.inner.lock().unwrap();
        inner
            .active_session
            .as_ref()
            .is_some_and(|s| s.device_id == device_id)
    }

    /// The phone's soft mute, as it last told us or as the flyout set it.
    pub fn set_phone_muted(&self, muted: bool) {
        self.inner.lock().unwrap().phone_muted = muted;
    }

    pub fn is_phone_muted(&self) -> bool {
        self.inner.lock().unwrap().phone_muted
    }

    pub fn handle_hello(&self, hello: &HelloPayload) -> HandshakeOutcome {
        let mut inner = self.inner.lock().unwrap();
        handshake::evaluate_hello(&mut inner, &self.next_request_id, hello)
    }

    pub fn wait_for_decision(&self, request_id: u64, timeout: Duration) -> HandshakeOutcome {
        decision::wait_for_decision_inner(&self.inner, &self.condvar, request_id, timeout)
    }

    pub fn queue_control(&self, payload: ControlPayload) {
        let mut inner = self.inner.lock().unwrap();
        if inner.active_session.is_some() {
            inner.pending_controls.push(payload);
        }
    }

    pub fn take_pending_controls(&self) -> Vec<ControlPayload> {
        let mut inner = self.inner.lock().unwrap();
        std::mem::take(&mut inner.pending_controls)
    }

    pub fn resolve_pending(&self, request_id: u64, allow: bool) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.pending_requests.get_mut(&request_id) {
            entry.decision = Some(allow);
            self.condvar.notify_all();
            true
        } else {
            false
        }
    }

    pub fn list_pending(&self) -> Vec<PendingRequest> {
        let inner = self.inner.lock().unwrap();
        inner
            .pending_requests
            .values()
            .map(|e| e.request.clone())
            .collect()
    }

    pub fn record_frame_received(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(ref mut session) = inner.active_session {
            session.last_frame_at = Instant::now();
            session.transport_dropped_at = None;
        }
    }

    pub fn notify_transport_dropped(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(ref mut session) = inner.active_session {
            if session.transport_dropped_at.is_none() {
                session.transport_dropped_at = Some(Instant::now());
            }
        }
    }

    pub fn close_session(&self, _reason: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.active_session = None;
        inner.phone_muted = false;
    }

    pub fn forget_device(&self, device_id: &str) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let changed = inner.config.forget_device(device_id);
        if changed {
            let _ = inner.config.save_to(&inner.config_path);
        }
        changed
    }
}
