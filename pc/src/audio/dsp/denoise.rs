use super::echo::EchoSuppressor;
use nnnoiseless::DenoiseState;

const GATE_THRESHOLD_RMS: f32 = 180.0;
const GATE_FLOOR_GAIN: f32 = 0.05;
const GATE_ATTACK_ALPHA: f32 = 0.35;
const GATE_RELEASE_ALPHA: f32 = 0.04;

pub struct AudioDsp {
    denoise: Box<DenoiseState<'static>>,
    echo_suppressor: EchoSuppressor,
    pub(crate) gate_gain: f32,
    scratch_in: [f32; DenoiseState::FRAME_SIZE],
    scratch_out: [f32; DenoiseState::FRAME_SIZE],
}

impl AudioDsp {
    pub fn new() -> Self {
        Self {
            denoise: DenoiseState::new(),
            echo_suppressor: EchoSuppressor::new(),
            gate_gain: 1.0,
            scratch_in: [0.0; DenoiseState::FRAME_SIZE],
            scratch_out: [0.0; DenoiseState::FRAME_SIZE],
        }
    }

    pub fn push_reference(&mut self, samples: &[f32]) {
        self.echo_suppressor.push_reference(samples);
    }

    pub fn process(&mut self, samples: &mut [i16], ns_strength_pct: u32, aec_enabled: bool) {
        if samples.is_empty() {
            return;
        }

        let strength_ratio = (ns_strength_pct.min(100) as f32) / 100.0;

        for chunk in samples.chunks_mut(DenoiseState::FRAME_SIZE) {
            let chunk_len = chunk.len();
            if chunk_len < DenoiseState::FRAME_SIZE {
                for s in chunk.iter_mut() {
                    *s = ((*s as f32) * self.gate_gain).clamp(-32768.0, 32767.0) as i16;
                }
                continue;
            }

            for (dest, &src) in self.scratch_in.iter_mut().zip(chunk.iter()) {
                *dest = src as f32;
            }

            if aec_enabled {
                self.echo_suppressor.process_chunk(&mut self.scratch_in);
            }

            let sum_sq: f32 = self.scratch_in.iter().map(|&v| v * v).sum();
            let frame_rms = (sum_sq / DenoiseState::FRAME_SIZE as f32).sqrt();

            let target_gate_gain = if frame_rms < GATE_THRESHOLD_RMS {
                GATE_FLOOR_GAIN
            } else {
                1.0
            };

            let alpha = if target_gate_gain > self.gate_gain {
                GATE_ATTACK_ALPHA
            } else {
                GATE_RELEASE_ALPHA
            };
            self.gate_gain += (target_gate_gain - self.gate_gain) * alpha;

            if strength_ratio > 0.001 {
                let _vad = self
                    .denoise
                    .process_frame(&mut self.scratch_out, &self.scratch_in);

                for (dest, (&raw, &denoised)) in chunk
                    .iter_mut()
                    .zip(self.scratch_in.iter().zip(self.scratch_out.iter()))
                {
                    let blended = raw * (1.0 - strength_ratio) + denoised * strength_ratio;
                    let gated = blended * self.gate_gain;
                    *dest = gated.clamp(-32768.0, 32767.0) as i16;
                }
            } else {
                for (dest, &raw) in chunk.iter_mut().zip(self.scratch_in.iter()) {
                    let gated = raw * self.gate_gain;
                    *dest = gated.clamp(-32768.0, 32767.0) as i16;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.echo_suppressor.reset();
        self.gate_gain = 1.0;
        self.scratch_in.fill(0.0);
        self.scratch_out.fill(0.0);
    }
}

impl Default for AudioDsp {
    fn default() -> Self {
        Self::new()
    }
}
