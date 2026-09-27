mod handshake;
#[cfg(test)]
mod tests;
mod types;

pub use types::{
    ActiveSession, HandshakeOutcome, PendingRequest, PENDING_TIMEOUT, SESSION_HOLD_DURATION,
};

use crate::config::{generate_random_hex, Config};
use crate::protocol::HelloPayload;
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

    pub fn handle_hello(&self, hello: &HelloPayload) -> HandshakeOutcome {
        let mut inner = self.inner.lock().unwrap();
        handshake::evaluate_hello(&mut inner, &self.next_request_id, hello)
    }

    pub fn wait_for_decision(&self, request_id: u64, timeout: Duration) -> HandshakeOutcome {
        let mut inner = self.inner.lock().unwrap();
        let deadline = Instant::now() + timeout;

        loop {
            if let Some(entry) = inner.pending_requests.get(&request_id) {
                if let Some(allow) = entry.decision {
                    let req = entry.request.clone();
                    inner.pending_requests.remove(&request_id);

                    if allow {
                        let session_token = generate_random_hex(32);
                        let now = Instant::now();
                        inner.active_session = Some(ActiveSession {
                            session_token: session_token.clone(),
                            device_id: req.device_id.clone(),
                            device_name: req.device_name.clone(),
                            current_level: req.level,
                            started_at: now,
                            last_frame_at: now,
                            transport_dropped_at: None,
                        });

                        inner.config.add_or_update_device(
                            req.device_id,
                            req.device_name,
                            req.token.clone(),
                            Some(req.level),
                        );
                        let _ = inner.config.save_to(&inner.config_path);

                        return HandshakeOutcome::Accept {
                            token: req.token,
                            resumed: false,
                            pc_id: inner.config.pc_id.clone(),
                            pc_name: inner.config.pc_name.clone(),
                            session_token,
                        };
                    } else {
                        return HandshakeOutcome::Reject {
                            reason: "denied".to_string(),
                        };
                    }
                }
            } else {
                return HandshakeOutcome::Reject {
                    reason: "denied".to_string(),
                };
            }

            let now = Instant::now();
            if now >= deadline {
                inner.pending_requests.remove(&request_id);
                return HandshakeOutcome::Reject {
                    reason: "timeout".to_string(),
                };
            }

            let remaining = deadline - now;
            inner = self.condvar.wait_timeout(inner, remaining).unwrap().0;
        }
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
