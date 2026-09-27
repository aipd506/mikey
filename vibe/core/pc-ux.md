# 7. PC App — UX Specification

## 7.1 Presence & Interaction Model

- **System Tray Icon:** Lives in the notification area (Windows notification area / Linux StatusNotifier). No persistent main window, no clutter in the taskbar.
- **Icon States:**
  - **Grey** (`#8E8E93`): Idle / disconnected / searching for phones.
  - **Green** (`#30D158`): Streaming active (mic and/or camera live).
  - **Amber** (`#FFD60A`): Waiting for approval, ADB authorization required, driver missing, or incoming connection request.
- **Click Behavior:**
  - **Left-Click:** Opens or toggles the **Mikey Flyout Companion App** — a compact, borderless dialog window anchored directly adjacent to the tray icon.
  - **Right-Click:** Opens the flyout directly, or provides a fast 3-item fallback menu (`Open Mikey`, `Mute Mic`, `Quit`).
  - **Light Dismiss:** The flyout automatically closes when it loses focus (clicking anywhere outside the dialog), pressing `Esc`, or clicking the tray icon again.
- **Resource Footprint:** Starts at login. Consumes < 15 MB RAM when the flyout is closed. Zero background rendering or animation overhead.

---

## 7.2 Tray Flyout Companion App

### Why a Flyout instead of a Native OS Context Menu?
Standard Windows context menus (`muda`/Win32 popup menus) are rigid, visually dated, and limited strictly to plain text and checkmarks. They cannot host rich interactive controls: live audio VU meters, volume/gain sliders, embedded camera preview thumbnails, real-time connection telemetry, or inline action cards (e.g. join approvals and driver install prompts).

The Mikey Flyout is an anchored companion dialog (~350 px wide, dynamic height ~440–540 px) crafted with Mikey’s sleek dark design language (`#000000` / `#111111`, `#2C2C2E` dividers, `#FFFFFF` text, `#8E8E93` subtext).

### Visual Layout & Components

```text
┌────────────────────────────────────────────────────────┐
│  Mikey                          [L1 USB 480M]  ● Live  │ ← Status & level badge
│  Pixel 7 Pro                    12 ms · 96 kbps · 0%   │ ← Telemetry
├────────────────────────────────────────────────────────┤
│  MICROPHONE                                            │
│  ● Mic Streaming Active                     [Mute]     │ ← Streaming state
│  Level  [██████████████░░░░░░░░] -14 dB                │ ← Live VU meter
│  Gain   [───────●──────────────] 100%                  │ ← Volume/gain slider
│                                                        │
│  DSP: [✓ Noise Suppress] [✓ Echo Cancel] [✓ Noise Gate]│ ← DSP toggles
│  Sink: VB-Cable (Ready ✓)       Echo Ref: Speakers ▼   │ ← Audio routing
├────────────────────────────────────────────────────────┤
│  CAMERA                                                │
│  ● Camera Live — 1080p @ 30fps              [Pop Out]  │ ← Pop-out button
│  ┌──────────────────────────────────────────────────┐  │
│  │                                                  │  │
│  │               [Live Video Preview]               │  │ ← 16:9 embedded preview
│  │                                                  │  │
│  └──────────────────────────────────────────────────┘  │
│  Driver: softcam (Ready ✓)             Flip: Front [↻] │ ← Driver & camera flip
├────────────────────────────────────────────────────────┤
│  DEVICES                                               │
│  ● Pixel 7 Pro (Active)                   [Disconnect] │ ← Active device
│  ○ Galaxy Tab S8 (Paired)                 [Connect]    │ ← Device switching
├────────────────────────────────────────────────────────┤
│  ▸ Advanced Settings                                   │ ← Collapsible drawer
│    Ask before joining                              ☐   │
│    Start with computer                             ☑   │
│    Open app on phone when USB plugged in           ☑   │
│    Connection levels: L1 ☑  L2 ☑  L3 ☑  L4 ☑          │
│    Trust new Wi-Fi devices automatically           ☐   │
├────────────────────────────────────────────────────────┤
│  Mikey v1.0.0             [Open Logs]    [Quit Mikey]  │ ← Footer
└────────────────────────────────────────────────────────┘
```

### Detailed Component Specifications

#### A. Header & Connection Status
- **Device Identity:** Name of the currently connected phone (e.g., `Pixel 7 Pro`).
- **Connection Badge:** Pill badge displaying active transport level (`L1 USB`, `L2 Tether`, `L3 BT`, `L4 Wi-Fi`) and link rate.
- **Status Dot:** `#30D158` (green = live stream), `#FFD60A` (amber = waiting / handshake), `#8E8E93` (grey = idle).
- **Live Stream Telemetry:** Displays real-time round-trip latency (`ms`), audio/video bitrate (`kbps`), and packet loss percentage (`0.0%`).
- **Quick Action:** [Disconnect] button to immediately release the session.

#### B. Microphone & Audio Controls
- **Streaming State:** Shows whether the phone microphone is actively sending audio.
- **Live Audio VU Meter:** Horizontal meter bar with peak hold, updating at 30 fps directly from the decoded audio pipeline (`#30D158` normal, `#FFD60A` peak, `#FF453A` clipping).
- **Output Gain Slider:** Smooth slider controlling software gain (0% to 150%) sent to the virtual microphone, alongside a quick soft-mute button.
- **Audio DSP Quick-Toggles:**
  - `Noise Suppression`: Toggles pure-Rust `RNNoise` filter.
  - `Echo Cancellation`: Toggles `SpeexDSP` acoustic echo cancellation.
  - `Noise Gate`: Toggles threshold gating.
- **Audio Routing & Drivers:**
  - Virtual Microphone status: `VB-Cable: Ready ✓` (or an amber `[Install VB-Cable...]` button if missing).
  - Echo Reference Device: Dropdown selecting the PC speaker output monitored for echo cancellation.

#### C. Camera & Video Controls
- **Streaming State:** Shows whether video capture is active from the phone.
- **Resolution & FPS:** Badge displaying current capture format (e.g. `1080p @ 30fps` or `720p @ 30fps`).
- **Embedded 16:9 Preview:** Live video frame decoded and blitted directly inside the flyout card. Video decode only runs while the flyout or detached preview window is visible.
- **Pop-Out Action:** [Pop Out] button detaches the preview into the floating, movable window (§ 7.5).
- **Camera Flip Request:** Sends a control frame (`0x04 CONTROL`) to switch front/back lens on the phone.
- **Virtual Camera Driver:** `softcam: Ready ✓` (or an amber `[Install softcam...]` button if driver not registered).

#### D. Device Switcher & Approvals
- **Active & Paired Devices:** Lists currently active device and previously paired devices stored in `config.toml`.
- **Inline Actions:** [Disconnect], [Connect], and [Forget] buttons per device.
- **Pending Join Request Banner:** If a phone requests connection while the PC is busy or waiting for approval, an actionable banner appears at the top:
  `⚠️ Join Request: Galaxy Tab wants to connect`
  `[Allow]  [Always Allow]  [Deny]  (Auto-expires in 45s)`

#### E. Collapsible Settings Drawer
- `Ask before joining`: Toggle (default OFF).
- `Start with computer`: Toggles autostart entry (`Run` registry key on Windows, autostart desktop entry on Linux).
- `Open app on phone when USB plugged in`: Drives ADB watcher to launch Mikey on phone connection.
- `Connection levels`: Checkboxes to selectively enable/disable specific transport levels (USB debugging, USB tethering, Bluetooth, Wi-Fi).
- `Trust new Wi-Fi devices automatically`: Toggle (default OFF).

#### F. Footer
- Version string (`Mikey v1.0.0`).
- [Open Logs] button: Opens `%APPDATA%\Mikey\logs` (Windows) or `~/.config/mikey/logs` (Linux) in the system file explorer.
- [Quit Mikey] button: Cleanly terminates background accept threads, shuts down the session, and exits the process.

---

## 7.3 Anchoring & Multi-Monitor Positioning

- **Position Calculation:**
  - The flyout dynamically queries the tray icon rectangle via OS shell APIs (`Shell_NotifyIconGetRect` on Windows, or cursor position fallback).
  - **Bottom Taskbar (default):** Flyout opens centered or right-aligned directly above the tray icon, with an 8 px margin from the taskbar edge.
  - **Top Taskbar:** Flyout opens directly below the tray icon with an 8 px margin.
  - **Left / Right Taskbar:** Flyout opens adjacent to the taskbar edge.
- **Screen Boundary Clamping:** Clamped strictly within the current monitor's `work_area`. Multi-monitor setups anchor the window to whichever display holds the clicked tray icon.
- **Window Properties:** Frameless, borderless, no taskbar button (`WS_EX_TOOLWINDOW`), 8 px corner radius, 1 px subtle `#2C2C2E` border outline.

---

## 7.4 Ask Before Joining & Approvals

- **Off (default):** Known devices connect instantly. New devices on USB or Bluetooth connect instantly. New devices on Wi‑Fi prompt once (§ 11.3).
- **On:** Every connection attempt triggers:
  1. An OS notification: *"Pixel 7 wants to use your mic/camera — [Allow] [Deny]"*.
  2. The tray icon turns amber.
  3. Clicking the tray icon opens the flyout with the prominent **Join Request Banner** at the top.
  4. If unanswered within 60 s, the request expires, sending `0x12 REJECT` to the phone.
- **Busy:** If a phone is already streaming and another device connects, the flyout surfaces *"Switch connection to Galaxy Tab?"*. Declining keeps the existing stream active.

---

## 7.5 Pop-Out Preview Window

- In addition to the embedded preview thumbnail inside the flyout, clicking [Pop Out] launches a detached preview window.
- **Window Specs:** Borderless, movable (drag anywhere by clicking on the preview frame), default `320×180` (16:9), resizable, remembers its last screen position in `config.toml`.
- **Lifecycle:** Closing the pop-out window returns video preview rendering exclusively to the flyout card. Decoding and blitting suspend completely when neither window is visible.

---

## 7.6 PC Notifications (Kept to a Minimum)

Notifications appear only for actionable system events:
1. Join requests (when ask-before-join is ON or new Wi-Fi device).
2. "Tap Allow on your phone" (ADB authorization pending).
3. Missing virtual device driver notice with download button.
4. Unexpected stream drop while actively in a call.
No "Connected!" or "Ready" toasts.

---

## 7.7 PC Settings File

- Path: `%APPDATA%\Mikey\config.toml` (Windows) / `~/.config/mikey/config.toml` (Linux).
- Human-readable TOML, written atomically.
- Stores:
  - Toggles: `ask_before_join`, `start_with_computer`, `adb_auto_open`, `trust_wifi_auto`, enabled transports.
  - Device list: Trusted devices table (id, friendly name, pairing token, last transport).
  - Audio preferences: Output gain, noise gate threshold, selected echo reference audio device.
  - Window state: Last pop-out preview window position and dimensions.
- Logs rotate automatically at 1 MB × 3 files.
