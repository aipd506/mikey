# Mikey

> **Plug in. Tap once. Forget it exists.**

Turn the Android phone in your pocket into a microphone and webcam for your PC — over USB, Bluetooth, or Wi-Fi — with no accounts, no cloud, no setup ritual.

---

## The Idea

Most people own a phone whose camera and microphone are dramatically better than what came with their laptop. Yet the moment they join a video call, they're broadcasting through a $3 plastic pinhole.

Mikey bridges that gap. It is a two-part open-source system:

| Part | What it is |
|---|---|
| **Mikey** (Android app) | A foreground service that captures audio and video and streams it to your PC over the best available link. The UI is two tap targets — mic and camera — and nothing more. |
| **Mikey for PC** (`mikey`) | A lightweight Rust tray binary. Starts with your computer, sits in the system tray, exposes a virtual microphone and a virtual webcam to every app, and waits. You never open it manually. |

The phone is always the client. The PC is always the server. The best connection is chosen automatically, and upgraded or downgraded transparently if conditions change.

---

## Unique Selling Points

### 1 · Four connection levels, automatic and ranked

Mikey doesn't ask you to pick a connection type. It picks the best one available — in real time — and upgrades silently when a better option appears.

| Priority | Level | Audio quality | Video |
|---|---|---|---|
| **1 — USB debugging (ADB)** | Developer options on, tap *Allow* once | PCM 48 kHz lossless | MJPEG up to 1080p30 |
| **2 — USB tethering** | Toggle tethering each time | PCM 48 kHz lossless | MJPEG up to 1080p30 |
| **3 — Bluetooth RFCOMM** | Pair phone & PC once in OS settings | Opus 32–48 kbps | Audio only |
| **4 — Wi-Fi / LAN** | Same network (or phone hotspot) | Opus 96 kbps | MJPEG 720p30, adaptive |

If you plug in a cable mid-call while on Wi-Fi, Mikey upgrades to USB with < 300 ms of audio glitch. If you unplug, it falls back to Wi-Fi within 2 s. The PC's virtual devices never disappear — apps like Zoom don't even blink.

### 2 · Zero accounts, zero cloud, zero telemetry

Everything travels over a direct local TCP socket. No relay server, no analytics, no login, no paid tier, no ads. The PC binary never phones home. The phone app only uses its network permission to reach your own PC. What happens between your phone and your PC stays there.

### 3 · Audio DSP lives on the PC, not the phone

The phone sends raw, unprocessed audio (48 kHz mono, `UNPROCESSED` source). All noise suppression, echo cancellation, and gating runs on the PC side. This means:
- The phone's battery and thermal budget are untouched by DSP.
- Audio quality is identical regardless of which phone model you use.
- The PC's AEC receives a clean reference signal (WASAPI loopback on Windows, monitor source on Linux) before any double-processing can corrupt it.

**Pipeline:** `raw capture → [Opus on L3/L4] → jitter buffer → drift resampler → echo cancellation (SpeexDSP) → noise gate → RNNoise → virtual mic`

### 4 · Make-before-break transport handover

Transport upgrades and downgrades are seamless because Mikey opens the new connection *before* tearing down the old one. Session tokens let the PC identify that the same device is resuming, not joining fresh. The PC holds the session — and outputs silence through the virtual mic rather than letting it vanish — for up to 30 s after any transport drop.

### 5 · Near-zero idle cost

The PC binary idles with all accept loops blocked in the kernel: ~0% CPU, ~15 MB RAM. Threads for camera output, AEC reference capture, and preview exist only while they're needed. The phone doesn't poll continuously — probing is event-driven (USB connect, network interface change, Bluetooth bond change).

### 6 · Privacy by construction

Mic and camera always start **off**. A new device cannot silently inject audio into your PC over Wi-Fi; the PC presents an explicit ask-before-join prompt for unknown devices. Trust is stored as a pairing token (32 random bytes) — there is no persistent identifier that could be tracked.

---

## Architecture

```
┌─────────────────────────── Android: Mikey ────────────────────────────┐
│  MainActivity (Compose) — observes StateFlow, no logic                 │
│  MikeyService (foreground)                                             │
│    ├─ SessionController   idle → connecting → live → ...               │
│    ├─ TransportManager    probes & ranks L1..L4, make-before-break      │
│    │    ├─ AdbTransport       TCP → 127.0.0.1:7653 (adb reverse)       │
│    │    ├─ TetherTransport    TCP over rndis0/usb0/ncm0                 │
│    │    ├─ BluetoothTransport RFCOMM (bonded PC, cached MAC)            │
│    │    └─ WifiTransport      TCP + UDP discovery beacon                │
│    ├─ AudioCapture        AAudio / AudioRecord, 48 kHz mono             │
│    └─ VideoCapture        CameraX → JPEG → frames                      │
└──────────── L1 USB │ L2 Tether │ L3 Wi-Fi │ L4 Bluetooth ─────────────┘
                     ▼                           ▼
┌─────────────────── PC: Mikey for PC (mikey) ──────────────────────────┐
│  Listeners: TCP :7653 · UDP beacon :7654 · RFCOMM · AdbWatcher        │
│  SessionManager   tokens, trust, ask-before-join, 30 s session hold    │
│  Audio pipeline   Opus decode → jitter buf → drift resample → AEC      │
│                   → noise gate → RNNoise → virtual mic                  │
│  Video pipeline   JPEG decode → scale/letterbox → virtual cam           │
│  Tray / UI        icon + menu, notifications, optional preview window   │
└──────────── Virtual mic (VB-Cable / PipeWire) ────────────────────────┘
             Virtual cam (softcam DirectShow / v4l2loopback)
```

**Wire protocol** — binary framing over a reliable stream:
```
Frame = type u8 | len u32 BE | payload (max 4 MiB)
Media: seq u32 | ts u64 µs | codec u8 | rsv u8
0x00 HELLO  0x10 WELCOME  0x11 PENDING  0x12 REJECT
0x01 AUDIO  0x02 VIDEO  0x03 HEARTBEAT  0x04 CONTROL  0x05 BYE
```

**Key architectural decisions:**

| Decision | Choice | Why |
|---|---|---|
| Who initiates | Phone is always the client | One state machine per side; PC just listens |
| PC concurrency | Blocking `std` threads + bounded channels | Predictable, no async runtime (Tokio confined to Linux BT module only) |
| Audio DSP location | PC only | Better AEC, no phone heat/battery cost, uniform quality across all phones |
| Discovery | Custom UDP beacon on port 7654 | No mDNS dependency; interface-pinnable; no extra library |
| Settings ownership | Phone owns stream settings; PC owns PC settings | Phone is the single place the user configures the stream |

---

## Tech Stack

| Side | Language | Key libraries |
|---|---|---|
| Android | Kotlin + Jetpack Compose | AAudio (NDK), CameraX, Opus (JNI) |
| PC | Rust | `cpal`, `opus`, `zune-jpeg`, `softcam` / `v4l2loopback`, `bluer` (Linux BT) |

- Android min SDK: **26** (Android 8.0)
- PC targets: **Windows 10/11** · **Linux** (glibc ≥ 2.31)
- No Electron, no Node, no HTTP server, no database, no cloud SDK — ever.

---

## Roadmap

| Phase | Goal | Status |
|---|---|---|
| **1** — Mic over USB | Phone mic → PC virtual mic via ADB tunnel; prove the audio path end-to-end | 🔨 In progress |
| **2** — Four levels + trust | Full `TransportManager`; real UI; pairing tokens; auto upgrade/downgrade | ⏳ Upcoming |
| **3** — Camera | Phone camera as virtual webcam in Zoom, Teams, Meet, OBS | ⏳ Upcoming |
| **4** — Audio quality | Noise gate, RNNoise, SpeexDSP echo cancellation | ⏳ Upcoming |
| **5** — Hardening & release | Signed APK + Windows installer + Linux packages; v1.0.0 | ⏳ Upcoming |

---

## Repository Layout

```
android/   — Mikey Android app (Kotlin, Jetpack Compose)
pc/        — Mikey for PC tray binary (Rust)
docs/      — Install and troubleshooting guides
vibe/      — Project brain: vision, architecture, rules, playbooks
```

---

## Building

### Android

Requires Android Studio, or JDK 17+ and the Android SDK, plus CMake for libopus (SDK Manager → SDK Tools → CMake, or `brew install cmake`).

```sh
git submodule update --init   # libopus
cd android
./gradlew assembleDebug
# APK → android/app/build/outputs/apk/debug/
```

### PC

Requires Rust stable (1.77+). On Linux, BlueZ dev headers are also required.

```sh
cd pc
cargo build --release
# Binary → pc/target/release/mikey
```

---

## Product Principles

1. **Convenience first, configuration last** — defaults work for 90% of people; settings are one level deeper.
2. **Advanced costs clicks** — power users can find everything; everyone else never sees it.
3. **Lightweight by construction** — small binaries, few dependencies, near-zero idle CPU.
4. **Honest state** — the UI never claims a state that isn't true; if audio stops, the indicator changes within seconds.
5. **Self-healing** — drops, cable pulls, and Wi-Fi hiccups recover without user action.
6. **Private by default** — mic and camera start off; unknown devices require explicit approval.
7. **Remember choices** — settings survive app restarts, reboots, and updates.

---

## What Mikey will never do

Ads · accounts · subscriptions · cloud relay · telemetry · analytics · background data collection · auto-enable mic or camera

---

*Full design documentation — wire protocol, UX specs, coding rules, and operational playbooks — lives in [`vibe/`](vibe/index.md).*
