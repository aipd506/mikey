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
- **Launch:** The tray icon shows at once; audio devices, adb and the virtual camera come up in the background. Started by hand, Mikey opens the flyout, since the icon may sit in the hidden overflow. The login start passes `--autostart` and stays in the tray. Launching Mikey while it runs opens the running one's flyout instead of a second copy.

---

## 7.2 Tray Flyout Companion App

### Why a Flyout instead of a Native OS Context Menu?
Standard Windows context menus (`muda`/Win32 popup menus) are rigid, visually dated, and limited strictly to plain text and checkmarks. They cannot host rich interactive controls: live audio VU meters, volume/gain sliders, embedded camera preview thumbnails, real-time connection telemetry, or inline action cards (e.g. join approvals and driver install prompts).

The Mikey Flyout is an ultra-compact, minimal companion dialog (~300 px wide, dynamic height ~195–310 px) crafted with Mikey’s sleek dark design language (`#000000` / `#111111`, `#2C2C2E` dividers, `#FFFFFF` text, `#8E8E93` subtext). It prioritizes visual density, icons over verbose text, an integrated VU meter, and persistent camera preview controls. Common settings (noise suppression, echo cancellation, noise gate, mute, lens flip, preview) are bidirectional and kept in sync between Android and PC via `0x04 CONTROL` frames.

### Visual Layout & Components

```text
┌────────────────────────────────────────────────────────┐
│  Mikey                  Pixel 7 Pro  [L1 USB]  ● Live  │ ← Header: device & status pill
├────────────────────────────────────────────────────────┤
│  🎙 Microphone                              [ Mute ]   │ ← Mic row + soft-mute pill
│  [██████████████████░░░░░░░░░░░░░] -12 dB              │ ← Integrated live 4px VU meter
├────────────────────────────────────────────────────────┤
│  📷 Camera                      [ Flip ⟲ ]  [ Preview ]│ ← Camera row: flip + preview
│  ┌──────────────────────────────────────────────────┐  │
│  │               [Live Video Preview]               │  │ ← 16:9 embedded preview (when live)
│  └──────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────┤
│  DSP:  [───●─────] NS 50%                 [ Gate ]     │ ← Minimal NS slider & Gate (AEC always on)
├────────────────────────────────────────────────────────┤
│  v1.0.0                         [ 📁 ]   [ ⚡ ]   [ ⏻ ]  │ ← Footer: logs, disconnect, quit
└────────────────────────────────────────────────────────┘
```

### Detailed Component Specifications

#### A. Header & Connection Status
- **Device Identity:** Name of the currently connected phone (e.g., `Pixel 7 Pro` or `Waiting for phone...`).
- **Connection Badge:** Pill badge displaying active transport level (`L1 USB`, `L2 Tether`, `L3 Wi-Fi`, `L4 BT`, or `IDLE`) with its matching transport icon (`\u{E88E}` USB, `\u{E702}` BT, `\u{E701}` Wi-Fi).
- **Status Dot:** `#30D158` (green = live stream), `#FFD60A` (amber = waiting / handshake), `#3A3A3C` (dimmed grey = idle).

#### B. Microphone & Audio Controls
- **Streaming State:** Soft microphone glyph dynamically highlights in `#30D158` when active, `#FF453A` when muted, and `#3A3A3C` when idle.
- **Mute Action:** Compact pill button (`[ Mute ]` / `[ Unmute ]`) controlling soft-mute, synchronized bidirectionally with the phone notification and UI.
- **Integrated Live VU Meter:** Direct 4 px horizontal bar immediately under the mic label with instant attack and smooth decay (`#30D158` normal, `#FFD60A` peak, `#FF453A` clipping).

#### C. Camera & Video Controls
- **Persistent Camera Option:** Even when camera video is off/idle, a visual **`[ Preview ]`** button is always accessible to open/close the detached floating preview window (§ 7.5).
- **Lens Flip Control:** Visual **`[ Flip ⟲ ]`** button sends a `0x04 CONTROL` frame to flip the phone's front ↔ back lens seamlessly.
- **Embedded 16:9 Thumbnail:** When video streaming is active and the detached window is not popped out, the flyout expands dynamically from ~215 px to ~330 px to blit the live 16:9 video frame.
- **Camera Indicator:** Soft camera glyph illuminates in vivid `#0A84FF` when streaming is active.

#### D. Audio DSP Quick-Toggles & Bidirectional Sync
- **Always-On Echo Cancellation (`AEC`):** Acoustic Echo Cancellation with WASAPI loopback reference matching and non-linear residual suppression is **always enabled by default** in the audio DSP engine. Because it permanently prevents speaker sound from looping back to remote meeting participants, no manual toggle button is shown in the UI, eliminating clutter.
- **Noise Suppression (`RNNoise`):** Compact slider adjusting software suppression strength (0–100%).
- **Noise Gate:** Compact pill toggle (`[ Gate ]`) suppressing background room hiss. On sets a -45 dB threshold, off sends `gate_db: null`. Off by default, like on the phone.
- **Disconnect** (footer and per device): ends the session. The PC sends BYE `disconnect` and the phone stops its mic and camera ([wire-protocol.md](../architecture/wire-protocol.md)).
- **Bidirectional Settings Sync:** All audio DSP and video settings are shared common settings synchronized via `0x04 CONTROL` frames — changing them on the PC updates the phone, and changing them in the phone drawer updates the PC flyout.


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
