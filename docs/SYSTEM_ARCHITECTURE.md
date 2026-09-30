# System Architecture & Topology

Mikey is a distributed, real-time media streaming system consisting of an Android capture client and a lightweight PC companion daemon. This document provides an exhaustive, code-level architectural breakdown of the system topology, component responsibilities, threading models, and inter-process boundaries.

---

## 1. System Topology Overview

```text
┌─────────────────────────────────────── Android Phone (Client) ────────────────────────────────────────┐
│                                                                                                        │
│   MainActivity (Jetpack Compose View)                                                                  │
│    ├── Observes: MikeyService.state (StateFlow<MikeyState>) & MikeyService.micLevel (StateFlow<Float>) │
│    └── Dispatches: Intents (ACTION_MIC_ON, ACTION_MIC_OFF, ACTION_MUTE, ACTION_FLIP, ACTION_STOP)     │
│                                                                                                        │
│   MikeyService (Foreground Service with Media Types)                                                  │
│    ├── Foreground Types: FOREGROUND_SERVICE_TYPE_MICROPHONE | FOREGROUND_SERVICE_TYPE_CAMERA           │
│    ├── Notifier (NotificationView): Notification media actions, status glyph, level indicator          │
│    └── SessionController: State machine, reconnect backoff, heartbeats, bidirectional control        │
│         ├── TransportManager: Dynamic interface listener, candidate ranking, anti-flapping probation  │
│         │    ├── [Level 1] AdbTransport       ── TCP to 127.0.0.1:7653 (via adb reverse)              │
│         │    ├── [Level 2] TetherTransport    ── TCP over USB NIC (rndis0 / usb0 / ncm0)               │
│         │    ├── [Level 3] WifiTransport      ── TCP to LAN IP + UDP Beacon (:7654) listener          │
│         │    └── [Level 4] BluetoothTransport ── RFCOMM (SPP UUID, cached MAC)                        │
│         ├── AudioCapture (AAudio / AudioRecord)                                                        │
│         │    ├── 48 kHz mono 16-bit PCM (10ms frames = 480 samples = 960 bytes)                        │
│         │    ├── Native AAudio NDK stream (aaudio_jni.c) with AudioRecord fallback                    │
│         │    ├── LevelMeter: Loudness RMS computation emitted 20 times/sec                             │
│         │    └── OpusEncoder (opus_jni.c): FrameJoiner (20ms blocks) enabled for L3 / L4              │
│         └── VideoCapture (CameraX ImageAnalysis)                                                       │
│              ├── STRATEGY_KEEP_ONLY_LATEST (zero queue backlog, frame rate cap)                        │
│              ├── OrientationEventListener: 20° hysteresis thresholding for upright picture             │
│              ├── Nv21: YUV420 to semi-planar NV21 conversion, optional 1:1 center square crop         │
│              └── Adaptive JPEG: Hardware compressToJpeg scaling quality (80% down to 50%)              │
│                                                                                                        │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                 Wire Protocol (Framed Stream over TCP / RFCOMM on Port 7653)
                 5-byte Frame Header [type: u8, len: u32 BE]
                 14-byte Media Header [seq: u32 BE, capture_ts: u64 BE µs, codec: u8, rsv: u8]
                                                    │
                                                    ▼
┌────────────────────────────────────────── PC Companion (Server) ──────────────────────────────────────┐
│                                                                                                        │
│   Listeners & Watchers:                                                                                │
│    ├── TCP Listener (0.0.0.0:7653): Accepts L1 (ADB reverse), L2 (Tether), L3 (Wi-Fi) connections       │
│    ├── UDP Discovery Responder (0.0.0.0:7654): Responds to subnet broadcast probes                    │
│    ├── ADB Watcher Thread: Background polling running `adb reverse tcp:7653 tcp:7653`                  │
│    └── Bluetooth RFCOMM Server: WinSock BTH (Windows) / BlueZ bluer (Linux)                           │
│                                                                                                        │
│   SessionManager (pc/src/session/):                                                                    │
│    ├── Config (config.toml): Trusted device tokens, level toggles, autostart, audio device overrides  │
│    ├── evaluate_hello(): TOFU authentication, automatic USB trust, Wi-Fi verification                  │
│    ├── ActiveSession: Session tokens, active transport level, last frame arrival timestamps            │
│    └── Session Hold Timer: 30-second session hold duration during network switches or transient drops  │
│                                                                                                        │
│   Audio Pipeline (pc/src/audio/):                                                                      │
│    ├── Opus Decoder: Decompresses incoming L3/L4 packets into 48 kHz PCM                               │
│    ├── JitterBuffer: Adaptive queue depth (USB 20ms, Wi-Fi 40ms, BT 80ms, max 120ms)                  │
│    │    ├── Exponential Moving Average (EMA) jitter variance tracking                                  │
│    │    └── Hard Latency Drop Cap (MAX_LATENCY_MS = 200ms)                                             │
│    ├── Drift Resampler (resample.rs): Cubic interpolation phase accumulator (clamped to ±0.2%)         │
│    ├── Loudness Normalizer (normalizer.rs): Unity gain speech delivery (1.0×, eliminates ducking/pumping)│
│    ├── Audio DSP (denoise.rs): RNNoise neural network model with dynamic strength control              │
│    └── Virtual Audio Sink (sink/): WASAPI event-driven shared stream feeding VB-Audio Cable            │
│                                                                                                        │
│   Video Pipeline (pc/src/video/):                                                                      │
│    ├── latest_jpeg Slot: Condvar-synchronized worker queue enforcing KEEP_ONLY_LATEST                  │
│    ├── TurboJPEG Decoder: Decompresses JPEG to BGRA byte arrays off the network receiver thread        │
│    ├── Virtual Camera Driver (vcam/): DirectShow in-process COM filter (softcam.dll) registration      │
│    ├── Privacy Placeholder: Pushes clean 1920x1080 dark card when camera toggles off                   │
│    └── Preview Window: Lightweight native Win32 floating preview window                                │
│                                                                                                        │
│   User Interface (pc/src/flyout/ & tray.rs):                                                           │
│    ├── Windows Shell Tray: Shell_NotifyIconW with animated icon states and context menu               │
│    └── Flyout Window: Double-buffered Win32 GDI/GDI+ card renderer, live VU meter, DSP sliders, TOFU   │
│                                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Core Subsystem Responsibilities

### 2.1 Android Subsystem Breakdown

| Class / Component | Source File | Core Responsibility |
| :--- | :--- | :--- |
| **`MainActivity`** | [`MainActivity.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/MainActivity.kt) | Pure Compose activity. Observes `MikeyService.state` and renders [`MainScreen`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/ui/MainScreen.kt). Dispatches user intents. |
| **`MikeyService`** | [`MikeyService.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/service/MikeyService.kt) | Foreground service. Manages OS capture permissions, dynamic foreground service types, and notification media actions. |
| **`SessionController`** | [`SessionController.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/service/SessionController.kt) | Master state machine. Manages network loops, wire handshake, frame writing, 5s heartbeats, and make-before-break upgrade handoffs. |
| **`TransportManager`** | [`TransportManager.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/transport/TransportManager.kt) | Network interface observer. Manages candidate ranking across L1–L4, network socket pinning, and anti-flapping probation (`DEAD_MS = 10_000`). |
| **`AudioCapture`** | [`AudioCapture.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/media/AudioCapture.kt) | Ingests 48 kHz mono PCM in 10 ms frames on `mikey-capture` thread with AAudio NDK low latency or AudioRecord fallback. |
| **`VideoCapture`** | [`VideoCapture.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/media/VideoCapture.kt) | Manages CameraX `ImageAnalysis`, gravity orientation hysteresis, NV21 conversion, and adaptive JPEG encoding. |
| **`LevelMeter`** | [`LevelMeter.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/media/LevelMeter.kt) | Computes instantaneous audio loudness RMS for UI volume bars. |
| **`Discovery`** | [`Discovery.kt`](file:///e:/Programs/mikey/android/app/src/main/java/com/mikey/transport/Discovery.kt) | Scans network interfaces, computes broadcast addresses, and emits UDP discovery probes on port `7654`. |

### 2.2 PC Subsystem Breakdown

| Module / Struct | Source File | Core Responsibility |
| :--- | :--- | :--- |
| **`main`** | [`main.rs`](file:///e:/Programs/mikey/pc/src/main.rs) | Detaches console, enforces single-instance mutex, coordinates pipeline threads, and starts network listeners. |
| **`SessionManager`** | [`session/mod.rs`](file:///e:/Programs/mikey/pc/src/session/mod.rs) | Manages authentication, TOFU pairing prompts, session hold grace periods, and control message routing. |
| **`JitterBuffer`** | [`audio/pipeline/mod.rs`](file:///e:/Programs/mikey/pc/src/audio/pipeline/mod.rs) | Thread-safe sample ring buffer with arrival timestamp tracking, adaptive depth calculation, and latency caps. |
| **`AudioNormalizer`**| [`audio/pipeline/normalizer.rs`](file:///e:/Programs/mikey/pc/src/audio/pipeline/normalizer.rs) | Unity gain speech level monitoring (1.0×) preventing voice ducking and background pumping. |
| **`AudioDsp`** | [`audio/dsp/denoise.rs`](file:///e:/Programs/mikey/pc/src/audio/dsp/denoise.rs) | Neural network speech noise suppression (RNNoise) and adjustable strength mixing. |
| **`VideoPipeline`** | [`video/pipeline.rs`](file:///e:/Programs/mikey/pc/src/video/pipeline.rs) | Dedicated worker thread decompressing JPEG to BGRA, pushing frames to softcam driver, and updating preview. |
| **`VirtualCamera`** | [`video/vcam/mod.rs`](file:///e:/Programs/mikey/pc/src/video/vcam/mod.rs) | DirectShow filter registration and shared memory frame delivery for video conferencing apps. |
| **`FlyoutWindow`** | [`flyout/window.rs`](file:///e:/Programs/mikey/pc/src/flyout/window.rs) | Native Win32 GDI/GDI+ double-buffered companion card with interactive volume meter and controls. |

---

## 3. Detailed Threading & Execution Model

### 3.1 Android Thread Architecture

```text
[Main / UI Thread] (Looper)
       │  MainActivity Compose render, user touch events, SettingsSheet interaction
       │
[Service Thread] (Looper)
       │  MikeyService lifecycle, broadcast receiver callbacks (ACTION_USB_STATE)
       │
[mikey-capture] (Native OS Thread)
       │  Process.THREAD_PRIORITY_URGENT_AUDIO
       │  Continuous read loop from AAudio / AudioRecord (10 ms ticks)
       │  Dumps to frames (ArrayBlockingQueue<AudioFrame>, capacity 8, drop oldest)
       │
[mikey-video] (Single Thread Executor)
       │  CameraX ImageAnalysis analyzer callback
       │  Orientation evaluation, YUV420 to NV21 conversion, JPEG compression
       │  Dumps to videoFrames (ArrayBlockingQueue<VideoFrame>, capacity 2, drop oldest)
       │
[mikey-session] (Dedicated Thread)
       │  SessionController loop: connects wire, performs handshake
       │  Sender loop: all waiting audio frames, then at most one picture, per turn; flushes TCP
       │  5-second periodic heartbeat sender
       │
[mikey-upgrade] (Dedicated Thread)
       │  Watches TransportManager.waitForBetterChance()
       │  Probes higher connection levels concurrently (make-before-break)
       │  Performs silent handshake and atomic socket handoff
```

### 3.2 PC Thread Architecture

```text
[Main Thread]
       │  CLI parsing, single-instance verification, launches worker threads
       │  Maintains 1-second park loop monitoring running atomic flag
       │
[Tray Message Pump Thread]
       │  Win32 message loop (GetMessageW / DispatchMessageW)
       │  Shell_NotifyIconW callback processing, tray click handling
       │
[Flyout Window Thread / WndProc]
       │  WM_PAINT double-buffered GDI/GDI+ rendering
       │  WM_MOUSEMOVE / WM_LBUTTONDOWN hit testing, slider dragging
       │
[TCP Listener Thread] (0.0.0.0:7653)
       │  Blocking accept() loop
       │  Spawns dedicated handle_client() worker thread per incoming connection
       │
[Client Receiver Thread] (Per Active Connection)
       │  Blocking read_frame() loop with 15s socket timeout
       │  Parses 0x01 AUDIO -> RNNoise -> pushes to JitterBuffer
       │  Parses 0x02 VIDEO -> pushes to VideoPipeline latest_jpeg slot
       │  Parses 0x03 HEARTBEAT -> writes echo heartbeat frame
       │  Parses 0x04 CONTROL -> dispatches DSP / mute state updates
       │
[Video Pipeline Worker Thread]
       │  Condvar-driven: wakes when latest_jpeg slot receives a frame
       │  TurboJPEG SIMD decompression to BGRA
       │  Pushes to VirtualCamera driver and PreviewWindow surface
       │
[Audio Sink Thread] (WASAPI System Thread)
       │  High-priority OS real-time callback
       │  Non-blocking try-lock fetch from JitterBuffer
       │  Runs Drift Resampler -> feeds virtual mic
```

---

## 4. Failure Modes & Recovery Matrix

| Failure Event | Detection Point | Handling Mechanism | User Impact |
| :--- | :--- | :--- | :--- |
| **USB Cable Pulled** | `TransportManager` receives `ACTION_USB_STATE = false`. Socket read throws `IOException`. | PC enters `HELD` state for 30s. Android falls back to Wi-Fi (L3) or Bluetooth (L4) via `reconnectDelayMs()`. | Audio/video pauses for < 1.5s, then resumes over wireless link. Virtual mic remains connected. |
| **Wi-Fi Packet Storm / High Jitter** | `JitterBuffer` measures arrival timestamp delta variance $\Delta_{\text{jitter}}$. | Adaptive buffer expands from 40 ms up to 120 ms. If queue exceeds 200 ms, oldest frames dropped. | Audio remains glitch-free; latency slightly increases temporarily, then recovers. |
| **Camera Thermal Throttling** | `PowerManager` triggers `thermalListener`. | `VideoCapture` drops JPEG quality from 80% to 50%, reducing byte volume by 60%. | Frame rate remains 30 fps; subtle reduction in sharpness prevents phone overheating. |
| **PC Sleep / Wake Cycle** | TCP socket write fails with `ConnectionReset`. | Android detects dropped link, emits searching state, initiates exponential backoff reconnect. | Re-pairs automatically within 2 seconds of PC waking up. |
| **Port 7653 Conflict** | `bind_listener()` returns `EADDRINUSE`. | Loop sleeps 2 seconds and retries indefinitely while logging clean diagnostic warning. | Mikey waits for prior process to terminate without crashing. |
