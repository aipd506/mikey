use super::types::*;
use crate::config::generate_random_hex;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub(crate) fn wait_for_decision_inner(
    inner_mutex: &Arc<Mutex<SessionInner>>,
    condvar: &Arc<Condvar>,
    request_id: u64,
    timeout: Duration,
) -> HandshakeOutcome {
    let mut inner = inner_mutex.lock().unwrap();
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
        inner = condvar.wait_timeout(inner, remaining).unwrap().0;
    }
}
