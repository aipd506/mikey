use super::constants::*;
use super::JitterBuffer;
use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl JitterBuffer {
    /// Fills the output device's buffer, resampled from the phone's 48 kHz to the device's rate.
    pub fn pop_samples(&self, out: &mut [f32], channels: u16) {
        let ch = channels.max(1) as usize;
        let mut started = self.started.lock().unwrap();
        let mut buf = self.buffer.lock().unwrap();
        let target = self.adaptive_target_samples.load(Ordering::Relaxed);
        let mut phase = self.resample_phase.lock().unwrap();
        let gain = self.get_auto_gain();
        let step = SAMPLE_RATE as f32 / self.output_rate.load(Ordering::Relaxed) as f32;

        drift_resample_pop(
            &mut buf,
            out,
            ch,
            target,
            step,
            &mut phase,
            &mut started,
            gain,
        );
    }
}

pub(crate) struct JitterStats {
    pub(crate) last_arrival: Option<Instant>,
    pub(crate) last_capture_ts: Option<u64>,
    pub(crate) jitter_estimate_us: f64,
}

impl JitterStats {
    pub(crate) fn new() -> Self {
        Self {
            last_arrival: None,
            last_capture_ts: None,
            jitter_estimate_us: 0.0,
        }
    }
}

pub(crate) fn downmix_and_resample_reference(
    samples: &[f32],
    channels: u16,
    sample_rate: u32,
) -> Vec<f32> {
    let ch = channels.max(1) as usize;
    let mut mono = Vec::with_capacity(samples.len() / ch);
    for frame in samples.chunks(ch) {
        let sum: f32 = frame.iter().sum();
        mono.push(sum / ch as f32);
    }

    if sample_rate != SAMPLE_RATE && sample_rate > 0 {
        let ratio = SAMPLE_RATE as f32 / sample_rate as f32;
        let out_len = ((mono.len() as f32) * ratio) as usize;
        let mut resampled = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let src_idx = (i as f32) / ratio;
            let idx0 = (src_idx as usize).min(mono.len() - 1);
            let idx1 = (idx0 + 1).min(mono.len() - 1);
            let frac = src_idx - (idx0 as f32);
            resampled.push(mono[idx0] + frac * (mono[idx1] - mono[idx0]));
        }
        resampled
    } else {
        mono
    }
}

/// `step` is how many 48 kHz samples one output frame advances: 48 kHz over the device's rate.
/// Drift correction then nudges it by up to MAX_DRIFT_RATIO to hold the buffer at `target`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drift_resample_pop(
    buf: &mut VecDeque<i16>,
    out: &mut [f32],
    channels: usize,
    target: usize,
    step: f32,
    phase: &mut f32,
    started: &mut bool,
    gain: f32,
) {
    let frames_needed = out.len() / channels;

    if !*started {
        if buf.len() >= target {
            *started = true;
        } else {
            out.fill(0.0);
            return;
        }
    }

    let current_depth = buf.len();
    let delta = current_depth as f32 - target as f32;
    let speed_adjust = (delta / (target as f32 * 2.0)).clamp(-MAX_DRIFT_RATIO, MAX_DRIFT_RATIO);
    let effective_rate = step * (1.0 + speed_adjust);

    let mut out_idx = 0;
    for _ in 0..frames_needed {
        let frame = &mut out[out_idx..out_idx + channels];
        out_idx += channels;

        if buf.is_empty() {
            frame.fill(0.0);
            *started = false;
        } else {
            let s0 = buf[0] as f32;
            let s1 = if buf.len() > 1 { buf[1] as f32 } else { s0 };
            let sample_val = s0 + *phase * (s1 - s0);
            let normalized = (sample_val * gain) / 32768.0;
            let float_val = normalized.clamp(-1.0, 1.0);

            for item in frame.iter_mut() {
                *item = float_val;
            }

            *phase += effective_rate;
            while *phase >= 1.0 {
                if !buf.is_empty() {
                    buf.pop_front();
                }
                *phase -= 1.0;
            }
        }
    }
}

pub(crate) fn update_peak_level(peak: &std::sync::atomic::AtomicUsize, samples: &[i16], gain: f32) {
    let max_val = samples
        .iter()
        .map(|&s| ((s as f32) * gain).abs() as usize)
        .max()
        .unwrap_or(0);
    let cur = peak.load(std::sync::atomic::Ordering::Relaxed);
    if max_val > cur {
        peak.store(max_val, std::sync::atomic::Ordering::Release);
    } else {
        let decayed = (cur * 92) / 100;
        peak.store(decayed.max(max_val), std::sync::atomic::Ordering::Release);
    }
}
