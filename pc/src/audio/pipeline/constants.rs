pub const SAMPLE_RATE: u32 = 48_000;
pub const SAMPLES_PER_MS: usize = (SAMPLE_RATE / 1000) as usize; // 48 samples/ms
pub const MAX_LATENCY_MS: usize = 200; // 200 ms hard cap
pub const MAX_SAMPLES: usize = MAX_LATENCY_MS * SAMPLES_PER_MS; // 9600 samples

pub const USB_TARGET_MS: usize = 20; // 960 samples
pub const WIFI_TARGET_MS: usize = 40; // 1920 samples
pub const BT_TARGET_MS: usize = 80; // 3840 samples
pub const MAX_ADAPTIVE_TARGET_MS: usize = 120; // 5760 samples max

pub const MAX_DRIFT_RATIO: f32 = 0.002; // ±0.2% max drift correction

// Auto loudness normalization: targets unity gain (1.0×) to eliminate voice ducking and pumping
pub const TARGET_RMS_I16: f32 = 4126.0; // 32767 × 10^(−18/20)
pub const NOISE_FLOOR_I16: f32 = 100.0; // below this RMS, hold current gain
pub const MAX_AUTO_GAIN: f32 = 1.0; // unity ceiling prevents voice ducking and background pumping
pub const MIN_AUTO_GAIN: f32 = 1.0; // unity floor
pub const RMS_ALPHA: f32 = 0.1; // EMA smoothing (~100 ms at 10 ms chunks)
pub const GAIN_ATTACK_ALPHA: f32 = 0.05; // gain-up rate per chunk
pub const GAIN_RELEASE_ALPHA: f32 = 0.01; // gain-down rate per chunk
