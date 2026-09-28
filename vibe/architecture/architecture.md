# Architecture

## System diagram

```
┌──────────────────────────────── Android: Mikey ────────────────────────────────┐
│                                                                                │
│  MainActivity (Compose)  ── remote control only, no logic ──┐                  │
│    SplitScreen · FlipButton · Chevron/Drawer · StatusDot    │ observes StateFlow│
│                                                             ▼                  │
│  MikeyService (foreground service — owns everything below)                     │
│    ├─ SessionController      state machine: idle → connecting → live → ...     │
│    ├─ TransportManager       probes & ranks Level 1..4, upgrades/downgrades    │
│    │    ├─ AdbTransport        TCP → 127.0.0.1:7653   (tunnel via adb reverse) │
│    │    ├─ TetherTransport     TCP over rndis0/usb0/ncm0 interface             │
│    │    ├─ BluetoothTransport  RFCOMM socket, Mikey service UUID               │
│    │    └─ WifiTransport       TCP over Wi-Fi / hotspot interface              │
│    ├─ Discovery              UDP beacon on a chosen interface (L2, L3)         │
│    ├─ AudioCapture           raw 48 kHz mono (no phone DSP) → [Opus] → frames  │
│    ├─ VideoCapture           CameraX ImageAnalysis → JPEG → frames             │
│    ├─ Settings               SharedPreferences, pushed to PC on connect/change │
│    └─ Notifier               persistent notification + actions                 │
└────────────────────────────────────────────────────────────────────────────────┘
          │ L1 USB (adb reverse)   │ L2 USB tether   │ L3 Wi-Fi       │ L4 Bluetooth
          ▼                        ▼                 ▼                ▼
┌──────────────────────────────── PC: Mikey for PC (mikey) ─────────────────────┐
│                                                                                │
│  Listeners                                                                     │
│    TCP  0.0.0.0:7653        (L1 arrives on 127.0.0.1, L2/L3 on LAN IPs)        │
│    UDP  0.0.0.0:7654        discovery beacon responder                         │
│    RFCOMM server            SDP record with Mikey UUID (L4)                    │
│    AdbWatcher               tracks devices, runs `adb reverse`, opens app      │
│                                                                                │
│  SessionManager             auth (HELLO/token), trust, ask-before-join,        │
│                             one active device, transport handover              │
│                                                                                │
│  Audio pipeline             demux → [Opus decode] → jitter buffer + drift      │
│                             resampler → [noise gate] → [RNNoise] → [AEC]       │
│                             → virtual mic                                      │
│  Video pipeline             demux → JPEG decode → scale/letterbox → virtual cam│
│                                                     └→ preview window (opt.)   │
│  Tray / Flyout UI           tray icon + companion flyout dialog,               │
│                             notifications, preview (embedded/pop-out)          │
│  Config                     config.toml, trusted devices, logs                 │
└────────────────────────────────────────────────────────────────────────────────┘
          │                                   │
          ▼                                   ▼
   Virtual mic                          Virtual camera
   Win: VB-Cable ("CABLE Input")        Win: softcam (DirectShow)  [MF VCam on Win11 later]
   Lin: PipeWire/Pulse virtual source   Lin: v4l2loopback /dev/videoN
```

## Key architectural decisions

| Decision | Choice | Reason |
|---|---|---|
| Who initiates | **Phone is always the client**, PC is always the server | One direction, one state machine per side. PC just listens. |
| PC process model | **Single per-user process**, thread per concern | Tray + audio context need the user session. No IPC, no service. |
| Transport abstraction | Every level yields a **reliable byte stream** (TCP or RFCOMM) | Same framing, same session logic on all four levels. |
| Media profile per level | Chosen by the phone, declared in `HELLO` | USB = raw PCM + high-quality video; BT = Opus 32–48 kbps, no video; Wi‑Fi = Opus 96 kbps (or raw PCM if enabled) + adaptive video. |
| Settings ownership | The phone keeps the stream/audio settings; either side can change them and CONTROL keeps both in sync. PC-only settings stay on the PC | One saved copy, on the phone, and the same controls on both sides. |
| Where audio DSP runs | **PC only** (noise suppression, gate, echo cancellation) | Better AEC with raw input, no phone heat/battery cost, same quality on every phone |
| Concurrency (PC) | Blocking std threads + bounded channels | Small, predictable, no async runtime. One exception: the Linux Bluetooth module ([tech-stack.md](tech-stack.md)). |
| Concurrency (phone) | Dedicated threads for capture/network; Kotlin coroutines + `StateFlow` only for UI state | Real-time audio must not share a dispatcher with UI work. |

## PC threads

| Thread | Job |
|---|---|
| `main` | Tray & flyout event loop (main thread), flyout mini-app & preview rendering |
| `tcp-accept` | Accepts TCP connections, hands each to a session reader thread |
| `udp-beacon` | Answers discovery probes |
| `bt-accept` | RFCOMM server accept loop |
| `adb-watch` | `adb track-devices`; on authorized device: `adb reverse`, optionally launch app |
| `session-rx` (per connection) | Reads frames, demuxes into audio/video channels |
| `audio-out` | Jitter buffer, drift resampler, DSP chain, writes to virtual mic in a real-time callback |
| `aec-ref` | Captures speaker loopback (only when AEC on) |
| `video-out` | JPEG decode, scale, push to virtual cam (only while camera on) |
| `watchdog` | Heartbeat timeouts, pending-approval timeouts |

Threads for camera, AEC and preview exist **only while needed**. Idle = accept loops blocked in the kernel = ~0% CPU.
