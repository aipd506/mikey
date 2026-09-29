# Changelog

All notable changes to Mikey will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Nothing yet.

---

## [0.1.0] - 2026-09-30

The first preview release.

### Phone app (Android)
- Foreground service that streams the mic and camera to the PC; the mic and camera always start off
- Four connection levels chosen automatically: USB debugging, USB tethering, Wi-Fi and Bluetooth (audio only), switching mid-call make-before-break
- Finds the PC on Wi-Fi with a UDP beacon, and trusts it on first use after the PC approves the phone
- Raw 48 kHz mic through AAudio with an AudioRecord fallback: PCM over USB, Opus over Wi-Fi (96 kbps) and Bluetooth (48 kbps)
- Camera through CameraX as JPEG frames, at 720p or 1080p, front or back lens
- Split-screen UI for the mic and camera, a full-height settings sheet kept in sync with the PC, and a notification with mic and camera buttons
- Voice always goes out before camera pictures, so a slow link delays video, not voice

### PC app (Windows)
- Tray app with a native panel: mic level, mute, camera preview and noise suppression
- TCP listener (`:7653`), UDP discovery beacon (`:7654`), Bluetooth RFCOMM and an adb watcher
- Sessions with trust on first use, ask-before-join and a 30 s hold when the link drops
- Audio: Opus or PCM decode, RNNoise noise suppression, an adaptive jitter buffer, smooth drift correction with cubic interpolation, fades at gaps and a soft clip, into VB-CABLE as *Mikey Mic*
- Video: JPEG decode, smooth scaling to one fixed 1920×1080 size, into the built-in virtual camera *Mikey Cam* (softcam)

### Project
- CI for the PC app (Windows and Linux) and the Android app, grouped weekly Dependabot updates, a contributing guide, code of conduct, security policy, and issue and pull request templates

[Unreleased]: https://github.com/diveshpatil9104/mikey/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/diveshpatil9104/mikey/releases/tag/v0.1.0
