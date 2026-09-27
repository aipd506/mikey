# 24. Changelog vs. PRD v0

> 2026-09-26: `MASTER.md` was split into the files in `vibe/` (one per section, grouped into `core/`, `architecture/` and `rules/`, plus playbooks in `skills/`) and removed from the repo. The list below is the earlier change from PRD v0 to `MASTER.md`.

- Renamed `PRD.md` → `MASTER.md`; added motto, empathy, principles, feasibility, risks, open decisions.
- **Added the four connection levels** (USB debugging > USB tethering > Bluetooth > Wi‑Fi) with automatic upgrade/downgrade.
- PC component changed from a boot service to a **per-user tray app** with a small menu, ask-before-join and preview window.
- **PC UI upgraded to Tray Flyout Companion App** ([pc-ux.md](pc-ux.md)): Replaced the static, boring OS context menu with an anchored, borderless dialog window at the tray icon location, enabling real-time audio VU metering, volume sliders, DSP switches, device switcher, inline join approvals, and embedded 16:9 preview.
- Replaced mDNS with a UDP beacon; removed the hardcoded `192.168.42.x` assumption.
- Replaced OBS Virtual Camera with softcam; Linux mic is now a proper virtual source.
- Protocol: added handshake, tokens, PENDING/REJECT/BYE, media header (seq + timestamp), frame size limit; heartbeat 2 s / timeout 6 s.
- Audio moved to 48 kHz; added drift resampling, latency cap, noise gate, RNNoise, AEC settings synced from phone.
- **All audio processing moved to the PC**; phone captures raw (AAudio). Format per level: raw PCM on USB, Opus 96 kbps / 10 ms on Wi‑Fi, Opus 32–48 kbps on Bluetooth. Noise suppression simplified to one toggle.
- Background blur/removal explicitly excluded.
- Phone UI: chevron moved to the center split line; flip button top-left (opposite the status dot); amber waiting state; mic level ring; Advanced section; persistence table; notification actions and swipe-away behavior; Android 14+ FGS rules.
- Added performance budgets, security section, test matrix and a 5-phase roadmap starting with the #1 priority level.
- **No camera preview on the phone** (2026-09-27): the camera half shows a live state and lens label instead; the video is shown only on the PC.
