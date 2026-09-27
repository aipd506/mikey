# Phase Roadmap

Each phase ends with a real-device test on at least two phones (one Pixel/near-stock, one Samsung or Xiaomi) and one Windows + one Linux machine, and with the budgets in [performance-budgets.md](../rules/performance-budgets.md) checked.

## Phase 1 — Mic over the simplest wire

**Goal:** prove the audio path end-to-end on the #1 priority level.
**Done when:** you plug in a phone with USB debugging on, open Mikey, tap mic, and a Zoom call hears you — without typing anything.

Android
- [x] Single activity, two tap halves (no polish), status dot
- [x] `MikeyService` foreground service (mic type), notification with Stop
- [x] `AudioRecord` 48 kHz mono → PCM frames (10 ms) → TCP to `127.0.0.1:7653`
- [x] HELLO / WELCOME / HEARTBEAT / BYE (no tokens yet)
- [x] Manual IP field (debug only) for Wi‑Fi testing
- [x] Swipe-away from Recents stops everything

PC
- [x] Console binary (no tray yet): TCP listener, framing, handshake
- [ ] `adb.rs`: track devices, `adb reverse`
- [ ] Jitter buffer (fixed 20 ms) → virtual mic (VB-Cable on Windows, null-sink + remap-source on Linux)
- [x] Logs `[connected] Pixel 7 via L1` / `[disconnected]`

## Phase 2 — The four levels, trust, and the real phone UI

**Goal:** Mikey finds the PC by itself on every level and upgrades/downgrades automatically.
**Done when:** with the same phone, you can go Wi‑Fi → plug in (USB) → unplug → turn Wi‑Fi off (Bluetooth) during a single call and the other person keeps hearing you, with only brief blips.

> Order on Android: every feature first, the full UI last. Until then the app keeps its plain Phase 1 screen, with debug-only controls where a feature needs input.

Android
- [ ] Full UI per [phone-ux.md](phone-ux.md) (split halves, chevron + drawer, status dot 3 colors, rotation behavior)
- [ ] Settings persistence per [phone-ux.md](phone-ux.md)
- [x] `TransportManager` for USB debugging, USB tethering and Wi‑Fi: event-driven probing, make-before-break upgrade, 10 s hysteresis ([connection-levels.md](../architecture/connection-levels.md))
- [x] UDP discovery beacon; last-IP fast path
- [x] Interface-pinned sockets (`Network.bindSocket`)
- [x] Opus on Wi‑Fi (96 kbps / 10 ms, low-delay mode; raw PCM if lossless is on)
- [x] Opus on Bluetooth (48 kbps / 20 ms, VOIP mode); needs the real-device test
- [ ] AAudio low-latency capture
- [x] Bluetooth transport (bonded computers only, cached MAC); needs the real-device test
- [x] Notification: live state text, soft mute/unmute, Stop
- [x] Pairing token storage; PENDING/REJECT handling; "Forget"
- [x] USB-connected-but-no-link tethering hint (in the state; the UI shows it later)

PC
- [x] Tray icon + autostart at login
- [x] Tray flyout companion app per [pc-ux.md](pc-ux.md) (replaces native context menu with anchored dialog: live VU meter, volume slider, DSP toggles, device switcher, embedded preview)
- [x] `beacon.rs`, `bt.rs` (Windows + Linux)
- [x] `session.rs`: tokens, trust rules ([sessions-trust.md](../architecture/sessions-trust.md)), ask-before-join prompts, one-active rule, handover with 30 s session hold
- [x] Opus decode; adaptive jitter buffer; drift resampler; 200 ms latency cap
- [x] Virtual-device detection with "Install…" guidance
- [x] Installer (Windows) and .deb/AppImage (Linux) with firewall rule
- [x] Refactor pass: modules as in [repo-structure.md](../architecture/repo-structure.md) (Phase 1 was allowed to be rough)

## Phase 3 — Camera

**Goal:** the phone camera is a webcam in every major meeting app.
**Done when:** Zoom, Teams, Google Meet (Chrome) and OBS show the Mikey camera; flipping front/back mid-call works with no black flash; the phone is held in any orientation and the image stays upright.

Android
- [ ] CameraX capture → JPEG → VIDEO frames; KEEP_ONLY_LATEST
- [ ] Camera half: live state with lens label (no video on the phone), flip button (top-left, only when on)
- [ ] Aspect ratio + quality + fps settings; auto quality by level; thermal back-off
- [ ] Camera FGS type added only while camera is on
- [ ] Camera disabled with explanation on Level 3

PC
- [x] JPEG decode → virtual camera (softcam / v4l2loopback)
- [x] Hold-last-frame during flip; placeholder frame when camera off
- [x] Letterboxing on mid-use aspect change
- [x] "Show preview" window
- [x] Installer registers softcam; Linux v4l2loopback setup + docs

## Phase 4 — Audio quality: noise & echo

**Goal:** clean audio in noisy rooms and on speakerphone calls.
**Done when:** with PC speakers at normal volume, the far end hears no echo on USB and only minor residual echo on Wi‑Fi; typing and fan noise are clearly reduced with noise suppression on.

Android
- [ ] Noise suppression toggle + strength slider (control message only — phone does no DSP)
- [ ] Echo cancellation toggle; noise gate threshold slider (Advanced)
- [ ] Settings greyed out when the PC doesn't report the capability

PC
- [x] Noise gate, RNNoise (`nnnoiseless`)
- [x] Loopback reference capture; SpeexDSP AEC with delay hint from jitter buffer
- [x] Echo reference device choice in tray flyout
- [x] Quality evaluation; decision recorded here on whether to move to WebRTC AEC3

## Phase 5 — Hardening & release

**Goal:** a stranger installs and uses Mikey in under 5 minutes.
**Done when:** v1.0.0 is tagged with signed APK + Windows installer + Linux packages attached, README complete, and the test matrix ([test-matrix.md](../rules/test-matrix.md)) passes.

- [ ] Optional: AES-GCM on Level 4; UDP audio path if measurements justify it ([wire-protocol.md](../architecture/wire-protocol.md))
- [ ] Optional: Media Foundation virtual camera on Windows 11
- [ ] "Open Mikey on phone when plugged in" (ADB `am start`)
- [ ] OEM battery-optimization guidance in the app (one-time, only on known-aggressive OEMs)
- [ ] GitHub Actions: APK build/sign; `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu` builds; installers
- [ ] `README.md` (one GIF, three steps), `INSTALL_*`, `TROUBLESHOOTING.md`, `CONTRIBUTING.md`
- [ ] App icon (flat, single stroke weight), `versionCode`/`versionName`
- [ ] Tag `v1.0.0`; repo description: *"Use your Android phone as a mic and webcam for your PC — over USB, Bluetooth or Wi‑Fi. One tap. Open source."*
