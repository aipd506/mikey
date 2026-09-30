# Development Roadmap & Test Matrix

This document tracks Mikey’s five implementation phases, current development status, release criteria, and the real-device test matrix required to validate production builds.

---

## 1. Development Phases & Current Status

- **Current Status**: **Phases 1 through 4 are complete on both Android and PC**. Both codebases build cleanly and implement full media streaming, multi-transport negotiation, audio DSP, video decoding, and native OLED-black UI.
- **Next Step**: **Phase 5 (Hardening, Packaging & Release)**: Signed APK builds, Inno Setup installer automation, real-device test matrix sign-offs, and final release documentation.

---

## 2. Phase Breakdown & Completion Checklists

### 2.1 Phase 1 - Microphone over the Simplest Wire (Complete)
*Goal: Prove the audio path end-to-end over Level 1 USB debugging.*
- [x] **Android**: Single activity, basic tap halves, status dot.
- [x] **Android**: `MikeyService` foreground service with microphone type.
- [x] **Android**: `AudioRecord` 48 kHz mono capture → 10 ms PCM frames → TCP to `127.0.0.1:7653`.
- [x] **Android**: Protocol handshake: `HELLO`, `WELCOME`, `HEARTBEAT`, `BYE`.
- [x] **Android**: Swipe-away from Recents stops capture and terminates the service cleanly.
- [x] **PC**: Console binary with TCP listener on port 7653.
- [x] **PC**: Media demuxing and basic audio routing to virtual microphone (VB-Audio Cable).
- [x] **PC**: Connection logging (`[connected] Pixel via L1`).

### 2.2 Phase 2 - Multi-Transport Engine & Trust (Complete)
*Goal: Automatic transport negotiation, discovery, make-before-break upgrades, and pairing trust.*
- [x] **Android**: Production UI with OLED split-screen, settings sheet, and three-color status dot.
- [x] **Android**: `TransportManager` managing Levels 1–4, event-driven probing, and make-before-break handoff.
- [x] **Android**: UDP discovery beacon responder and fast-path IP caching.
- [x] **Android**: Socket pinning via `Network.bindSocket` to prevent cellular route leakage.
- [x] **Android**: Native Opus encoding on Wi-Fi (96 kbps) and Bluetooth (48 kbps) with `FrameJoiner`.
- [x] **Android**: Native AAudio low-latency capture via NDK with AudioRecord fallback.
- [x] **Android**: Bluetooth RFCOMM client targeting bonded PC endpoints.
- [x] **Android**: Notification controls: live state text, soft mute/unmute, and stop action.
- [x] **Android**: Persistent pairing tokens, TOFU pending/reject handling, and "Forget" button.
- [x] **PC**: Background tray icon with silent startup and autostart registry integration.
- [x] **PC**: Native Win32 GDI/GDI+ companion flyout with live VU meter and controls.
- [x] **PC**: `SessionManager`: TOFU pairing, ask-before-join modals, and 30-second session hold.
- [x] **PC**: Audio pipeline: Opus decoding, adaptive jitter buffer (20–120 ms), and drift resampler.
- [x] **PC**: Missing virtual driver detection with guided installation links.

### 2.3 Phase 3 - Video Pipeline & Virtual Camera (Complete)
*Goal: Stream camera video into all major videoconferencing applications.*
- [x] **Android**: CameraX `ImageAnalysis` capture with `STRATEGY_KEEP_ONLY_LATEST`.
- [x] **Android**: Live camera state tile with front/back flip toggle (no local screen preview).
- [x] **Android**: Aspect ratio options (16:9, 4:3, 1:1), quality presets, and thermal throttling back-off.
- [x] **Android**: Dynamic `FOREGROUND_SERVICE_TYPE_CAMERA` added only while video capture is active.
- [x] **Android**: Camera disabled with informative message over Bluetooth or when PC lacks virtual camera.
- [x] **Android**: Gravity orientation hysteresis (20° threshold) ensuring upright video output.
- [x] **PC**: TurboJPEG SIMD background decoding thread.
- [x] **PC**: DirectShow virtual camera filter registration (`softcam.dll`) exposed as "Mikey Cam".
- [x] **PC**: Aspect ratio preservation with automatic letterboxing/pillarboxing.
- [x] **PC**: Privacy placeholder frame (1920×1080) displayed when camera is toggled off.
- [x] **PC**: Detached native Win32 floating preview window.

### 2.4 Phase 4 - Audio Quality & DSP (Complete)
*Goal: Clean, studio-quality speech with background noise suppression and stable low latency.*
- [x] **Android**: Bidirectional control frame sync (`0x04 CONTROL`) for noise suppression and mute.
- [x] **Android**: Settings disabled/greyed out when PC companion reports missing capabilities.
- [x] **PC**: RNNoise neural network speech noise reduction (`nnnoiseless`) with strength slider.
- [x] **PC**: Adaptive jitter buffer (20 to 120 ms) and smooth drift correction with cubic interpolation.
- [x] **PC**: Streamlined DSP pipeline (SpeexDSP AEC and noise gate retired in #37 in favor of pure RNNoise neural suppression and adaptive jitter buffering).
- [x] **PC**: Full bidirectional synchronization of audio and video settings across devices.

### 2.5 Phase 5 - Packaging, Hardening & Release (In Progress)
*Goal: A stranger installs and uses Mikey in under 5 minutes.*
- [ ] Signed Android APK build automation with ABI splits (`arm64-v8a`, `armeabi-v7a`).
- [ ] Windows Inno Setup installer bundling `mikey.exe`, DirectShow `softcam.dll`, and firewall rules.
- [ ] Linux `.deb` and AppImage packages with desktop autostart entries.
- [ ] Optional: AES-GCM encryption on Level 3 (Wi-Fi) using pairing-established keys.
- [ ] Real-device test matrix validation across phones, host OSes, and video conferencing apps.
- [ ] Tag `v1.0.0` release on GitHub.

---

## 3. Real-Device Test Matrix

Because hardware timers, audio HALs, camera sensors, and USB interfaces cannot be simulated reliably in virtual machines, the following test matrix must pass on physical hardware before closing releases:

| Test Area | Concrete Scenarios & Validation Criteria |
| :--- | :--- |
| **Transport Levels** | Validate L1 (USB debugging), L2 (USB tethering), L3 (Wi-Fi), and L4 (Bluetooth) independently. Test mid-call upgrades (Wi-Fi → USB plug) with < 300 ms glitch and downgrades (USB unplug → Wi-Fi) with < 2 s recovery. |
| **Lifecycle & Background** | Minimize app, lock phone screen for 30 minutes, swipe app away from Android Recents, trigger Stop from notification, reboot phone, trigger PC sleep/wake, and log out/in. |
| **Trust & TOFU** | Verify first-time pairing on all 4 levels with Ask-Before-Join OFF and ON. Verify rejection of secondary phone when PC is busy. Verify "Forget Device" resets pairing state on both sides. |
| **Android OS Versions** | Validate on Android 8.0 (AudioRecord fallback), Android 10, Android 12, Android 13 (Notification permissions), Android 14, and Android 15/16 (Foreground service type constraints). |
| **OEM Implementations** | Validate on Google Pixel (AOSP stock), Samsung One UI, Xiaomi MIUI/HyperOS (aggressive background process killer testing), and OnePlus/Oppo ColorOS. |
| **Network Environments** | Test on standard home Wi-Fi (5 GHz / 2.4 GHz), phone mobile hotspot with PC joined, client-isolated corporate Wi-Fi (manual IP configuration), and congested RF environments. |
| **Application Integration**| Verify virtual mic and virtual camera recognition in Zoom, Microsoft Teams, Google Meet (Chrome, Edge, Firefox), Discord, OBS Studio, and Audacity. |
| **Host Operating Systems** | Validate on Windows 10 x64, Windows 11 x64, Ubuntu Desktop (GNOME), Fedora (PipeWire), and Arch Linux (KDE Wayland). |
| **Long-Run Endurance** | Execute a continuous **3-hour streaming call** on each transport level: verify zero buffer drift growth, zero memory leaks, zero CPU spikes, and zero audio synchronization slips. |
