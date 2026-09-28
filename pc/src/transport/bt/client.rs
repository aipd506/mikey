use crate::audio::opus::OpusDecoderWrapper;
use crate::audio::pipeline::JitterBuffer;
use crate::protocol::{
    read_frame, write_frame, Frame, FrameType, MediaHeader, RejectPayload, WelcomePayload,
    CODEC_OPUS, CODEC_PCM, MEDIA_HEADER_LEN,
};
use crate::session::{HandshakeOutcome, SessionManager, PENDING_TIMEOUT};
use std::io::{Read, Write};
use std::sync::Arc;

pub fn handle_bt_client<S: Read + Write>(
    mut stream: S,
    session_manager: SessionManager,
    jitter_buffer: Arc<JitterBuffer>,
) {
    let frame = match read_frame(&mut stream) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[bt] Failed to read initial frame: {}", e);
            return;
        }
    };

    if frame.frame_type != FrameType::Hello {
        return;
    }

    let hello: crate::protocol::HelloPayload = match serde_json::from_slice(&frame.payload) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("[bt] Invalid HELLO payload: {}", e);
            return;
        }
    };

    let outcome = session_manager.handle_hello(&hello);
    let final_outcome = match outcome {
        HandshakeOutcome::Pending { request_id } => {
            let pending_frame = Frame::empty(FrameType::Pending);
            let _ = write_frame(&mut stream, &pending_frame);
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
                pc_caps: vec!["pcm".to_string(), "opus".to_string()],
            };
            let welcome_bytes = match serde_json::to_vec(&welcome) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[bt] Failed to serialize WELCOME: {}", e);
                    return;
                }
            };
            if write_frame(&mut stream, &Frame::new(FrameType::Welcome, welcome_bytes)).is_err() {
                return;
            }
        }
        HandshakeOutcome::Reject { reason } => {
            let reject = RejectPayload {
                reason: reason.clone(),
            };
            if let Ok(bytes) = serde_json::to_vec(&reject) {
                let _ = write_frame(&mut stream, &Frame::new(FrameType::Reject, bytes));
            }
            return;
        }
        HandshakeOutcome::Pending { .. } => return,
    }

    println!("[bt-connected] {} via Bluetooth RFCOMM", hello.device_name);
    jitter_buffer.set_level(4); // Level 4: Bluetooth

    let mut opus_decoder = OpusDecoderWrapper::new().ok();
    let mut sample_buf = Vec::with_capacity(960);

    while let Ok(frame) = read_frame(&mut stream) {
        session_manager.record_frame_received();

        match frame.frame_type {
            FrameType::Audio => {
                if frame.payload.len() >= MEDIA_HEADER_LEN {
                    if let Ok(header) = MediaHeader::parse(&frame.payload[0..MEDIA_HEADER_LEN]) {
                        jitter_buffer.record_arrival(header.capture_ts);
                        let payload = &frame.payload[MEDIA_HEADER_LEN..];

                        match header.codec {
                            CODEC_PCM => {
                                let samples: Vec<i16> = payload
                                    .as_chunks::<2>()
                                    .0
                                    .iter()
                                    .map(|&c| i16::from_le_bytes(c))
                                    .collect();
                                jitter_buffer.push_samples(&samples);
                            }
                            CODEC_OPUS => {
                                if let Some(ref mut dec) = opus_decoder {
                                    sample_buf.resize(960, 0);
                                    if let Ok(count) = dec.decode(payload, &mut sample_buf) {
                                        jitter_buffer.push_samples(&sample_buf[..count]);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            FrameType::Heartbeat => {
                let _ = write_frame(
                    &mut stream,
                    &Frame::new(FrameType::Heartbeat, frame.payload),
                );
            }
            FrameType::Control => {
                if let Ok(ctrl) =
                    serde_json::from_slice::<crate::protocol::ControlPayload>(&frame.payload)
                {
                    if let Some(ref audio) = ctrl.audio {
                        if let Some(aec) = audio.aec {
                            jitter_buffer.set_aec_enabled(aec);
                        }
                        if let Some(strength) = audio.ns_strength {
                            jitter_buffer.set_ns_strength((strength * 100.0) as u32);
                        }
                    }
                }
            }
            FrameType::Bye => break,
            _ => {}
        }

        for ctrl in session_manager.take_pending_controls() {
            if let Ok(bytes) = serde_json::to_vec(&ctrl) {
                let _ = write_frame(&mut stream, &Frame::new(FrameType::Control, bytes));
            }
        }
    }

    session_manager.notify_transport_dropped();
    println!(
        "[bt-dropped] RFCOMM connection ended for {}",
        hello.device_name
    );
}
