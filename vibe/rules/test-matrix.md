# 19. Test Matrix

Emulators and CI can't test foreground services, USB, Bluetooth or virtual devices meaningfully. These run on real hardware at the end of each phase.

| Area | Cases |
|---|---|
| Levels | Each of L1 to L4 alone; every upgrade (3→1, 3→2, 4→1, 4→3…) and downgrade (cable pull, Wi‑Fi off, BT off) mid-call |
| Lifecycle | Minimize; screen off 30 min; swipe from Recents; Stop from notification; phone reboot; PC sleep/wake; PC logout/login |
| Trust | New device each level with ask-before-join off/on; second device while streaming; Forget on each side; token mismatch |
| Android versions | 8.0, 10, 12, 13, 14, 15/16 (FGS rules, permissions) |
| OEMs | Pixel, Samsung, Xiaomi/Redmi, OnePlus/Oppo (battery killers, swipe-away behavior, tether subnet) |
| Networks | Home Wi‑Fi, phone hotspot with PC joined, client-isolated network (manual IP), 2.4 GHz congested |
| Apps | Zoom, Teams, Google Meet (Chrome/Edge/Firefox), Discord, OBS, Audacity |
| PC | Windows 10, Windows 11; Ubuntu (GNOME), Fedora (GNOME, PipeWire), Kubuntu/Arch (KDE, Wayland) |
| Long run | 3-hour call on each level: no drift growth, no memory growth, no stall |
