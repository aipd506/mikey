mod handshake;

pub use handshake::{bind_listener, configure_stream, perform_handshake, SOCKET_TIMEOUT};

use crate::audio::opus::OpusDecoderWrapper;
use crate::audio::pipeline::JitterBuffer;
use crate::protocol::{
    read_frame, write_frame, Frame, FrameType, MediaHeader, CODEC_JPEG, CODEC_OPUS, CODEC_PCM,
    MEDIA_HEADER_LEN,
};
use crate::session::SessionManager;
use crate::video::VideoPipeline;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn handle_client(
    mut stream: TcpStream,
    jitter_buffer: Arc<JitterBuffer>,
    video_pipeline: Arc<VideoPipeline>,
    session_manager: SessionManager,
) {
    if let Err(e) = configure_stream(&stream) {
        eprintln!("[tcp] Failed to configure socket: {}", e);
        return;
    }

    let hello = match perform_handshake(&mut stream, &session_manager) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("[handshake] Handshake rejected or failed: {}", e);
            return;
        }
    };

    println!("[connected] {} via L{}", hello.device_name, hello.level);
    jitter_buffer.set_level(hello.level);

    let mut opus_decoder = OpusDecoderWrapper::new().ok();
    let mut sample_buf = Vec::with_capacity(960);

    while let Ok(frame) = read_frame(&mut stream) {
        session_manager.record_frame_received();

        match frame.frame_type {
            FrameType::Audio => {
                if frame.payload.len() >= MEDIA_HEADER_LEN {
                    let header = match MediaHeader::parse(&frame.payload[0..MEDIA_HEADER_LEN]) {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("[tcp] Failed to parse audio media header: {}", e);
                            continue;
                        }
                    };
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
                                if let Ok(decoded_count) = dec.decode(payload, &mut sample_buf) {
                                    jitter_buffer.push_samples(&sample_buf[..decoded_count]);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            FrameType::Video => {
                if frame.payload.len() >= MEDIA_HEADER_LEN {
                    let header = match MediaHeader::parse(&frame.payload[0..MEDIA_HEADER_LEN]) {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("[tcp] Failed to parse video media header: {}", e);
                            continue;
                        }
                    };
                    let payload = &frame.payload[MEDIA_HEADER_LEN..];
                    if header.codec == CODEC_JPEG {
                        video_pipeline.push_jpeg_frame(
                            payload.to_vec(),
                            header.seq,
                            header.capture_ts,
                        );
                    }
                }
            }
            FrameType::Heartbeat => {
                let _ = write_frame(
                    &mut stream,
                    &Frame::new(FrameType::Heartbeat, frame.payload),
                );
            }
            FrameType::Bye => {
                println!("[disconnect] Phone sent BYE frame");
                break;
            }
            _ => {}
        }
    }

    session_manager.notify_transport_dropped();
    println!("[dropped] TCP connection ended for {}", hello.device_name);
}

pub fn start_tcp_listener(
    listener: TcpListener,
    jitter_buffer: Arc<JitterBuffer>,
    video_pipeline: Arc<VideoPipeline>,
    session_manager: SessionManager,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let jb = Arc::clone(&jitter_buffer);
                    let vp = Arc::clone(&video_pipeline);
                    let sm = session_manager.clone();
                    thread::spawn(move || {
                        handle_client(stream, jb, vp, sm);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    if running.load(Ordering::Relaxed) {
                        eprintln!("[tcp] Accept error: {}", e);
                    }
                    break;
                }
            }
        }
    })
}
