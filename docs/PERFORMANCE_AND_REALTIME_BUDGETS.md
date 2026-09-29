# Performance Budgets & Real-Time Constraints

Mikey is a real-time communications system. Every millisecond of latency, every dropped audio packet, and every megabyte of RAM is governed by **strict, non-negotiable performance budgets**.

---

## 1. Latency Budgets by Transport Level

Total end-to-end latency is measured from the physical moment a sound wave strikes the phone’s microphone diaphragm (or light enters the camera sensor) to the moment the audio sample/video pixel is delivered to the Windows virtual device driver.

| Level | Physical Medium | Audio Latency Budget | Video Latency Budget | Jitter Buffer Target |
| :---: | :--- | :---: | :---: | :---: |
| **1** | **USB Debugging** (ADB reverse) | **≤ 20 ms** | **≤ 30 ms** | 20 ms (960 samples) |
| **2** | **USB Tethering** (RNDIS/NCM) | **≤ 25 ms** | **≤ 35 ms** | 20 ms (960 samples) |
| **3** | **Wi-Fi / LAN** (802.11ac/ax) | **≤ 40 ms** | **≤ 60 ms** | 40 ms (1,920 samples) |
| **4** | **Bluetooth RFCOMM** (SPP) | **≤ 80 ms** | *N/A (Audio Only)* | 80 ms (3,840 samples) |

### 1.1 Audio Subsystem Latency Breakdown (Level 1 USB)

```text
Physical Mic Diaphragm
       │  ~1.0 ms (Acoustic & Hardware Ingest)
       ▼
AAudio Native Ingest (aaudio_jni.c)
       │  ~3.0 ms (ALSA Kernel DMA Buffer)
       ▼
10 ms Frame Packaging (AudioCapture.kt)
       │  ~5.0 ms (Average packetization wait)
       ▼
ADB Reverse Socket Transit (127.0.0.1:7653)
       │  ~1.5 ms (USB 2.0/3.0 Bulk Transfer & Loopback)
       ▼
PC JitterBuffer & DSP (pc/src/audio/)
       │  ~5.0 ms (Queue stabilization + Resampler + Normalizer)
       ▼
WASAPI System Playback Buffer (sink/stream.rs)
       │  ~3.0 ms (Windows Shared Mixer Engine)
       ▼
VB-Audio Virtual Cable / Mic Endpoint
       │
       Total: ~18.5 ms (Well within ≤ 20 ms budget)
```

---

## 2. Resource & Memory Consumption Budgets

| Metric | Android Client Budget | PC Daemon Budget | Rationale / Target |
| :--- | :---: | :---: | :--- |
| **Idle CPU Usage** | **0.0%** (Process idle) | **< 0.5%** | PC tray must sit silently in background without waking CPU cores |
| **Active CPU (Audio Only)** | **< 2.0%** | **< 1.0%** | Audio pipeline uses pre-allocated buffers and SIMD-friendly DSP |
| **Active CPU (Audio + Video)** | **< 5.0%** | **< 3.5%** | CameraX native YUV conversion + TurboJPEG multithreaded decode |
| **RAM Footprint (Idle)** | **< 30 MB** | **< 20 MB** | No heavy runtimes or embedded web engines |
| **RAM Footprint (Active)** | **< 60 MB** | **< 40 MB** | Zero dynamic heap expansion in hot streaming loops |
| **Binary Size** | **< 10 MB** (APK) | **< 8 MB** (EXE) | Single self-contained static executable with zero DLL installers |

---

## 3. Buffer Bounds & Real-Time Timeout Policies

### 3.1 Audio Buffer Ceilings
- **Frame Granularity**: Exactly **10 ms** (480 samples @ 48 kHz mono = 960 bytes).
- **Maximum Adaptive Buffer**: **120 ms** (`MAX_ADAPTIVE_TARGET_MS = 120`, 5,760 samples).
- **Hard Latency Drop Cap**: **200 ms** (`MAX_LATENCY_MS = 200`, 9,600 samples). If packet flow stalls and backlog exceeds 200 ms, the oldest samples are dropped immediately to preserve real-time conversation synchronization.
- **Clock Drift Clamp**: **±0.2%** (`MAX_DRIFT_RATIO = 0.002`). Resampling pitch adjustments are kept within imperceptible limits.

### 3.2 Video Queue Discipline
- **Backpressure Strategy**: `STRATEGY_KEEP_ONLY_LATEST`. The video queue depth is exactly **1 frame**. If the network or decoder thread is processing a frame when a new sensor image arrives, the intermediate frame is dropped immediately.

### 3.3 Network Timeouts & Anti-Flap Timers
- **Socket Connect Timeout**: **500 ms** per candidate interface.
- **Heartbeat Interval**: **5,000 ms** (`HEARTBEAT_INTERVAL_MS`).
- **Socket Read Timeout**: **15,000 ms** (`LINK_TIMEOUT_MS`). Three missed consecutive heartbeats trigger immediate socket abortion.
- **Session Hold Grace Period**: **30,000 ms** (`SESSION_HOLD_DURATION`). Virtual device handles remain alive for 30 seconds following abrupt disconnection.
- **Anti-Flap Probation (`DEAD_MS`)**: **10,000 ms**. A dropped or failed transport level is locked out for 10 seconds to eliminate flapping between unstable connections.

---

## 4. Verification & Quality Commands

Engineers and automated CI pipelines verify performance and code quality using the following standard commands:

### Android Verification
```bash
# Run unit tests
cd android && ./gradlew testDebugUnitTest

# Run static analysis and linting
cd android && ./gradlew lintDebug
```

### PC Verification
```bash
# Run formatting inspection
cd pc && cargo fmt --check

# Run static linter with zero allowed warnings
cd pc && cargo clippy -- -D warnings

# Execute unit and protocol regression tests
cd pc && cargo test

# Verify audio output pipeline without a physical phone (plays 3-second test tone)
cd pc && cargo run -- --test-tone
```
