use crate::protocol::{
    read_frame, write_frame, Frame, FrameType, HelloPayload, RejectPayload, WelcomePayload,
    PORT_TCP, PROTO_VERSION,
};
use crate::session::{HandshakeOutcome, SessionManager, PENDING_TIMEOUT};
use std::io::{self, Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(6);

pub fn bind_listener() -> io::Result<TcpListener> {
    let addr = format!("0.0.0.0:{}", PORT_TCP);
    let listener = TcpListener::bind(&addr)?;
    Ok(listener)
}

pub fn configure_stream(stream: &TcpStream) -> io::Result<()> {
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(SOCKET_TIMEOUT))?;
    stream.set_write_timeout(Some(SOCKET_TIMEOUT))?;
    stream.set_nonblocking(false)?;
    Ok(())
}

pub fn perform_handshake(
    stream: &mut TcpStream,
    session_manager: &SessionManager,
) -> io::Result<HelloPayload> {
    let frame = read_frame(stream)?;
    if frame.frame_type != FrameType::Hello {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "expected HELLO frame (0x00), got 0x{:02x}",
                frame.frame_type.to_u8()
            ),
        ));
    }

    let hello: HelloPayload = serde_json::from_slice(&frame.payload)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("invalid HELLO JSON: {}", e)))?;

    if hello.proto != PROTO_VERSION {
        let reject = RejectPayload {
            reason: "version".to_string(),
        };
        let reject_bytes = serde_json::to_vec(&reject)
            .map_err(|e| Error::other(format!("failed to serialize REJECT: {}", e)))?;
        let _ = write_frame(stream, &Frame::new(FrameType::Reject, reject_bytes));
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "protocol version mismatch: phone has {}, pc has {}",
                hello.proto, PROTO_VERSION
            ),
        ));
    }

    let outcome = session_manager.handle_hello(&hello);
    let final_outcome = match outcome {
        HandshakeOutcome::Pending { request_id } => {
            let pending_frame = Frame::empty(FrameType::Pending);
            write_frame(stream, &pending_frame)?;
            session_manager.wait_for_decision(request_id, PENDING_TIMEOUT)
        }
        other => other,
    };

    match final_outcome {
        HandshakeOutcome::Accept {
            token,
            resumed,
            pc_id,
            pc_name,
            ..
        } => {
            let welcome = WelcomePayload {
                pc_id,
                pc_name,
                token,
                resumed,
                pc_caps: vec!["pcm".to_string(), "opus".to_string(), "vcam".to_string()],
            };
            let welcome_bytes = serde_json::to_vec(&welcome)
                .map_err(|e| Error::other(format!("failed to serialize WELCOME: {}", e)))?;
            write_frame(stream, &Frame::new(FrameType::Welcome, welcome_bytes))?;
            Ok(hello)
        }
        HandshakeOutcome::Reject { reason } => {
            let reject = RejectPayload {
                reason: reason.clone(),
            };
            let reject_bytes = serde_json::to_vec(&reject)
                .map_err(|e| Error::other(format!("failed to serialize REJECT: {}", e)))?;
            let _ = write_frame(stream, &Frame::new(FrameType::Reject, reject_bytes));
            Err(Error::new(ErrorKind::PermissionDenied, reason))
        }
        HandshakeOutcome::Pending { .. } => {
            Err(Error::new(ErrorKind::TimedOut, "pending timed out"))
        }
    }
}
