use nnnoiseless::DenoiseState;
use std::collections::VecDeque;

/// Maximum reference history held for cross-correlation delay matching (~300 ms at 48 kHz)
const MAX_REF_SAMPLES: usize = 14_400;
/// Minimum delay search bound (~5 ms at 48 kHz)
const MIN_DELAY_SAMPLES: usize = 240;
/// Maximum delay search bound (~140 ms at 48 kHz)
const MAX_DELAY_SAMPLES: usize = 6_720;

/// Acoustic Echo Canceller with PC Speaker Loopback Reference matching.
/// Cancels system sounds / speaker audio bleeding into the phone microphone.
pub struct EchoSuppressor {
    ref_ring: VecDeque<f32>,
    prev_frame: [f32; DenoiseState::FRAME_SIZE],
    feedback_est: [f32; DenoiseState::FRAME_SIZE],
    decay: f32,
}

impl EchoSuppressor {
    pub fn new() -> Self {
        Self {
            ref_ring: VecDeque::with_capacity(MAX_REF_SAMPLES),
            prev_frame: [0.0; DenoiseState::FRAME_SIZE],
            feedback_est: [0.0; DenoiseState::FRAME_SIZE],
            decay: 0.85,
        }
    }

    /// Pushes reference audio samples captured from PC speaker output (WASAPI loopback).
    pub fn push_reference(&mut self, samples: &[f32]) {
        self.ref_ring.extend(samples.iter().copied());
        if self.ref_ring.len() > MAX_REF_SAMPLES {
            let excess = self.ref_ring.len() - MAX_REF_SAMPLES;
            self.ref_ring.drain(0..excess);
        }
    }

    /// Suppresses PC speaker loopback bleed and acoustic room reflections from microphone audio.
    pub fn process_chunk(&mut self, chunk: &mut [f32]) {
        let frame_size = DenoiseState::FRAME_SIZE;

        // Step 1: System Sound Cancellation via Speaker Loopback Correlation
        let ref_len = self.ref_ring.len();
        if ref_len >= MAX_DELAY_SAMPLES + frame_size {
            let search_window_start = ref_len - (MAX_DELAY_SAMPLES + frame_size);
            let search_window_end = ref_len;
            let ref_energy: f32 = self
                .ref_ring
                .range(search_window_start..search_window_end)
                .step_by(8)
                .map(|&s| s * s)
                .sum::<f32>()
                / ((MAX_DELAY_SAMPLES + frame_size) / 8) as f32;

            if ref_energy > 1600.0 {
                let mut best_corr = 0.0f32;
                let mut best_delay = MIN_DELAY_SAMPLES;

                let mut delay = MIN_DELAY_SAMPLES;
                while delay <= MAX_DELAY_SAMPLES {
                    let start_idx = ref_len - delay - frame_size;
                    let mut corr = 0.0f32;
                    let mut i = 0;
                    while i < frame_size {
                        corr += chunk[i] * self.ref_ring[start_idx + i];
                        i += 4;
                    }
                    if corr > best_corr {
                        best_corr = corr;
                        best_delay = delay;
                    }
                    delay += 48;
                }

                let fine_start = best_delay.saturating_sub(24).max(MIN_DELAY_SAMPLES);
                let fine_end = (best_delay + 24).min(MAX_DELAY_SAMPLES);
                let mut fine_delay = fine_start;
                while fine_delay <= fine_end {
                    let start_idx = ref_len - fine_delay - frame_size;
                    let mut corr = 0.0f32;
                    let mut i = 0;
                    while i < frame_size {
                        corr += chunk[i] * self.ref_ring[start_idx + i];
                        i += 2;
                    }
                    if corr > best_corr {
                        best_corr = corr;
                        best_delay = fine_delay;
                    }
                    fine_delay += 2;
                }

                let start_idx = ref_len - best_delay - frame_size;
                let mut ref_frame_energy = 0.0f32;
                for i in 0..frame_size {
                    let r = self.ref_ring[start_idx + i];
                    ref_frame_energy += r * r;
                }

                if ref_frame_energy > 1000.0 && best_corr > 0.0 {
                    let alpha = (best_corr * 2.0 / (ref_frame_energy + 1000.0)).clamp(0.0, 1.8);
                    if alpha > 0.05 {
                        for (item, &r) in chunk
                            .iter_mut()
                            .zip(self.ref_ring.range(start_idx..start_idx + frame_size))
                        {
                            let echo = r * alpha;
                            *item -= echo;
                        }

                        let norm_corr = best_corr / ((ref_frame_energy + 1.0).sqrt() * 1000.0);
                        if norm_corr > 0.3 {
                            let suppression = (1.0 - (norm_corr * 0.4)).clamp(0.4, 1.0);
                            for item in chunk.iter_mut() {
                                *item *= suppression;
                            }
                        }
                    }
                }
            }
        }

        // Step 2: Room Acoustic Reflection & Feedback Suppression
        for (i, item) in chunk.iter_mut().enumerate() {
            let sample = *item;
            let estimated_echo = self.prev_frame[i] * 0.3 + self.feedback_est[i] * 0.2;
            self.feedback_est[i] = estimated_echo * self.decay;
            self.prev_frame[i] = sample;

            let suppressed = sample - estimated_echo;
            *item = suppressed.clamp(-32768.0, 32767.0);
        }
    }

    pub fn reset(&mut self) {
        self.ref_ring.clear();
        self.prev_frame.fill(0.0);
        self.feedback_est.fill(0.0);
    }
}

impl Default for EchoSuppressor {
    fn default() -> Self {
        Self::new()
    }
}
