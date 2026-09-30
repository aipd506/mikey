# Architecture Overview

> This is a contributor-friendly summary. For the full specification, see [`SYSTEM_ARCHITECTURE.md`](./SYSTEM_ARCHITECTURE.md).

## System Topology

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
│  Audio pipeline   Opus decode → RNNoise → jitter buf            │
│                   → drift resample → virtual mic                │
│  Video pipeline   JPEG decode → scale/letterbox → virtual cam   │
│  Tray / UI        icon + flyout, notifications, preview window  │
└──────── Virtual mic (VB-Cable / PipeWire) ───────────────────────┘
          Virtual cam (softcam / v4l2loopback)
```

## Key Concepts

### Connection Levels

Mikey automatically selects the best available transport:

| Priority | Level | Medium | Audio | Video |
|----------|-------|--------|-------|-------|
| 1 | USB Debugging | ADB reverse | PCM 48 kHz lossless | MJPEG ≤1080p30 |
| 2 | USB Tethering | TCP over USB NIC | PCM 48 kHz lossless | MJPEG ≤1080p30 |
| 3 | Wi-Fi / LAN | TCP + UDP discovery | Opus 96 kbps | MJPEG 720p30 |
| 4 | Bluetooth | RFCOMM | Opus 48 kbps | None |

Upgrades are **make-before-break**: the new connection opens before the old one closes. Downgrades happen within 2 seconds.

### Wire Protocol

Binary framing over a reliable stream:
```
Frame = type (u8) | length (u32 BE) | payload (max 4 MiB)
Media = seq (u32 BE) | timestamp (u64 BE µs) | codec (u8) | reserved (u8) | data
```

Frame types: `0x00` HELLO, `0x10` WELCOME, `0x11` PENDING, `0x12` REJECT, `0x01` AUDIO, `0x02` VIDEO, `0x03` HEARTBEAT, `0x04` CONTROL, `0x05` BYE.

### Threading Model

- **Android:** Coroutines for service orchestration; audio and camera callbacks on their own threads.
- **PC:** Blocking `std` threads + bounded channels. No async runtime except Tokio in `bt.rs` on Linux (required by `bluer`). Every socket has a timeout. Every channel is bounded.

### Trust Model

- Phone is always the client. PC is always the server.
- Trust-on-first-use: PC prompts "Allow [device name]?" for unknown devices.
- Trust stored as a random 32-byte pairing token. No persistent identifiers.
- Mic and camera always start OFF. Never auto-enable from background.

## Where to Learn More

| Topic | Document |
|-------|----------|
| Full system design | [SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md) |
| Transport details | [TRANSPORTS_AND_NETWORKING.md](./TRANSPORTS_AND_NETWORKING.md) |
| Session lifecycle | [SESSIONS_AND_TRUST.md](./SESSIONS_AND_TRUST.md) |
| Wire protocol spec | [WIRE_PROTOCOL.md](./WIRE_PROTOCOL.md) |
| Audio/video pipelines | [AUDIO_PIPELINE.md](./AUDIO_PIPELINE.md) & [VIDEO_PIPELINE.md](./VIDEO_PIPELINE.md) |
| Performance & budgets | [PERFORMANCE_AND_REALTIME_BUDGETS.md](./PERFORMANCE_AND_REALTIME_BUDGETS.md) |
| Roadmap & Test matrix | [ROADMAP_AND_TEST_MATRIX.md](./ROADMAP_AND_TEST_MATRIX.md) |
| Developer playbooks | [DEVELOPER_PLAYBOOKS_AND_SKILLS.md](./DEVELOPER_PLAYBOOKS_AND_SKILLS.md) |
