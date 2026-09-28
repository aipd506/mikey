# 22. Risk Register

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Echo cancellation quality poor over Wi‑Fi jitter / clock drift | High | Medium | Delay hints from jitter buffer; drift resampler; document "best on USB"; WebRTC AEC3 as fallback engine |
| R2 | OEM battery managers kill the service (Xiaomi, Oppo, some Samsung) | Medium | High | Foreground service + connectedDevice type; one-time guidance on known OEMs; link to dontkillmyapp.com |
| R3 | Android FGS/permission rules change again | High | Medium | All FGS logic in `MikeyService` + `Notifier`; test on newest Android beta each phase |
| R4 | Virtual camera not visible in some apps (Windows Camera app, some UWP) | Medium | Medium | softcam covers major meeting apps; MF virtual camera on Win11 in Phase 5 |
| R5 | ADB version conflict with a developer's own Android SDK | Medium | Low | Prefer `adb` on PATH; bundled only as fallback |
| R6 | Bluetooth RFCOMM unreliable on some PC adapters | Medium | Low | It's level 4 of 4; clear fallback; audio only |
| R7 | USB tethering routes PC internet via mobile data | Medium | Medium | One-time tip; Level 1 preferred when available |
| R8 | GNOME hides tray icons | Medium | Medium | Document AppIndicator extension; notifications still work; `mikey --settings` CLI fallback |
| R9 | Secure Boot blocks unsigned v4l2loopback | Medium | Medium | Recommend distro DKMS package (auto-signed on Ubuntu/Fedora with MOK); troubleshooting doc |
| R10 | VB-Cable can't be bundled | Certain | Low | One-click guided install + auto-detect; long-term: evaluate an MIT virtual audio driver |
| R11 | Client-isolated Wi‑Fi blocks discovery | Medium | Low | Last-IP fast path, manual IP in Advanced, USB levels |
| R12 | Scope creep ("just one more setting") | High | High | Principle 2: anything new goes under Advanced or doesn't ship |
