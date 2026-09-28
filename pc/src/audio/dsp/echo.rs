use nnnoiseless::DenoiseState;
use std::collections::VecDeque;

/// Maximum reference history held (~400 ms at 48 kHz).
const MAX_REF_SAMPLES: usize = 19_200;
/// Minimum delay search bound (~5 ms at 48 kHz).
const MIN_DELAY_SAMPLES: usize = 240;
/// Maximum delay search bound (~250 ms at 48 kHz).
const MAX_DELAY_SAMPLES: usize = 12_000;

/// Acoustic Echo Canceller with PC Speaker Loopback Reference matching.
/// Eliminates remote participant voice looping through PC speakers into phone mic.
pub struct EchoSuppressor {
    ref_ring: VecDeque<f32>,
    last_delay: usize,
    suppression_gain: f32,
}

impl EchoSuppressor {
    pub fn new() -> Self {
        Self {
            ref_ring: VecDeque::with_capacity(MAX_REF_SAMPLES),
            last_delay: 2_400,
            suppression_gain: 1.0,
        }
    }

    /// Pushes reference audio samples captured from PC speaker output (WASAPI loopback).
    /// Automatically detects and normalizes float amplitude [-1.0, 1.0] to PCM scale.
    pub fn push_reference(&mut self, samples: &[f32]) {
        let is_normalized = samples.iter().take(64).all(|&s| s.abs() <= 1.5);
        let scale = if is_normalized { 32767.0 } else { 1.0 };
        for &s in samples {
            self.ref_ring.push_back(s * scale);
        }
        if self.ref_ring.len() > MAX_REF_SAMPLES {
            let excess = self.ref_ring.len() - MAX_REF_SAMPLES;
            self.ref_ring.drain(0..excess);
        }
    }

    /// Suppresses PC speaker loopback bleed from microphone audio.
    pub fn process_chunk(&mut self, chunk: &mut [f32]) {
        let frame_size = DenoiseState::FRAME_SIZE;
        let ref_len = self.ref_ring.len();
        if ref_len < MAX_DELAY_SAMPLES + frame_size {
            return;
        }

        let mic_energy: f32 = chunk.iter().map(|&s| s * s).sum::<f32>() / frame_size as f32;

        let best_delay = self.find_best_delay(ref_len, frame_size, chunk);
        self.last_delay = best_delay;

        let start_idx = ref_len - best_delay - frame_size;
        let mut ref_energy = 0.0f32;
        let mut dot_prod = 0.0f32;
        for (i, &mic) in chunk.iter().enumerate() {
            let r = self.ref_ring[start_idx + i];
            ref_energy += r * r;
            dot_prod += mic * r;
        }
        ref_energy /= frame_size as f32;
        dot_prod /= frame_size as f32;

        if ref_energy > 400.0 {
            let norm_corr = (dot_prod / ((mic_energy * ref_energy).sqrt() + 1.0)).clamp(-1.0, 1.0);
            let alpha = (dot_prod / (ref_energy + 1.0)).clamp(0.0, 2.0);

            if norm_corr > 0.15 && alpha > 0.02 {
                for (item, &r) in chunk
                    .iter_mut()
                    .zip(self.ref_ring.range(start_idx..start_idx + frame_size))
                {
                    *item -= r * alpha;
                }
            }

            let post_energy: f32 = chunk.iter().map(|&s| s * s).sum::<f32>() / frame_size as f32;
            let is_far_end_active = ref_energy > 800.0;
            let is_near_end_talking = post_energy > ref_energy * 0.8 && norm_corr < 0.3;

            let target_gain = if norm_corr > 0.45 || (is_far_end_active && !is_near_end_talking) {
                0.03
            } else if norm_corr > 0.25 {
                0.35
            } else {
                1.0
            };

            let alpha_step = if target_gain < self.suppression_gain {
                0.4
            } else {
                0.15
            };
            self.suppression_gain += (target_gain - self.suppression_gain) * alpha_step;
        } else {
            self.suppression_gain += (1.0 - self.suppression_gain) * 0.25;
        }

        if self.suppression_gain < 0.99 {
            for item in chunk.iter_mut() {
                *item = (*item * self.suppression_gain).clamp(-32768.0, 32767.0);
            }
        }
    }

    fn find_best_delay(&self, ref_len: usize, frame_size: usize, chunk: &[f32]) -> usize {
        let (search_start, search_end, step) = if self.last_delay >= 480 {
            (
                self.last_delay.saturating_sub(480).max(MIN_DELAY_SAMPLES),
                (self.last_delay + 480).min(MAX_DELAY_SAMPLES),
                16,
            )
        } else {
            (MIN_DELAY_SAMPLES, MAX_DELAY_SAMPLES, 48)
        };

        let mut best_corr = -1.0f32;
        let mut best_delay = self.last_delay;

        let mut delay = search_start;
        while delay <= search_end {
            let start = ref_len - delay - frame_size;
            let mut corr = 0.0f32;
            let mut i = 0;
            while i < frame_size {
                corr += chunk[i] * self.ref_ring[start + i];
                i += 4;
            }
            if corr > best_corr {
                best_corr = corr;
                best_delay = delay;
            }
            delay += step;
        }

        let fine_start = best_delay.saturating_sub(step).max(MIN_DELAY_SAMPLES);
        let fine_end = (best_delay + step).min(MAX_DELAY_SAMPLES);
        let mut fine = fine_start;
        while fine <= fine_end {
            let start = ref_len - fine - frame_size;
            let mut corr = 0.0f32;
            let mut i = 0;
            while i < frame_size {
                corr += chunk[i] * self.ref_ring[start + i];
                i += 2;
            }
            if corr > best_corr {
                best_corr = corr;
                best_delay = fine;
            }
            fine += 2;
        }

        best_delay
    }

    pub fn reset(&mut self) {
        self.ref_ring.clear();
        self.last_delay = 2_400;
        self.suppression_gain = 1.0;
    }
}

impl Default for EchoSuppressor {
    fn default() -> Self {
        Self::new()
    }
}
