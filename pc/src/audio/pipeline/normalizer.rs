use super::constants::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct AudioNormalizer {
    auto_gain: AtomicUsize,
    rms_estimate: Mutex<f32>,
    smoothed_gain: Mutex<f32>,
}

impl AudioNormalizer {
    pub fn new() -> Self {
        Self {
            auto_gain: AtomicUsize::new(1000), // 1.0× initial
            rms_estimate: Mutex::new(0.0),
            smoothed_gain: Mutex::new(1.0),
        }
    }

    pub fn get_gain(&self) -> f32 {
        self.auto_gain.load(Ordering::Acquire) as f32 / 1000.0
    }

    pub fn update(&self, processed: &[i16]) {
        if processed.is_empty() {
            return;
        }

        let sum_sq: f64 = processed.iter().map(|&s| (s as f64) * (s as f64)).sum();
        let chunk_rms = (sum_sq / processed.len() as f64).sqrt() as f32;

        if let (Ok(mut rms_est), Ok(mut gain)) =
            (self.rms_estimate.lock(), self.smoothed_gain.lock())
        {
            if *rms_est < 1.0 {
                *rms_est = chunk_rms;
            } else {
                *rms_est = RMS_ALPHA * chunk_rms + (1.0 - RMS_ALPHA) * *rms_est;
            }

            if *rms_est > NOISE_FLOOR_I16 {
                let target_gain = (TARGET_RMS_I16 / *rms_est).clamp(MIN_AUTO_GAIN, MAX_AUTO_GAIN);
                let alpha = if target_gain > *gain {
                    GAIN_ATTACK_ALPHA
                } else {
                    GAIN_RELEASE_ALPHA
                };
                *gain += (target_gain - *gain) * alpha;
                *gain = gain.clamp(MIN_AUTO_GAIN, MAX_AUTO_GAIN);
            }

            self.auto_gain
                .store((*gain * 1000.0) as usize, Ordering::Release);
        }
    }

    pub fn reset(&self) {
        if let Ok(mut rms) = self.rms_estimate.lock() {
            *rms = 0.0;
        }
        if let Ok(mut gain) = self.smoothed_gain.lock() {
            *gain = 1.0;
        }
        self.auto_gain.store(1000, Ordering::Relaxed);
    }
}
