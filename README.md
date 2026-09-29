<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/Mikey-Use_your_phone_as_a_mic_and_webcam-white?style=for-the-badge&labelColor=000000">
    <img alt="Mikey" src="https://img.shields.io/badge/Mikey-Use_your_phone_as_a_mic_and_webcam-black?style=for-the-badge&labelColor=ffffff">
  </picture>
</p>

<h1 align="center">Mikey</h1>

<p align="center">
  <strong>Plug in. Tap once. Forget it exists.</strong>
</p>

<p align="center">
  Turn the Android phone in your pocket into a microphone and webcam for your PC - over USB, Bluetooth, or Wi-Fi - with no accounts, no cloud, and no setup ritual.
</p>

<p align="center">
  <a href="https://github.com/diveshpatil9104/mikey/releases"><img src="https://img.shields.io/github/v/release/diveshpatil9104/mikey?style=for-the-badge&color=D71921&label=Download" alt="Download"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License: MIT"></a>
  <a href=".github/CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-Welcome-brightgreen?style=for-the-badge" alt="PRs Welcome"></a>
  <a href="https://github.com/diveshpatil9104/mikey/discussions"><img src="https://img.shields.io/badge/Discuss-GitHub-blue?style=for-the-badge&logo=github" alt="Discussions"></a>
  <a href="https://github.com/diveshpatil9104/mikey/stargazers"><img src="https://img.shields.io/github/stars/diveshpatil9104/mikey?style=for-the-badge&color=FFD60A&logo=github" alt="Stars"></a>
</p>

---

## Why Mikey?

Most people own a phone whose camera and microphone are **dramatically better** than what came with their laptop. Yet the moment they join a video call, they're broadcasting through a $3 plastic pinhole.

Mikey bridges that gap. It is a two-part open-source system:

| Component | What it does |
|-----------|-------------|
| **Mikey** (Android app) | A foreground service that captures audio and video and streams it to your PC over the best available link. The UI is two tap targets - mic and camera - and nothing more. |
| **Mikey for PC** (`mikey`) | A lightweight Rust tray binary. Starts with your computer, sits in the system tray, exposes a virtual microphone and a virtual webcam to every app, and waits. You never open it manually. |

The phone is always the client. The PC is always the server. The best connection is chosen automatically, and upgraded transparently if conditions change.

---

## Quick Start

### 1. Install on your PC

<table>
<tr>
<td><b>Windows 10/11</b></td>
<td>

Download [`Mikey-Setup.exe`](https://github.com/diveshpatil9104/mikey/releases) from Releases and run the installer.
Install [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) (free) for the virtual microphone.

</td>
</tr>
<tr>
<td><b>Linux</b></td>
<td>

```bash
# Debian/Ubuntu
sudo dpkg -i mikey_*_amd64.deb
# Or use the AppImage
chmod +x Mikey-*.AppImage && ./Mikey-*.AppImage
```

</td>
</tr>
<tr>
<td><b>From source</b></td>
<td>

```bash
git clone https://github.com/diveshpatil9104/mikey.git
cd mikey/pc && cargo build --release
cargo run --release
```

</td>
</tr>
</table>

### 2. Install on your phone

Download the APK from [Releases](https://github.com/diveshpatil9104/mikey/releases) or [build from source](docs/INSTALL_ANDROID.md).

### 3. Connect and go

Plug in your phone, open Mikey, tap the mic. That's it - Zoom, Teams, Meet, Discord, and OBS will see your phone's mic and camera.

---

## What Makes Mikey Different

<table>
<tr><td width="320"><b>Automatic Multi-Level Transport</b></td><td>Mikey selects the best physical connection in real time and upgrades silently as faster physical links become available.</td></tr>
<tr><td><b>Local-First Architecture</b></td><td>All audio and video travels over a direct physical or local link. No cloud relay servers, no mandatory accounts, and no audio/video telemetry.</td></tr>
<tr><td><b>PC-Side Audio DSP</b></td><td>The phone sends raw capture frames. Noise suppression, echo cancellation, and gating run on the PC, saving mobile battery and ensuring uniform output across hardware.</td></tr>
<tr><td><b>Seamless Transport Handover</b></td><td>Upgrades and downgrades are make-before-break. Connecting a USB cable mid-call transitions seamlessly without interrupting video call applications.</td></tr>
<tr><td><b>Minimal Idle Footprint</b></td><td>The PC binary idles blocked in kernel accept loops (~0% CPU, ~15 MB RAM). Media processing threads exist only during active streaming.</td></tr>
<tr><td><b>Privacy by Construction</b></td><td>Microphone and camera always initialize in the off state. Unknown PC endpoints require explicit user approval on the phone.</td></tr>
</table>

---

## Connection Levels

| Priority | Level | How | Audio | Video |
|:--------:|-------|-----|-------|-------|
| **1** | USB Debugging (ADB) | Developer options on, tap *Allow* once | PCM 48 kHz lossless | MJPEG ≤1080p30 |
| **2** | USB Tethering | Toggle tethering in Android settings | PCM 48 kHz lossless | MJPEG ≤1080p30 |
| **3** | Wi-Fi / LAN | Same network or phone hotspot | Opus 96 kbps | MJPEG 720p30 |
| **4** | Bluetooth RFCOMM | Pair phone & PC once in OS settings | Opus 48 kbps | Audio only |

> If you plug in a cable mid-call while on Wi-Fi, Mikey upgrades to USB with <300 ms of audio glitch. If you unplug, it falls back to Wi-Fi within 2 s. The PC's virtual devices never disappear - apps like Zoom don't even blink.

---

## Architecture

```
┌──────────────────────── Android: Mikey ──────────────────────────┐
│  MainActivity (Compose) - observes StateFlow, no logic           │
│  MikeyService (foreground)                                       │
│    ├─ SessionController    idle → connecting → live → ...        │
│    ├─ TransportManager     probes & ranks L1..L4                 │
│    │    ├─ AdbTransport       TCP → 127.0.0.1:7653              │
│    │    ├─ TetherTransport    TCP over rndis0/usb0/ncm0         │
│    │    ├─ BluetoothTransport RFCOMM (bonded, cached MAC)       │
│    │    └─ WifiTransport      TCP + UDP beacon                  │
│    ├─ AudioCapture         AAudio / AudioRecord, 48 kHz mono    │
│    └─ VideoCapture         CameraX → JPEG → frames             │
└─────────── L1 USB │ L2 Tether │ L3 Wi-Fi │ L4 Bluetooth ───────┘
                    ▼                           ▼
┌──────────────────── PC: Mikey for PC (mikey) ────────────────────┐
│  Listeners: TCP :7653 · UDP beacon :7654 · RFCOMM · AdbWatcher  │
│  SessionManager   tokens, trust, ask-before-join, session hold   │
│  Audio pipeline   Opus decode → jitter buf → drift resample     │
│                   → AEC → noise gate → RNNoise → virtual mic    │
│  Video pipeline   JPEG decode → scale/letterbox → virtual cam   │
│  Tray / UI        icon + flyout, notifications, preview window  │
└──────── Virtual mic (VB-Cable / PipeWire) ───────────────────────┘
          Virtual cam (softcam / v4l2loopback)
```

<details>
<summary><b>Wire Protocol</b></summary>

Binary framing over a reliable stream:
```
Frame = type (u8) | length (u32 BE) | payload (max 4 MiB)
Media = seq (u32 BE) | timestamp (u64 BE µs) | codec (u8) | reserved (u8) | data
```

| Type | ID | Purpose |
|------|----|---------|
| HELLO | `0x00` | Client introduction |
| WELCOME | `0x10` | Server accepts client |
| PENDING | `0x11` | Server waiting for user approval |
| REJECT | `0x12` | Server denies client |
| AUDIO | `0x01` | Audio frame |
| VIDEO | `0x02` | Video frame |
| HEARTBEAT | `0x03` | Liveness check |
| CONTROL | `0x04` | Settings sync |
| BYE | `0x05` | Graceful disconnect |

Ports: TCP `:7653` (control & data), UDP `:7654` (discovery beacon).

Full specification: [`docs/WIRE_PROTOCOL.md`](docs/WIRE_PROTOCOL.md)

</details>

<details>
<summary><b>Audio Pipeline (PC)</b></summary>

```
raw capture (phone) → [Opus encode on L3/L4] → TCP/RFCOMM
    → Opus decode → adaptive jitter buffer → drift resampler
    → SpeexDSP echo cancellation (WASAPI/PipeWire loopback reference)
    → noise gate → RNNoise → virtual microphone
```

- Jitter buffer: adaptive, capped at 200 ms
- Drift resampler: compensates for clock mismatch between phone and PC
- AEC: SpeexDSP with delay hint from jitter buffer, loopback reference from system speakers
- RNNoise: ML-based noise suppression (`nnnoiseless` Rust port)

</details>

<details>
<summary><b>Key Architectural Decisions</b></summary>

| Decision | Choice | Why |
|----------|--------|-----|
| Who initiates | Phone is always the client | One state machine per side; PC just listens |
| PC concurrency | Blocking `std` threads + bounded channels | Predictable, no async runtime (Tokio confined to Linux BT only) |
| Audio DSP location | PC only | Better AEC, no phone heat/battery cost, uniform quality |
| Discovery | Custom UDP beacon on port 7654 | No mDNS dependency; interface-pinnable; no extra library |
| Settings ownership | Phone owns stream settings; PC owns PC settings | Phone is the single place the user configures the stream |

</details>

---

## Tech Stack

| Side | Language | Key Libraries |
|------|----------|--------------|
| **Android** | Kotlin + Jetpack Compose | AAudio (NDK), CameraX, Opus (JNI) |
| **PC** | Rust | `cpal`, `opus-decoder`, `zune-jpeg`, `softcam` / `v4l2loopback`, `bluer` (Linux BT) |

- Android min SDK: **26** (Android 8.0)
- PC targets: **Windows 10/11** · **Linux** (glibc ≥ 2.31)
- No Electron, no Node, no HTTP server, no database, no cloud SDK - ever.

---

## Roadmap

| Phase | Goal | Status |
|:-----:|------|:------:|
| **1** | Mic over USB - prove the audio path end-to-end | Done |
| **2** | Four connection levels, trust, real phone UI | Done |
| **3** | Camera - phone as virtual webcam | Done |
| **4** | Audio quality - noise gate, RNNoise, AEC | Done |
| **5** | Hardening & release - signed builds, installers, docs | In Progress |

Milestones and hardware test tracking are documented in the [Roadmap & Test Matrix](docs/ROADMAP_AND_TEST_MATRIX.md).

---

## Repository Layout

```
mikey/
├── android/           Mikey Android app (Kotlin, Jetpack Compose)
├── pc/                Mikey for PC tray binary (Rust)
├── docs/              Architecture, protocol specs, pipelines, install, and troubleshooting
├── .github/           Issue/PR templates, contributing guidelines, security policy
├── AGENTS.md          Agent instructions and project source-of-truth pointers
└── LICENSE            MIT License
```

---

## Contributing

We welcome contributions from everyone - whether you're fixing a typo or building a new transport layer. See [**CONTRIBUTING.md**](.github/CONTRIBUTING.md) for the full guide.

**Quick version:**

1. Fork → `git clone` → `git checkout -b feat/your-idea`
2. Read the relevant [`docs/`](docs/README.md) for your area
3. Make your changes, run the linter, include proof of change
4. Open a PR using the template

> **New to open source?** Look for issues labeled [`good first issue`](https://github.com/diveshpatil9104/mikey/labels/good%20first%20issue).

<details>
<summary><b>Development Setup</b></summary>

**Android:**
```bash
git submodule update --init    # libopus
cd android
./gradlew assembleDebug
# APK → android/app/build/outputs/apk/debug/
```

**PC (Rust):**
```bash
cd pc
cargo build --release
cargo run --release
```

**Verify before submitting:**
```bash
# Android
cd android && ./gradlew testDebugUnitTest lintDebug

# PC
cd pc && cargo fmt --check && cargo clippy -- -D warnings
```

</details>

<details>
<summary><b>Proof of Change (Required)</b></summary>

Every PR must include evidence that the change works:

| Evidence | When to use |
|----------|-------------|
| Screenshot (before/after) | UI changes |
| Screen recording / GIF | Interactive features, connection flows |
| Test output | Automated test results |
| Log output | Relevant log lines showing correct behavior |
| Build output | Proof the project compiles |

PRs without proof of change will not be reviewed. See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for details.

</details>

<details>
<summary><b>Commit Messages</b></summary>

[Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat(android): stream mic to pc over tcp
fix(pc): skip unknown frame types in protocol parser
docs: add contributing guide and PR template
```

Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`

</details>

---

## Product Principles

1. **Convenience first, configuration last** - defaults work for 90% of people
2. **Advanced costs clicks** - power users can find everything; everyone else never sees it
3. **Lightweight by construction** - small binaries, few dependencies, near-zero idle CPU
4. **Honest state** - the UI never claims a state that isn't true
5. **Self-healing** - drops, cable pulls, and Wi-Fi hiccups recover without user action
6. **Private by default** - mic and camera start off; unknown devices require approval
7. **Remember choices** - settings survive restarts, reboots, and updates

---

## Core Guarantees

Mikey is engineered around strict boundaries:

- **No Mandatory Accounts**: Operational immediately without accounts, sign-ups, or login credentials.
- **No Cloud Media Relay**: All audio and video streams remain strictly on the local physical or network link.
- **No Audio/Video Telemetry**: Voice and camera data are never stored, analyzed, or transmitted to remote servers.
- **No Silent Capture**: Mic and camera always initialize in the off state; unknown PCs require explicit user confirmation.
- **No Subscription Paywalls**: Core phone-to-PC webcam and microphone functionality is permanently accessible.

---

## Documentation

| Document | Description |
|----------|-------------|
| [Install - Android](docs/INSTALL_ANDROID.md) | Sideload APK or build from source |
| [Install - PC](docs/INSTALL_PC.md) | Windows installer, Linux packages, or build from source |
| [Troubleshooting](docs/TROUBLESHOOTING.md) | Common connection, audio, and camera resolutions |
| [Architecture](docs/ARCHITECTURE.md) | High-level system design overview |
| [Contributing](.github/CONTRIBUTING.md) | Contribution standards and workflow |
| [Code of Conduct](.github/CODE_OF_CONDUCT.md) | Community participation standards |
| [Security](.github/SECURITY.md) | Vulnerability disclosure policy |
| [Changelog](docs/CHANGELOG.md) | Release and development history |

<details>
<summary><b>Full Design & Architecture Documentation (docs/)</b></summary>

The [`docs/`](docs/README.md) directory is the project source of truth for all architectural and implementation decisions.

**Core - Product & UX:**
[Vision & Scope](docs/PRODUCT_VISION_AND_SCOPE.md) · [Roadmap & Test Matrix](docs/ROADMAP_AND_TEST_MATRIX.md) · [Developer Playbooks](docs/DEVELOPER_PLAYBOOKS_AND_SKILLS.md)

**Architecture - System Design:**
[System Architecture](docs/SYSTEM_ARCHITECTURE.md) · [Connection Levels](docs/TRANSPORTS_AND_NETWORKING.md) · [Sessions & Trust](docs/SESSIONS_AND_TRUST.md) · [Wire Protocol](docs/WIRE_PROTOCOL.md) · [Audio Pipeline](docs/AUDIO_PIPELINE.md) · [Video Pipeline](docs/VIDEO_PIPELINE.md) · [Android Architecture](docs/ANDROID_ARCHITECTURE_AND_STYLE.md) · [PC Architecture](docs/PC_ARCHITECTURE_AND_STYLE.md)

**Rules - Quality Standards:**
[Design Language](docs/UI_AND_DESIGN_LANGUAGE.md) · [Performance Budgets](docs/PERFORMANCE_AND_REALTIME_BUDGETS.md) · [Developer Playbooks](docs/DEVELOPER_PLAYBOOKS_AND_SKILLS.md)

</details>

---

## License

[MIT License](LICENSE) - Copyright © 2026 Divesh Patil

---

<p align="center">
  <sub>Made with conviction that your phone deserves better than sitting in your pocket during calls.</sub>
</p>
