use super::find_output_device;
use crate::audio::pipeline::JitterBuffer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::io::{self, Error};
use std::sync::Arc;

/// Opens the phone's audio output (the virtual mic, else the default speakers) and the speaker
/// capture that echo cancellation compares against. The streams stop when dropped.
pub fn start_output(jitter_buffer: &Arc<JitterBuffer>) -> Vec<Stream> {
    let mut streams = Vec::new();
    match find_output_device() {
        Ok((device, name, is_vb_cable)) => {
            if is_vb_cable {
                println!("[audio] Using virtual mic device: {}", name);
            } else {
                println!(
                    "[audio] No virtual mic found. Using fallback output: {}",
                    name
                );
            }
            match start_audio_stream(&device, Arc::clone(jitter_buffer)) {
                Ok(stream) => streams.push(stream),
                Err(e) => eprintln!("[audio] Failed to start audio playback stream: {}", e),
            }
        }
        Err(e) => eprintln!("[audio] Error finding output device: {}", e),
    }
    match start_loopback_stream(Arc::clone(jitter_buffer)) {
        Ok(stream) => {
            println!(
                "[audio] AEC loopback reference stream active (speaker sound cancellation enabled)"
            );
            streams.push(stream);
        }
        Err(e) => eprintln!("[audio] Note: Loopback stream not available: {}", e),
    }
    streams
}

pub fn start_audio_stream(device: &Device, jitter_buffer: Arc<JitterBuffer>) -> io::Result<Stream> {
    let supported_config = device
        .default_output_config()
        .map_err(|e| Error::other(format!("failed to get default audio config: {}", e)))?;

    let channels = supported_config.channels();
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();
    // Shared-mode devices only run at their own rate, often 44.1 kHz, so the output resamples.
    jitter_buffer.set_output_rate(config.sample_rate.0);

    let err_fn = |err| eprintln!("[audio] Stream error: {}", err);

    let stream = match sample_format {
        SampleFormat::F32 => {
            let jb = Arc::clone(&jitter_buffer);
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        jb.pop_samples(data, channels);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| Error::other(format!("failed to build f32 audio stream: {}", e)))?
        }
        SampleFormat::I16 => {
            let jb = Arc::clone(&jitter_buffer);
            let mut float_buf = Vec::new();
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        float_buf.resize(data.len(), 0.0f32);
                        jb.pop_samples(&mut float_buf, channels);
                        for (dest, &src) in data.iter_mut().zip(float_buf.iter()) {
                            *dest = (src * 32767.0).clamp(-32768.0, 32767.0) as i16;
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| Error::other(format!("failed to build i16 audio stream: {}", e)))?
        }
        other => {
            return Err(Error::other(format!(
                "unsupported audio sample format: {:?}",
                other
            )));
        }
    };

    stream
        .play()
        .map_err(|e| Error::other(format!("failed to start playback stream: {}", e)))?;
    println!(
        "[audio] Output stream initialized ({} Hz)",
        config.sample_rate.0
    );

    Ok(stream)
}

pub fn start_loopback_stream(jitter_buffer: Arc<JitterBuffer>) -> io::Result<Stream> {
    let host = cpal::default_host();
    let default_output = host
        .default_output_device()
        .ok_or_else(|| Error::other("no default output device for loopback"))?;

    let supported_config = default_output
        .default_output_config()
        .map_err(|e| Error::other(format!("failed to query loopback format: {}", e)))?;

    let channels = supported_config.channels();
    let sample_rate = supported_config.sample_rate().0;
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();

    let err_fn = |err| eprintln!("[loopback] Loopback capture stream error: {}", err);

    let stream = match sample_format {
        SampleFormat::F32 => {
            let jb = Arc::clone(&jitter_buffer);
            default_output
                .build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        jb.push_reference_samples(data, channels, sample_rate);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| Error::other(format!("failed to build f32 loopback stream: {}", e)))?
        }
        SampleFormat::I16 => {
            let jb = Arc::clone(&jitter_buffer);
            let mut temp = Vec::new();
            default_output
                .build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        temp.clear();
                        temp.extend(data.iter().map(|&t| t as f32 / 32768.0));
                        jb.push_reference_samples(&temp, channels, sample_rate);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| Error::other(format!("failed to build i16 loopback stream: {}", e)))?
        }
        other => {
            return Err(Error::other(format!(
                "unsupported loopback sample format: {:?}",
                other
            )));
        }
    };

    stream
        .play()
        .map_err(|e| Error::other(format!("failed to start loopback capture stream: {}", e)))?;

    Ok(stream)
}
