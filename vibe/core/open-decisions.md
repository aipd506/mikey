# 23. Open Decisions

Decisions made in this brain that the owner may want to revisit. Current choice is in **bold**.

1. **Remember mic/camera on-state across launches?** → **Off by default** (privacy), opt-in under Advanced. The rest of the settings always persist.
2. **Wi‑Fi first-time approval even with ask-before-join off?** → **Yes**, once per new device (security on shared networks). Opt-out under Advanced.
3. **Camera over Bluetooth?** → **No** (bandwidth). Revisit only if a low-bitrate H.264 experiment shows usable 360p.
4. **Open the phone app automatically over ADB when plugged in?** → **On**, under Advanced (harmless; it never turns on mic/camera by itself).
5. **Video codec** → **MJPEG** (simple, low latency, no patent questions). Revisit H.264 via `MediaCodec` + `openh264` if Wi‑Fi bandwidth or phone heat is a problem.
6. **macOS** → **Not in v1.**
7. **Phone preview when camera is on** → **None.** The phone never shows the camera video. You see yourself on the PC (the meeting app's self-view and the PC preview window), and the phone shows a clear live state instead. Decided 2026-09-27: the phone usually faces away (back camera), and a preview costs battery and heat.
8. **Where audio processing runs** → **PC only.** The phone sends raw audio (raw PCM on USB, Opus on Wi‑Fi/Bluetooth).
9. **Background blur/removal** → **Not built.** It would add an AI model + inference runtime (several MB) and per-frame CPU (or phone battery/heat), and Zoom, Meet, Teams and Discord already do it well. Revisit only as an optional, off-by-default add-on if users ask.
10. **PC UI: Native context menu vs. custom tray flyout dialog** → **Custom tray flyout dialog.** A standard OS context menu (`muda`/Win32 menu) is static, visually dated, and cannot host rich interactive controls (live audio VU meters, volume/gain sliders, embedded camera preview thumbnail, DSP toggle buttons, or inline device switcher cards). The flyout is an anchored, borderless companion window built in lightweight native Rust (`egui`/`eframe` or borderless GUI) that opens on tray click and light-dismisses on blur, keeping idle memory < 15 MB.

