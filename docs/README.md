# Mikey Documentation Index

Welcome to the comprehensive technical documentation for **Mikey**.

Mikey turns an Android phone into a high-performance, ultra-low-latency microphone and webcam for a PC over four automatic connection levels:
`Level 1 (USB Debugging / adb reverse)` > `Level 2 (USB Tethering)` > `Level 3 (Wi-Fi LAN)` > `Level 4 (Bluetooth RFCOMM)`.

The project is built on strict **lean engineering principles**: zero accounts, zero cloud dependencies, zero electron/webviews, zero async runtime bloat, and pure functional OLED-black styling.

---

## 1. Documentation Index

### 1.1 Getting Started & Deployment
| Guide | Purpose |
| :--- | :--- |
| **[INSTALL_PC.md](./INSTALL_PC.md)** | Step-by-step setup for Windows 10/11 and Linux hosts (VB-Audio Cable, softcam, permissions). |
| **[INSTALL_ANDROID.md](./INSTALL_ANDROID.md)** | Android APK installation, USB debugging authorization, and first-launch steps. |
| **[TROUBLESHOOTING.md](./TROUBLESHOOTING.md)** | Diagnosing firewall isolation, ADB port conflicts, OEM battery killers, and Bluetooth RFCOMM. |
| **[CHANGELOG.md](./CHANGELOG.md)** | Version evolution history and release notes. |

### 1.2 Architecture & Protocols
| Document | Primary Focus | Key Source Files |
| :--- | :--- | :--- |
| **[PRODUCT_VISION_AND_SCOPE.md](./PRODUCT_VISION_AND_SCOPE.md)** | Motto, core vision, empathy personas, 7 product principles, v1.0 scope boundaries, user journeys, risks R1–R12, and 11 recorded decisions. | `README.md` |
| **[SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md)** | End-to-end system topology, component responsibilities, threading models, failure matrix, and platform boundaries. | `MainActivity.kt`, `MikeyService.kt`, `main.rs`, `lib.rs` |
| **[ARCHITECTURE.md](./ARCHITECTURE.md)** | Quick contributor summary of system topology, connection levels, and threading. | `docs/ARCHITECTURE.md` |
| **[TRANSPORTS_AND_NETWORKING.md](./TRANSPORTS_AND_NETWORKING.md)** | The 4 connection levels, make-before-break upgrades, UDP discovery beacon (`:7654`), subnet pinning, and anti-flap timers. | `TransportManager.kt`, `Discovery.kt`, `adb.rs`, `beacon.rs`, `tcp/` |
| **[WIRE_PROTOCOL.md](./WIRE_PROTOCOL.md)** | Complete binary wire framing, packet opcodes (`0x00`–`0x12`), 14-byte `MediaHeader`, JSON control schemas, and state machine. | `protocol/Frame.kt`, `protocol/Messages.kt`, `pc/src/protocol/` |
| **[AUDIO_PIPELINE.md](./AUDIO_PIPELINE.md)** | AAudio/AudioRecord capture, Opus JNI encoder, adaptive jitter buffer, drift resampler (±0.2%), auto-normalizer (unity gain vocal passthrough), and RNNoise DSP. | `AudioCapture.kt`, `OpusEncoder.kt`, `audio/pipeline/`, `audio/dsp/` |
| **[VIDEO_PIPELINE.md](./VIDEO_PIPELINE.md)** | CameraX capture (`KEEP_ONLY_LATEST`), 20° gravity orientation hysteresis, NV21 conversion, adaptive JPEG, DirectShow softcam, and preview. | `VideoCapture.kt`, `Nv21.kt`, `video/pipeline.rs`, `video/vcam/` |
| **[SESSIONS_AND_TRUST.md](./SESSIONS_AND_TRUST.md)** | Trust-on-first-use (TOFU), 32-byte pairing tokens, ask-before-join modals, 30s session hold, and single-device policy. | `SessionController.kt`, `session/mod.rs`, `session/handshake.rs` |

### 1.3 Platform Engineering & Quality
| Document | Primary Focus | Key Source Files |
| :--- | :--- | :--- |
| **[ANDROID_ARCHITECTURE_AND_STYLE.md](./ANDROID_ARCHITECTURE_AND_STYLE.md)** | Kotlin design patterns, reactive Compose observing `MikeyService`, foreground service lifecycle, real-time priority, JNI, and anti-slop rules. | `android/app/src/main/` |
| **[PC_ARCHITECTURE_AND_STYLE.md](./PC_ARCHITECTURE_AND_STYLE.md)** | Rust systems architecture, std threads + bounded channels, strict no-async rule, silent background tray entry, and Clippy rigor. | `pc/src/` |
| **[UI_AND_DESIGN_LANGUAGE.md](./UI_AND_DESIGN_LANGUAGE.md)** | Pure functional aesthetic, OLED pure black palette (`#000000`), Geist typography, icon metrics, and double-buffered Win32 GDI flyout engine. | `Palette.kt`, `Controls.kt`, `flyout/render.rs`, `flyout/palette.rs` |
| **[PERFORMANCE_AND_REALTIME_BUDGETS.md](./PERFORMANCE_AND_REALTIME_BUDGETS.md)** | Hard latency budgets (≤20ms USB, ≤40ms Wi-Fi), memory/CPU limits, buffer bounds, timeouts, and verification steps. | `constants.rs`, `docs/PERFORMANCE_AND_REALTIME_BUDGETS.md` |
| **[ROADMAP_AND_TEST_MATRIX.md](./ROADMAP_AND_TEST_MATRIX.md)** | Checklists for Phases 1–5 (Phases 1–4 complete), Phase 5 release criteria, and real-device test matrix across OEMs, OSes, and apps. | `ROADMAP_AND_TEST_MATRIX.md` |
| **[DEVELOPER_PLAYBOOKS_AND_SKILLS.md](./DEVELOPER_PLAYBOOKS_AND_SKILLS.md)** | Step-by-step developer playbooks (build Android, add dependency, change protocol, close phase), coding rules 1–18, and pre-commit hook. | `AGENTS.md` |

---

## 2. At a Glance: Key Protocol & Runtime Parameters

| Parameter | Value | Location / Constant | Rationale / Role |
| :--- | :---: | :--- | :--- |
| **TCP Control & Data Port** | `7653` | `PORT_TCP` (`protocol/mod.rs`) | Main bidirectional control & media stream port |
| **UDP Discovery Port** | `7654` | `PORT_BEACON` (`protocol/mod.rs`) | LAN and tether subnet broadcast discovery responder |
| **Wire Protocol Version** | `2` | `PROTO_VERSION` (`protocol/mod.rs`) | Major protocol revision handshake identifier |
| **Max Frame Payload** | `4 MiB` | `MAX_PAYLOAD_LEN` (`protocol/mod.rs`) | Memory exhaustion safety ceiling |
| **Binary Media Header Length** | `14 bytes` | `MEDIA_HEADER_LEN` (`protocol/mod.rs`) | `seq u32 BE` + `capture_ts u64 BE` + `codec u8` + `rsv u8` |
| **Audio Sample Rate** | `48,000 Hz` | `SAMPLE_RATE` (`AudioCapture.kt` / `constants.rs`) | Broadcast studio standard audio rate |
| **Audio Frame Duration** | `10 ms` | `FRAME_SAMPLES = 480` | 480 samples = 960 bytes per raw PCM frame |
| **Hard Latency Drop Cap** | `200 ms` | `MAX_LATENCY_MS` (`constants.rs`) | Samples exceeding 200 ms dropped to prevent creeping latency |
| **Clock Drift Ratio Limit** | `±0.2%` | `MAX_DRIFT_RATIO = 0.002` | Linear resampling clamp aligning crystal oscillators |
| **Speech Target Loudness** | `1.0× (Unity)` | `MIN_AUTO_GAIN = 1.0, MAX_AUTO_GAIN = 1.0` | Transparent vocal delivery without voice ducking |
| **Heartbeat Interval** | `5,000 ms` | `HEARTBEAT_MS` (`SessionController.kt`) | Bidirectional link liveness ping/pong |
| **Socket Timeout** | `15,000 ms`| `LINK_TIMEOUT_MS` / `SOCKET_TIMEOUT` | Three missed heartbeats trigger disconnect |
| **Session Hold Duration** | `30,000 ms`| `SESSION_HOLD_DURATION` (`types.rs`) | Virtual device handles held during transport switches |
| **Anti-Flap Probation** | `10,000 ms`| `DEAD_MS` (`TransportManager.kt`) | Lockout window on failed transport levels |

---

## 3. Repository Structure Overview

```text
mikey/
├── android/                  # Android Client (Kotlin, Jetpack Compose, CameraX, AAudio, NDK)
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── cpp/          # JNI bindings: aaudio_jni.c, opus_jni.c
│   │   │   ├── java/com/mikey/
│   │   │   │   ├── media/    # AudioCapture, VideoCapture, LevelMeter, Nv21, OpusEncoder
│   │   │   │   ├── protocol/ # Frame, FrameType, MediaHeader, Control, Messages
│   │   │   │   ├── service/  # MikeyService (foreground), SessionController, Notifier
│   │   │   │   ├── settings/ # Settings, PairedPc, SharedPreferences
│   │   │   │   ├── transport/# TransportManager, TcpTransport, BluetoothTransport, Discovery
│   │   │   │   ├── ui/       # MainScreen, Controls, SettingsSheet, Palette, Type, Glyphs
│   │   │   │   └── MainActivity.kt
│   │   │   └── res/          # Drawables, mipmaps, values
│   │   └── build.gradle.kts
│   └── settings.gradle.kts
├── pc/                       # PC Server & Companion (Rust, Win32 GDI, WASAPI, DirectShow)
│   ├── Cargo.toml
│   └── src/
│       ├── audio/            # JitterBuffer, drift resampler, normalizer, RNNoise DSP, audio sinks
│       ├── config/           # TOML configuration, pairing tokens, level toggles
│       ├── flyout/           # Native double-buffered Win32 GDI/GDI+ tray flyout UI
│       ├── protocol/         # Binary framing parser, frame serializers, payload structs
│       ├── session/          # SessionManager, TOFU pairing, handshake evaluator, session hold
│       ├── transport/        # TCP listener, UDP beacon responder, ADB watcher, BT RFCOMM
│       ├── video/            # Background JPEG decoder, virtual camera driver (softcam), preview window
│       ├── autostart.rs      # Windows Run registry key management
│       ├── instance.rs       # Single-instance mutex enforcement
│       ├── launch.rs         # Silent background startup, FreeConsole, busy cursor removal
│       ├── main.rs           # Entry point and multi-thread coordinator
│       └── tray.rs           # Windows Shell_NotifyIcon message pump and click dispatcher
├── docs/                     # Comprehensive architecture, protocols, and guides (This folder)
├── scripts/                  # Build and release automation scripts
└── AGENTS.md                 # Agent instructions and project source-of-truth pointers
```

---

## 4. Core Invariants & Rules

1. **Role Invariant**: The phone is **always the client**; the PC is **always the server**. The phone initiates all connections and transmits media; the PC handles processing, virtual device feeding, and remote control.
2. **State & Privacy Invariant**: Mic and camera always initialize in the **OFF** state. They never turn on automatically when connecting, booting, or switching transports.
3. **Threading Invariant**: PC uses **blocking std threads + bounded channels**. Async runtimes (Tokio) are strictly forbidden everywhere except Linux Bluetooth (`pc/src/transport/bt/server.rs` using `bluer`).
4. **Real-time Safety**: Audio and video capture paths must never block on network I/O or mutexes. Sockets have timeouts, channels have strict bounds, and old frames are dropped under pressure (`KEEP_ONLY_LATEST`).
5. **Aesthetic Invariant**: No gradients, no glassmorphism, no drop shadows, and no decorative spring animations. Pure functional utility with OLED black (`#000000`).
