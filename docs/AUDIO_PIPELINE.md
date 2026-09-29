# Audio Pipeline & Digital Signal Processing (DSP) Architecture

The Mikey audio subsystem is architected for **broadcast-grade speech fidelity, minimal end-to-end latency, and robust jitter resilience**. The architecture strictly separates raw, unadulterated capture on Android from intensive mathematical signal processing on the PC.

---

## 1. End-to-End Audio Pipeline Overview

```text
[Phone Microphone Hardware]
             │
             ▼
[AudioCapture.kt] ── 48 kHz mono 16-bit PCM (10 ms frames = 480 samples = 960 bytes)
    ├── Primary: AAudio NDK (AAUDIO_PERFORMANCE_MODE_LOW_LATENCY; UNPROCESSED with VOICE_RECOGNITION fallback)
    └── Fallback: AudioRecord (MediaRecorder.AudioSource.UNPROCESSED or VOICE_RECOGNITION)
             │
             ├──► [L1/L2 USB]: Transmitted as raw PCM (0x01 AUDIO, codec 0x01)
             │
             └──► [L3/L4 Wi-Fi/BT]: FrameJoiner.kt (aggregates two 10ms frames into 20ms)
                       │
                       ▼
                  Native Opus Encoder (opus_jni.c) ── 96 kbps (Wi-Fi) / 48 kbps (BT)
                       │
                       ▼
                 [Wire Protocol Socket (0x01 AUDIO, codec 0x02)]
                       │
                       ▼
                 [PC Client Ingest Thread]
                       │
                       ├── Raw PCM Ingest: unpacks little-endian i16 samples
                       └── Opus Decoder: decompress to 480 samples (10 ms)
                                │
                                ▼
                       [JitterBuffer] (pc/src/audio/pipeline/mod.rs)
                          - Adaptive target queue depth: 20 ms to 120 ms
                          - EMA arrival jitter variance tracking
                          - Hard latency cap: 200 ms (MAX_LATENCY_MS)
                                │
                                ▼
                       [Drift Resampler] (pc/src/audio/pipeline/resample.rs)
                          - Linear interpolation phase accumulator
                          - Continuous crystal clock drift clamp (±0.2%)
                                │
                                ▼
                       [AudioNormalizer] (pc/src/audio/pipeline/normalizer.rs)
                          - Transparent unity gain delivery (1.0×, zero ducking)
                          - Real-time vocal RMS tracking and speech headroom safety
                                │
                                ▼
                       [AudioDsp / RNNoise] (pc/src/audio/dsp/denoise.rs)
                          - Neural network RNN speech/noise classification
                          - Noise gate & user-adjustable strength mixing
                                │
                                ▼
                       [WASAPI Virtual Sink] (pc/src/audio/sink/)
                          - High-priority real-time audio thread callback
                          - Non-blocking try-lock fetch
                          - Feeds virtual microphone endpoint (VB-Audio Cable)
```

---

## 2. Android Capture Engine Details

### 2.1 Hardware Capture (`AudioCapture.kt`)
- **Native AAudio Engine (`app/src/main/cpp/aaudio_jni.c`)**:
  - Uses `AAudioStreamBuilder_setPerformanceMode(builder, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY)`.
  - Configured with `AAUDIO_SHARING_MODE_SHARED` (or `EXCLUSIVE` where supported by kernel drivers) to achieve direct DMA buffer reads from the audio DSP.
  - Tries `AAUDIO_INPUT_PRESET_UNPROCESSED` on Android 9+, falling back seamlessly to `AAUDIO_INPUT_PRESET_VOICE_RECOGNITION` if unsupported by device vendor HAL.
- **AudioRecord Fallback**:
  - Automatically activates if AAudio fails or is preempted by an incoming phone call.
  - Queries `AudioManager.PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED`. If true, sets `MediaRecorder.AudioSource.UNPROCESSED`. Otherwise, falls back to `MediaRecorder.AudioSource.VOICE_RECOGNITION` (which Android CDD mandates disables OEM AGC and compression, unlike `AudioSource.MIC`).
  - **System AGC Disable**: If `AutomaticGainControl.isAvailable()`, the capture loop explicitly creates and disables system AGC on the recorder's audio session ID to eliminate hardware voice ducking.
  - **The Voice Communication Rule**: Under no circumstances does Mikey use `AudioSource.VOICE_COMMUNICATION`. Android’s native voice processing injects aggressive, non-linear hardware AGC and echo cancellation that fundamentally breaks PC-side neural noise filters.
- **Thread Priority**: The capture loop runs inside a dedicated OS thread (`mikey-capture`) pinned to `Process.THREAD_PRIORITY_URGENT_AUDIO`.

### 2.2 Frame Joining & Opus Encoding
- **Native Frame Granularity**: 10 ms (480 samples @ 48 kHz mono = 960 bytes).
- **Opus Frame Joiner (`FrameJoiner.kt`)**: Opus delivers significantly higher coding efficiency and lower frame header overhead at 20 ms chunk sizes. For wireless transports (Wi-Fi and Bluetooth), `FrameJoiner` buffers two consecutive 10 ms frames into a single 20 ms frame (960 samples = 1,920 bytes) before encoding.
- **Native libopus Integration (`app/src/main/cpp/opus_jni.c`)**:
  ```c
  OpusEncoder* enc = opus_encoder_create(48000, 1, OPUS_APPLICATION_VOIP, &err);
  opus_encoder_ctl(enc, OPUS_SET_BITRATE(bitrate));
  opus_encoder_ctl(enc, OPUS_SET_COMPLEXITY(5));
  opus_encoder_ctl(enc, OPUS_SET_SIGNAL(OPUS_SIGNAL_VOICE));
  ```

---

## 3. PC Audio Processing Pipeline

### 3.1 Adaptive Jitter Buffer (`pc/src/audio/pipeline/mod.rs`)
The PC `JitterBuffer` absorbs packet arrival variance across unpredictable wireless links while maintaining sub-frame latency over USB:

- **Target Latencies per Medium (`constants.rs`)**:
  ```rust
  pub const SAMPLE_RATE: u32 = 48_000;
  pub const SAMPLES_PER_MS: usize = 48;
  pub const USB_TARGET_MS: usize = 20;     // 960 samples
  pub const WIFI_TARGET_MS: usize = 40;    // 1920 samples
  pub const BT_TARGET_MS: usize = 80;      // 3840 samples
  pub const MAX_ADAPTIVE_TARGET_MS: usize = 120; // 5760 samples max
  pub const MAX_LATENCY_MS: usize = 200;   // 9600 samples hard cap
  ```
- **EMA Arrival Tracking (`record_arrival`)**:
  Every frame contains a 64-bit hardware capture timestamp (`capture_ts`). When a frame arrives at instant $t_{\text{now}}$, the delta arrival time is compared against the delta capture timestamp:
  $$D = |(t_{\text{arrival}} - t_{\text{last\_arrival}}) - (\text{ts}_{\text{capture}} - \text{ts}_{\text{last\_capture}})|$$
  $$\text{JitterEstimate} \leftarrow \text{JitterEstimate} + \frac{D - \text{JitterEstimate}}{16.0}$$
  $$\text{AdaptiveTarget} = \text{BaseTarget} + (2 \times \text{JitterEstimate})$$
- **Hard Latency Drop**: If the buffer exceeds `MAX_SAMPLES = 9,600` (200 ms), the oldest samples are instantly purged. This prevents "latency creep" from accumulating over long streaming sessions.

### 3.2 Clock Drift Compensation (`pc/src/audio/pipeline/resample.rs`)
Because the phone’s hardware audio clock and the PC’s DAC audio clock are driven by separate physical quartz oscillators, their rates inevitably drift apart by up to ±50 ppm.

Mikey corrects drift using **continuous linear interpolation**:
- **Phase Accumulator**: Maintains a fractional sample offset $\text{phase} \in [0.0, 1.0)$.
- **Drift Ratio Clamp**:
  $$\text{Ratio} = 1.0 + \text{clamp}\left(\frac{\text{QueueDepth} - \text{TargetDepth}}{\text{AdjustmentScale}}, -0.002, +0.002\right)$$
- The ±0.2% limit (`MAX_DRIFT_RATIO = 0.002`) guarantees that any pitch adjustment is completely imperceptible to human speech perception while providing sufficient authority to eliminate buffer underruns.

### 3.3 Auto Loudness Normalization (`pc/src/audio/pipeline/normalizer.rs`)
- **Unity Gain Delivery**: Maintains constant unity gain (`MIN_AUTO_GAIN = 1.0`, `MAX_AUTO_GAIN = 1.0`, multiplier `1.0×`) across all vocal inputs. This completely eliminates dynamic voice ducking, compressor pumping, and background ambient noise surges during speech pauses.
- **Vocal Headroom & Monitoring**: Tracks continuous vocal RMS against broadcast speech standards (`TARGET_RMS_I16 = 4126.0`, -18 dBFS) for flyout UI level metering while preserving linear input dynamics for meeting applications.

### 3.4 Neural Speech Denoising (`pc/src/audio/dsp/denoise.rs`)
- **Model**: Embedded **RNNoise** recurrent neural network model trained on voice and background noise spectra.
- **Frequency Analysis**: Operates on 10 ms frequency bark bands, calculating speech presence probabilities and attenuating non-stationary noise (keyboards, HVAC, traffic).
- **Strength Slider**: The PC flyout UI exposes an adjustable 0–100% noise reduction strength slider (`ns_strength`). The output linearly crossfades between the clean dry signal and the neural denoised signal.

---

## 4. Virtual Microphone Output (WASAPI Sink)

In `pc/src/audio/sink/`:
- **WASAPI Integration**: Operates in Windows Audio Session API shared event-driven mode (`AUDCLNT_STREAMFLAGS_EVENTCALLBACK`).
- **Real-Time Guarantee**: The WASAPI render callback executes at high real-time priority. It retrieves audio from `JitterBuffer` via non-blocking try-locks. If contention occurs or the buffer runs dry, digital silence (zeroes) is emitted immediately to prevent audio stutter or crackle.
- **Virtual Audio Cable Compatibility**: Seamlessly links to VB-Audio Cable or dedicated virtual driver endpoints, exposing the stream as a standard microphone in Windows Sound Settings.
