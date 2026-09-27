use super::types::*;
use crate::config::generate_random_hex;
use crate::protocol::{HelloPayload, PROTO_VERSION};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub(crate) fn evaluate_hello(
    inner: &mut SessionInner,
    next_request_id: &AtomicU64,
    hello: &HelloPayload,
) -> HandshakeOutcome {
    let now = Instant::now();

    if hello.proto != PROTO_VERSION {
        return HandshakeOutcome::Reject {
            reason: "version".to_string(),
        };
    }

    if let Some(ref session) = inner.active_session {
        if session.is_expired(now) {
            inner.active_session = None;
        }
    }

    let is_same_device_as_active = inner
        .active_session
        .as_ref()
        .map(|s| s.device_id == hello.device_id)
        .unwrap_or(false);

    if inner.active_session.is_some() && !is_same_device_as_active {
        let request_id = next_request_id.fetch_add(1, Ordering::Relaxed);
        let token = generate_random_hex(32);
        let req = PendingRequest {
            request_id,
            device_id: hello.device_id.clone(),
            device_name: hello.device_name.clone(),
            level: hello.level,
            token,
            is_switch: true,
            requested_at: now,
        };
        inner.pending_requests.insert(
            request_id,
            PendingEntry {
                request: req,
                decision: None,
            },
        );
        return HandshakeOutcome::Pending { request_id };
    }

    if is_same_device_as_active {
        if let Some(ref mut session) = inner.active_session {
            session.current_level = hello.level;
            session.last_frame_at = now;
            session.transport_dropped_at = None;

            let session_token = session.session_token.clone();
            let token = inner
                .config
                .trusted_devices
                .get(&hello.device_id)
                .map(|d| d.token.clone())
                .unwrap_or_else(|| generate_random_hex(32));

            inner.config.add_or_update_device(
                hello.device_id.clone(),
                hello.device_name.clone(),
                token.clone(),
                Some(hello.level),
            );
            let _ = inner.config.save_to(&inner.config_path);

            return HandshakeOutcome::Accept {
                token,
                resumed: true,
                pc_id: inner.config.pc_id.clone(),
                pc_name: inner.config.pc_name.clone(),
                session_token,
            };
        }
    }

    let is_known = if let Some(ref token) = hello.token {
        inner.config.is_trusted(&hello.device_id, token)
    } else {
        false
    };

    let ask_always = inner.config.ask_before_joining;
    let needs_prompt = if ask_always {
        true
    } else if is_known {
        false
    } else {
        match hello.level {
            1 | 2 => false,
            3 => false,
            4 => !inner.config.trust_wifi_automatically,
            _ => true,
        }
    };

    if needs_prompt {
        let request_id = next_request_id.fetch_add(1, Ordering::Relaxed);
        let token = generate_random_hex(32);
        let req = PendingRequest {
            request_id,
            device_id: hello.device_id.clone(),
            device_name: hello.device_name.clone(),
            level: hello.level,
            token,
            is_switch: false,
            requested_at: now,
        };
        inner.pending_requests.insert(
            request_id,
            PendingEntry {
                request: req,
                decision: None,
            },
        );
        return HandshakeOutcome::Pending { request_id };
    }

    let token = if is_known {
        hello.token.clone().unwrap()
    } else {
        generate_random_hex(32)
    };

    inner.config.add_or_update_device(
        hello.device_id.clone(),
        hello.device_name.clone(),
        token.clone(),
        Some(hello.level),
    );
    let _ = inner.config.save_to(&inner.config_path);

    let session_token = generate_random_hex(32);
    inner.active_session = Some(ActiveSession {
        session_token: session_token.clone(),
        device_id: hello.device_id.clone(),
        device_name: hello.device_name.clone(),
        current_level: hello.level,
        started_at: now,
        last_frame_at: now,
        transport_dropped_at: None,
    });

    HandshakeOutcome::Accept {
        token,
        resumed: false,
        pc_id: inner.config.pc_id.clone(),
        pc_name: inner.config.pc_name.clone(),
        session_token,
    }
}
