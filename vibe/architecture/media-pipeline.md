# 13. Media Pipeline

## 13.1 Audio

**Phone:**
- **Raw capture, no phone-side processing.** 48 kHz, mono, 16-bit, audio source `UNPROCESSED` where the device supports it (API 24+ reports this), otherwise `MIC`. Never `VOICE_COMMUNICATION` — the phone's own noise suppression and auto-gain would distort the signal the PC's echo canceller depends on.
- Capture via **AAudio** in low-latency mode (NDK, through the existing JNI layer — no extra library), falling back to `AudioRecord` with the minimum buffer if AAudio isn't available (Android 8.0).
- What is sent depends on the level:

  | Level | Format | Frame | Why |
  |---|---|---|---|
  | 1–2 USB | Raw PCM s16le | 10 ms | ~0.77 Mbps is trivial for USB; zero codec delay; perfect quality |
  | 4 Wi‑Fi | Opus 96 kbps (raw PCM if *Lossless audio on Wi‑Fi* is on) | 10 ms | Transparent to the ear, adds ~5–10 ms, more robust on busy Wi‑Fi |
  | 3 Bluetooth | Opus 32–48 kbps | 20 ms | Only option that fits the link |

  Opus runs in `RESTRICTED_LOWDELAY` mode on Wi‑Fi (music-grade quality, lowest delay) and VOIP mode with in-band FEC on Bluetooth. Encoding costs far less phone CPU than any noise suppression would.
- Capture runs on a dedicated high-priority thread; it never blocks on the network. A small bounded queue sits between capture and send; if the network stalls, the **oldest** audio is dropped (freshness beats completeness in a live call).

**PC:**
```
frames → [Opus decode] → jitter buffer ──► drift resampler ──► AEC ──► noise gate ──► RNNoise ──► virtual mic
                         (target: USB 20 ms,  (rubato, ±0.2%     (if aec on)  (if gate_db)  (if ns on)
                          Wi-Fi 40 ms, BT      to hold buffer
                          80 ms; adaptive)     at target)
```
- **Order matters:** echo cancellation runs first, on the rawest signal, then noise gate and RNNoise clean what's left.
- **Latency cap:** if buffered audio exceeds 200 ms (e.g. after a Wi‑Fi stall), the excess is dropped rather than played late.
- **Drift:** the phone's and PC's clocks run at slightly different speeds. Without correction the buffer slowly grows or empties over long calls. A tiny adaptive resampling ratio keeps the buffer at its target.
- **Gaps:** silence is written (or Opus PLC on L3/L4) so the virtual mic never stalls.

**Echo cancellation (PC):**
- Reference = what the PC is playing: WASAPI loopback of the chosen output device (Windows) / the output's monitor source (Linux).
- Engine: **SpeexDSP** `speex_echo` (small, BSD). If quality testing shows it's not good enough under Wi‑Fi jitter, evaluate WebRTC's AEC3 (better delay handling, but a larger C++ build) behind the same `aec.rs` interface.
- Works best on USB levels (stable delay). On Wi‑Fi, the jitter buffer's delay is fed to the AEC as its delay hint.
- If the user wears headphones, AEC is unnecessary; it is harmless but costs CPU — a later improvement can auto-bypass when no echo is detected.

## 13.2 Video

**Phone:**
- CameraX `ImageAnalysis` only (YUV_420_888, `STRATEGY_KEEP_ONLY_LATEST`, so it never builds a backlog). No `Preview`: the phone never shows the video, it is shown on the PC.
- Resolution from Aspect ratio + Quality: 16:9 → 1280×720 / 1920×1080; 4:3 → 960×720 / 1440×1080; 1:1 → center-cropped 720×720 / 1080×1080. Auto = 720p on Wi‑Fi, 1080p on USB.
- Rotation applied so the PC always gets an upright image.
- JPEG encode (quality 75 default, auto-lowered if the send queue backs up or the phone reports thermal throttling via `PowerManager` thermal status).
- **Flip:** rebinds CameraX with the other lens. The PC holds the last frame during the ~300–500 ms switch, so apps see no black flash and no device change.

**PC:**
- Decode JPEG (`zune-jpeg`), convert to the virtual camera's pixel format, push.
- The virtual camera advertises the resolution chosen when it's first opened by an app. If the user changes aspect ratio while an app is using the camera, frames are **letterboxed/pillarboxed** into the existing size until the app reopens the camera (apps like Zoom don't tolerate mid-stream format changes).
- When the camera is off: the virtual camera shows a black frame with a small, neutral Mikey mark (not a frozen last frame — privacy).

## 13.3 Virtual devices

| | Windows | Linux |
|---|---|---|
| Mic | **VB-Cable** (user installs once; cannot be bundled under its license). Mikey writes to "CABLE Input"; apps pick "CABLE Output". | PipeWire/PulseAudio: `module-null-sink` (`mikey_sink`) + `module-remap-source` → apps see **"Mikey Microphone"** as a real input, not a "Monitor of…". Created on start, removed on quit. |
| Camera | **softcam** DirectShow filter (MIT), registered once by the installer. Works in Zoom, Teams, Meet (Chrome/Edge), OBS, Discord. Not visible to the built-in Windows Camera app. Later (Phase 5): Media Foundation virtual camera on Windows 11 for universal support. | **v4l2loopback** kernel module, loaded with `exclusive_caps=1 card_label="Mikey Camera"` (required for Chrome/WebRTC to see it). Installer adds `/etc/modules-load.d` + `modprobe.d` entries. Secure Boot systems need the DKMS module signed — documented. |

The tray flyout's audio and camera cards show Ready ✓ / Install… for each virtual device, and the phone's WELCOME `pc_caps` tells the phone whether the camera can be used at all.
