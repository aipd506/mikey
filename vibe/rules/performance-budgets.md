# 15. Performance Budgets

These are acceptance criteria, checked before every phase is closed.

| Metric | Budget |
|---|---|
| APK size (per ABI) | ≤ 6 MB |
| `mikey` binary | ≤ 10 MB (excluding bundled `adb`) |
| PC idle RAM (tray, flyout closed) | ≤ 15 MB |
| PC active RAM (flyout open with preview) | ≤ 30 MB |
| PC idle CPU | ~0% (no polling loops; event/blocking only) |
| PC CPU streaming mic + 720p30 camera + AEC | ≤ 5% of one modern core |
| Phone battery, mic only, screen off | ≤ 4% per hour (reference: mid-range phone) |
| Mouth-to-virtual-mic latency | L1/L2 ≤ 60 ms · L4 ≤ 120 ms · L3 ≤ 200 ms (p95) |
| Camera glass-to-virtual-cam latency | L1/L2 ≤ 120 ms · L4 ≤ 200 ms (p95) |
| Cold connect (app open → dot green) | ≤ 2 s on a known PC |
| Transport upgrade glitch | ≤ 300 ms |
| Recovery after cable pull (Wi‑Fi available) | ≤ 2 s |

Tools: `cargo bloat` before adding crates; Android Studio profiler + `adb shell dumpsys batterystats` per phase.
