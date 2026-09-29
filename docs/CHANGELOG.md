# Changelog

All notable changes to Mikey will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Full open source infrastructure: CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md, issue/PR templates
- Releases folder with download guidance
- Comprehensive documentation overhaul

---

## [0.1.0] - In Development

### Android
- MikeyService foreground service with mic and camera capture
- TransportManager with 4 connection levels (USB Debug, USB Tether, Wi-Fi, Bluetooth)
- Make-before-break transport upgrades with 10s hysteresis
- Opus encoding for Wi-Fi (96 kbps) and Bluetooth (48 kbps)
- AAudio low-latency capture with AudioRecord fallback
- CameraX → JPEG → VIDEO frame pipeline
- Full UI: split halves (camera/mic), settings drawer, status indicators
- Pairing token storage, PENDING/REJECT handling
- Notification with live state, mute/unmute, Stop

### PC (Rust)
- TCP listener (:7653), UDP discovery beacon (:7654), Bluetooth RFCOMM
- SessionManager with trust-on-first-use, ask-before-join, 30s session hold
- Audio pipeline: Opus/PCM decode → adaptive jitter buffer → drift resampler → SpeexDSP AEC → noise gate → RNNoise → virtual mic
- Video pipeline: JPEG decode → letterbox → softcam (Windows) / v4l2loopback (Linux)
- Win32 GDI flyout companion app with VU meter, volume, DSP toggles
- Tray icon with autostart at login
- Inno Setup installer (Windows), .deb and AppImage (Linux)

[Unreleased]: https://github.com/diveshpatil9104/mikey/compare/HEAD
[0.1.0]: https://github.com/diveshpatil9104/mikey/releases/tag/v0.1.0
