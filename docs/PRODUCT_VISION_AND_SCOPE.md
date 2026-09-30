# Product Vision, Principles & Scope

> **Motto**: *Plug in. Tap once. Forget it exists.*

Mikey is an open-source, two-part system-a native Android app and a lightweight PC companion tray app-that turns an Android smartphone into a high-performance microphone and webcam for a PC over four automatic connection levels: USB debugging, USB tethering, Wi-Fi, and Bluetooth.

---

## 1. Vision & Core Promise

Mikey feels less like software you operate and more like a physical hardware cable: you connect it, it works, and you stop thinking about it.

| Component | Responsibility & Interface |
| :--- | :--- |
| **Mikey (Android Client)** | Native Android app. Two split interactive halves: camera on top, mic on the bottom. One chevron drawer for settings. |
| **Mikey for PC (`mikey`)** | Lightweight native Rust companion executable. Autostarts with the computer, lives in the notification tray, and exposes virtual microphone and webcam endpoints to all meeting applications. |

### 1.1 Empathy: Who We Are Building For
1. **The Student or Remote Worker**: Has a laptop with a broken or subpar mic/webcam and cannot justify buying expensive hardware. Their phone’s camera sensor and microphone array are dramatically superior to low-end external webcams.
2. **The Person in a Hurry**: Their video call begins in 40 seconds. They cannot afford to open two applications, scan QR codes, configure IP addresses, or troubleshoot connection dialogues.
3. **The User Burned by Unreliable Tools**: Has used utilities that silently froze audio while both sides reported "Connected", discovering the failure only when colleagues pointed out they were muted.
4. **The Non-Technical User**: Configuration screens and driver dialogues are intimidating. They need one clear, honest indicator: *it's on / it's off*.

### 1.2 The Product Promise
- The PC companion never requires active management; it starts with the OS and waits silently.
- On the phone, the only required user action is tapping the microphone or camera.
- The best physical connection is negotiated automatically and upgraded without dropping streams (e.g. plugging in a USB cable mid-call).
- When a link breaks, Mikey self-heals automatically, and when it cannot, it explains why in plain, calm language.
- **Zero bloat guarantee**: No forced accounts, no cloud relays, no audio/video telemetry, and no paywalled core features.

---

## 2. The Seven Product Principles

1. **Convenience First, Configuration Last**: Default settings must be optimal for 90% of users. Primary screens show only what is needed to know *what is on and what is off*.
2. **Advanced Costs Clicks**: Non-essential controls reside one level deeper (the collapsible Advanced drawer on the phone or the flyout expansion on the PC). Power users can find them; typical users are never overwhelmed.
3. **Lightweight by Construction**: Small binaries (APK < 10 MB, PC EXE < 8 MB), minimal dependencies, near-zero idle CPU, and zero background processing when not streaming. Every dependency must justify its inclusion.
4. **Honest State**: The user interface never reports a state that is untrue. If audio transmission stalls, the visual indicators update within seconds.
5. **Self-Healing**: Transient link drops, cable unplugs, and Wi-Fi handoffs recover automatically without requiring user restarts.
6. **Private by Default**: Microphone and camera always initialize in the **OFF** state. An unfamiliar device cannot silently stream audio to a PC over a shared network without verification.
7. **Remember Choices**: User settings persist across app relaunches, device reboots, and updates.

---

## 3. Scope Boundaries

### 3.1 In Scope (v1.0)
- Android 8.0+ (API 26+) smartphones (`arm64-v8a` and `armeabi-v7a`).
- Windows 10/11 x64 and mainstream Linux distributions (Ubuntu, Fedora, Arch; PipeWire or PulseAudio; X11 or Wayland).
- Microphone streaming across all four connection levels (L1 USB debugging, L2 USB tethering, L3 Wi-Fi, L4 Bluetooth).
- Camera streaming on Levels 1, 2, and 3.
- Front/back lens switching, aspect ratio selection (16:9, 4:3, 1:1 square), and adaptive quality scaling.
- Audio processing (RNNoise noise suppression, jitter buffer and drift correction) runs **exclusively on the PC**; the phone sends raw audio.
- Single active streaming device per PC at a time (multiple devices known, one active).

### 3.2 Explicitly Out of Scope (v1.0)
- iOS application or macOS companion daemon.
- Reverse audio streaming (using the phone as a PC speaker).
- Remote screen mirroring, mouse/keyboard control, or file transfer.
- Wide-area internet streaming (Mikey is strictly a local link protocol).
- Multi-camera or multi-microphone simultaneous streaming to separate virtual devices.
- AI background blur or virtual green screens (meeting applications like Zoom, Teams, and Google Meet already execute this natively with GPU acceleration).

---

## 4. User Journeys & Experience

### 4.1 First-Time PC Setup
1. User downloads the single-file installer from GitHub Releases and executes it.
2. The installer copies the `mikey` binary, registers the DirectShow virtual camera (`softcam.dll`), sets an autostart registry entry, adds a private network firewall rule, and starts the tray icon.
3. If the virtual microphone driver (VB-Audio Cable) is not detected, the tray icon displays an amber dot and the flyout presents an actionable banner: *"One more step: install the virtual microphone"*. Clicking the link opens the official download page. Mikey detects installation automatically without requiring an application restart.

### 4.2 First-Time Phone Setup
1. User installs the APK from GitHub or F-Droid and opens Mikey.
2. The interface immediately displays two dimmed split tiles: camera on top, mic on the bottom. There is no multi-page onboarding carousel.
3. The app begins searching for the PC in the background; the status dot turns green when found.
4. Tapping the mic for the first time prompts for Android microphone permission (and notification permission on Android 13+). Tapping the camera prompts for camera permission. No permissions are requested ahead of time.

### 4.3 First Connection Friction by Level

| Transport | One-Time Interaction |
| :--- | :--- |
| **Level 1 (USB Debugging)** | Phone displays *"Allow USB debugging?"* → User checks *Always allow from this computer* and taps Allow. The PC tray turns amber with the prompt *"Tap Allow on your phone"*. |
| **Level 2 (USB Tethering)** | User toggles USB Tethering in Android settings. If a USB cable is connected for > 3s without a link, Mikey provides a direct shortcut link to the tethering settings page. |
| **Level 3 (Wi-Fi)** | First connection prompts the PC with *"Allow [Device Name] to connect?"*. After one approval, the pairing token is saved and future Wi-Fi connections connect silently. |
| **Level 4 (Bluetooth)** | Phone and PC are paired once in OS Bluetooth settings. Mikey requests the Android `BLUETOOTH_CONNECT` permission on first Bluetooth use. |

### 4.4 Mid-Call Make-Before-Break Upgrade
If a user begins a meeting over Wi-Fi and connects a USB cable mid-call:
1. `TransportManager` detects the USB cable event (`ACTION_USB_STATE`).
2. Mikey opens a new connection to `127.0.0.1:7653` and exchanges a silent handshake using the existing pairing token.
3. Media transmission swaps atomically from Wi-Fi to USB.
4. The Wi-Fi socket is closed with `BYE (reason: "switch")`.
5. Meeting participants experience at most a sub-300 ms glitch; audio does not drop.

---

## 5. Risk Register & Mitigations

| Risk ID | Description | Impact | Mitigation |
| :---: | :--- | :---: | :--- |
| **R1** | Audio delay and glitches over Wi-Fi packet jitter | Medium | Adaptive `JitterBuffer` with smooth drift correction; echo cancellation is left to the meeting app (Meet, Zoom, Teams). |
| **R2** | Aggressive OEM battery managers terminate background capture | High | Bind capture to a dedicated Foreground Service with dynamic media types; provide clear instructions for aggressive OEMs (dontkillmyapp.com). |
| **R3** | Android OS permission and Foreground Service rule churn | Medium | Isolate all FGS logic inside `MikeyService` and `Notifier` behind clean abstraction boundaries. |
| **R4** | DirectShow softcam not visible in some UWP / Windows Store apps | Medium | Softcam covers major meeting apps (Zoom, Teams, Meet, OBS); support Media Foundation virtual camera on Windows 11 as a phase upgrade. |
| **R5** | ADB binary conflicts with developer Android SDKs | Low | Prefer `adb` on system `PATH` if present; fall back to local bundled platform-tools binary only when missing. |
| **R6** | Bluetooth RFCOMM throughput instability | Low | Designate Bluetooth as Level 4 fallback; compress audio via Opus at 48 kbps CBR; strictly disable video streaming over Bluetooth. |
| **R7** | USB tethering routes PC traffic through phone mobile data | Medium | Present a one-time informative tip explaining upstream tethering behavior; prefer Level 1 (ADB) when available. |
| **R8** | Linux desktop environments hide standard tray icons | Medium | Support StatusNotifier/AppIndicator protocols; provide fallback command-line options (`mikey --settings`). |
| **R9** | Linux Secure Boot blocks unsigned `v4l2loopback` kernel module | Medium | Recommend distro DKMS packages (auto-signed with MOK on Ubuntu/Fedora); provide comprehensive troubleshooting documentation. |
| **R10**| VB-Audio Cable cannot be redistributed directly | Low | Implement one-click guided download and automated status detection in the companion flyout. |
| **R11**| Client-isolated enterprise Wi-Fi blocks broadcast discovery | Low | Provide `manualPcAddress` in Advanced settings and retain `lastPcAddress` cache for direct IP connection. |
| **R12**| Scope creep degrading core performance | High | Adhere strictly to Product Principle 2: non-essential features belong in Advanced or are rejected. |

---

## 6. Recorded Architectural Decisions

The following 11 architectural decisions govern the Mikey codebase:

1. **Remember Mic/Camera State on Launch**: **OFF by default** for user privacy. Opt-in via Advanced settings.
2. **Wi-Fi First-Time Approval**: **Mandatory prompt** on first connection from an unknown device to protect shared networks (dormitories, offices, coffee shops).
3. **Camera Streaming over Bluetooth**: **Disabled** due to RFCOMM bandwidth limits (~0.5–1.5 Mbps).
4. **Automatic App Launch over ADB**: **Supported** via `adb shell am start` when plugged into an authorized computer.
5. **Video Codec**: **MJPEG** for ultra-low latency, zero frame-dependency artifacts, and cross-platform compatibility without licensing entanglements.
6. **macOS Support**: **Excluded from v1.0** to focus on Windows and Linux stability.
7. **Local Camera Preview on Phone**: **Disabled** to eliminate GPU power draw and prevent thermal throttling; video is previewed exclusively on the PC.
8. **Audio DSP Location**: **PC only**. The phone captures and sends raw audio; RNNoise denoising, jitter buffering and drift correction run on the PC.
9. **Background Blur/Replacement**: **Excluded**; natively handled with hardware acceleration inside Zoom, Meet, and Teams.
10. **PC User Interface**: **Custom native Win32 GDI/GDI+ flyout companion**, replacing rigid OS popup menus with an anchored, double-buffered dark card featuring a live VU meter and controls.
11. **Transport Hierarchy Order**: **Level 1 (ADB) > Level 2 (Tether) > Level 3 (Wi-Fi) > Level 4 (Bluetooth)**. Wi-Fi precedes Bluetooth because it supports high-bandwidth video and lossless audio.
