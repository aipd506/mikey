# Feasibility & Viability

## Verdict summary

| Capability | Verdict | Notes |
|---|---|---|
| Phone mic → PC virtual mic | ✅ Feasible | Proven pattern (WO Mic, AudioRelay). Needs a virtual audio device on Windows (VB-Cable, user-installed). |
| Phone camera → PC virtual webcam | ✅ Feasible, some effort | Windows: DirectShow virtual cam via `softcam` (Win10/11); Media Foundation virtual camera (Win11 only) as a later upgrade. Linux: `v4l2loopback`. |
| **Level 1 — USB debugging (ADB)** | ✅ Feasible, best quality | `adb reverse` tunnels the phone's `localhost:7653` to the PC. Truly plug-and-play *after* the user has enabled USB debugging once and tapped "Allow" once. Minority of users have debugging on. |
| **Level 2 — USB tethering** | ✅ Feasible | Standard network interface over USB. Subnet is **not** always `192.168.42.x` — must be discovered, not hardcoded. User must toggle tethering (Android does not let apps turn it on). |
| **Level 3 — Bluetooth** | ⚠️ Feasible for **audio only** | Classic Bluetooth RFCOMM gives ~0.5–1.5 Mbps in practice. Enough for Opus voice (32–64 kbps); **not enough for usable video**. Camera is disabled on this level with a clear message. |
| **Level 4 — Wi‑Fi / LAN** | ✅ Feasible | UDP broadcast discovery + TCP session. Blocked by client-isolated networks (some offices, hotels, guest Wi‑Fi) → manual IP fallback in Advanced. |
| Automatic upgrade between levels | ✅ Feasible | Event-driven probing on the phone + make-before-break session handover ([connection-levels.md](../architecture/connection-levels.md)). |
| Noise suppression | ✅ Feasible | RNNoise on the **PC** (`nnnoiseless`, pure Rust, ~1–2% of a core). The phone sends raw audio. "Threshold" is implemented as a noise gate. |
| Echo cancellation | ⚠️ Feasible, **highest quality risk** | Must run on the **PC** (the echo comes from PC speakers; the phone can't hear the reference). Variable network latency and clock drift make AEC hard. Works best on USB. |
| Ask-before-join + tray prompt | ✅ Feasible | Native OS notification with Allow/Deny; tray flyout card fallback. |
| Phone settings persistence | ✅ Trivial | `SharedPreferences`. |
| Persistent notification while minimized | ✅ Required anyway | Android foreground service. |
| Swipe-away from Recents kills connection | ✅ Feasible | `stopWithTask` + `onTaskRemoved()`. A few OEM skins behave oddly; tested per device. |
| macOS PC side | ❌ Out of scope for v1 | Virtual camera needs a system extension; virtual mic needs a HAL plugin. Possible later. |

## Viability

- **Market:** DroidCam, Iriun, WO Mic and AudioRelay exist, and newer Windows 11 builds and some Pixel phones offer built-in phone-as-webcam features. None of them combine *mic + camera + four automatic transports + zero-ritual startup + open source + Linux*. Mikey's edge is not a feature list — it is **how little the user has to do**.
- **Cost to run:** zero. No servers, no relay, no accounts. Distribution via GitHub releases (and optionally F-Droid).
- **Maintenance burden:** the real cost is platform churn — Android foreground-service rules change almost every release, and virtual devices depend on third-party drivers. The architecture isolates both behind small modules so they can be patched alone.
- **Biggest adoption friction:** the one-time install of virtual device drivers on the PC (VB-Cable on Windows, `v4l2loopback` on Linux). The PC app must detect missing pieces and guide the user in one click each.

## Corrections to the original PRD (why the architecture changed)

These are real problems in PRD v0 that would have caused bugs or blocked features:

1. **"Daemon as a Windows service / system-wide systemd unit" does not work for this app.** Windows services run in session 0: they cannot show a tray icon or notifications, and they don't have the user's audio context. On Linux, PulseAudio/PipeWire are per-user, so a system-level unit cannot reach the user's audio server. → **Mikey for PC is now a per-user tray app** that autostarts at login (HKCU `Run` key on Windows, XDG autostart on Linux). No admin rights needed for the app itself.
2. **"No UI on the PC" conflicts with rich controls, ask-before-join and preview.** → A compact, anchored tray flyout companion app (a borderless dialog positioned directly adjacent to the tray icon). Provides interactive controls (live VU meter, volume sliders, DSP toggles, device switcher, embedded preview) with zero persistent taskbar clutter and light-dismiss on blur.
3. **USB tethering subnet is not fixed.** Many Android versions randomize the tether subnet. → Discovery over the tether interface instead of assuming `192.168.42.1`.
4. **Opus over TCP does not "handle packet loss gracefully".** TCP never shows loss to the app; it shows *delay*. Opus' loss concealment only helps over UDP. → v1 uses TCP everywhere with a latency cap (stale audio is dropped at the PC); an optional UDP media path for Wi‑Fi is planned ([wire-protocol.md](../architecture/wire-protocol.md)).
5. **mDNS via `NsdManager` can't be pinned to an interface** (needed for tether vs. Wi‑Fi). → Replaced with a tiny UDP broadcast beacon that works identically on every IP interface and removes the `mdns-sd` dependency.
6. **OBS Virtual Camera as a dependency is heavy** (a ~300 MB app) and injecting frames into it relies on OBS internals. → `softcam` (MIT, a small DirectShow DLL) on Windows.
7. **Echo cancellation on the phone can't work** for PC-speaker echo. → AEC stays on the PC, but the setting is exposed on the phone (settings sync over the control channel).
8. **Clock drift was not handled.** Phone mic and PC audio clocks differ slightly; over a long call a buffer grows or starves. → Adaptive resampling in the jitter buffer ([media-pipeline.md](../architecture/media-pipeline.md)).
9. **Open Wi‑Fi joining is a security hole.** With ask-before-join off, anyone on the same café/dorm Wi‑Fi with Mikey installed could pipe audio into your mic. → Trust-on-first-use for Wi‑Fi ([sessions-trust.md](../architecture/sessions-trust.md)), invisible for single-device users after the first click.
