//! Processing switches the phone owns (wire-protocol.md CONTROL), next to the ones in mod.rs.

use super::JitterBuffer;
use std::sync::atomic::Ordering;

impl JitterBuffer {
    /// Noise suppression on or off. Off keeps the strength, so switching back on restores it.
    pub fn set_ns_enabled(&self, enabled: bool) {
        self.ns_enabled.store(enabled, Ordering::Release);
    }

    pub fn is_ns_enabled(&self) -> bool {
        self.ns_enabled.load(Ordering::Acquire)
    }

    /// The strength the processing uses: 0 while noise suppression is off.
    pub(super) fn effective_ns_strength(&self) -> u32 {
        if self.is_ns_enabled() {
            self.get_ns_strength()
        } else {
            0
        }
    }

    /// The noise gate threshold in dB, or None for no gate (the default).
    pub fn set_gate_db(&self, gate_db: Option<f32>) {
        let bits = gate_db.unwrap_or(f32::NAN).to_bits();
        self.gate_db_bits.store(bits, Ordering::Release);
    }

    pub fn gate_db(&self) -> Option<f32> {
        Some(f32::from_bits(self.gate_db_bits.load(Ordering::Acquire))).filter(|db| !db.is_nan())
    }
}

/// A gate threshold in dB below full scale as the RMS of 16-bit samples, 0 for no gate.
/// -45 dB is about 184.
pub fn gate_rms(gate_db: Option<f32>) -> f32 {
    gate_db.map_or(0.0, |db| 32768.0 * 10f32.powf(db / 20.0))
}
