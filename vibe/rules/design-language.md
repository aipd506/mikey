# 8. Design Language

## 8.1 Principles

- **Flat, not glossy.** No glassmorphism, gradients or drop shadows.
- **Red means the PC is receiving.** The only red on the phone is the on-air dot on the mic and camera. Everything else is black, white and greys; the status dot keeps the colors it shares with the PC tray.
- **The screen is the button.** Each half of the phone screen is a tap target.
- **Say the state in words.** Under each circle a short label in capitals says what is happening (*MIC LIVE*, *CAMERA OFF · TAP TO START*), with one plain sentence when the user has to act.
- **Rotation aware, not rotation reactive.** Glyphs rotate; the layout doesn't.
- **Quiet motion.** Only short, functional transitions: the sheet slides in 180 ms, its scrim fades in 140 ms, a switch knob moves in 100 ms. The only live element is the mic's level ring.

## 8.2 Color palette

| Token | Hex | Usage |
|-------|-----|-------|
| `bg` | `#000000` | Background, OLED black |
| `sheet` | `#111111` | Settings sheet |
| `tile` | `#1C1C1C` | Grouped rows in the sheet; ring dots when off |
| `hairline` | `#2A2A2A` | Dividers, the split line, a switch that's off |
| `text` | `#FFFFFF` | Primary text; a mic or camera that's on |
| `text-secondary` | `#8E8E93` | Labels, hints, version string |
| `inactive` | `#3A3A3C` | Dimmed icons and outlines |
| `live` | `#D71921` | The on-air dot: the PC is receiving |
| `grid` | `#1A1A1A` | Dot texture behind the camera half |
| `status-ok` | `#30D158` | Status dot: streaming (as on the PC tray) |
| `status-wait` | `#FFD60A` | Status dot: waiting for approval |
| `status-err` | `#FF453A` | Status dot: no PC |

Pressed state: 0.08 white alpha overlay. Nothing else.

## 8.3 Typography

- **Geist** for text and **Geist Mono** for labels, bundled as variable fonts under the SIL Open Font License (`android/licenses/geist-OFL.txt`).
- Sizes: 17 sp medium for the sheet header, 15 sp for rows, 14 sp for segments and the sentence under a circle, 12.5 sp for row subtitles.
- State labels and section titles: Geist Mono 11 sp, capitals, 0.14 em letter spacing. Units keep their case (−45 dB).
- Sentence case everywhere else.

## 8.4 Icons & sizes

- Icons are drawn on a 24 grid with round caps and joins: the big mic and camera with a 0.9 stroke at 56 dp, everything else with 1.8.
- Mic and camera sit in 112 dp circles: dim outline when off, white outline when on but not reaching the PC, white with a black icon and the 14 dp red on-air dot when the PC receives. Dashed when unavailable or without permission.
- The mic's level ring: 48 dots of 5 dp on a 184 dp ring, filling from the bottom up with the voice.
- Flip button: 40 dp circle in a 48 dp target, top-left, only while the camera is on. Chevron: 36 dp circle on the split line.
- Status dot 10 dp rounded square (3 dp radius), 16 dp from the bottom-right edges.
- All touch targets ≥ 48 dp.

## 8.5 PC Tray Flyout Design Specifications

- **Container:**
  - Ultra-compact borderless floating dialog (300 px width, 220–330 px dynamic height).
  - Background `#111111` (`surface`), outer border 1 px solid `#2C2C2E` (`divider`), 16 px smooth corner radius.
  - Full anti-aliasing via GDI+ (`SmoothingModeAntiAlias`), subpixel ClearType text rendering.
  - No drop shadows, no blur/glassmorphism, no OS title bar.
- **Controls & Elements:**
  - **Minimal Typography & Soft Iconography:** Crisp Segoe UI vector fonts (11–15px), sentence case, zero unnecessary text labels. Native vector Heroicons (1.8px rounded stroke, round line caps/joins).
  - **Live Audio VU Meter:** Smooth horizontal pill (height 4 px) integrated directly into the microphone row, track `#1C1C1E`. Level bar renders `#30D158` (normal), `#FFD60A` (peak > -6 dB), `#FF453A` (clipping 0 dB). Instant attack, smooth decay.
  - **Unified Smooth Pills:** Rounded pill buttons (height 26 px, 13 px radius). Active background `#30D158` (audio) or `#0A84FF` (video), inactive background `#1C1C1E` with `#2C2C2E` border.
  - **Always-On AEC:** Echo cancellation runs permanently in the DSP background with WASAPI loopback reference. No UI toggle button is rendered, avoiding user confusion and saving horizontal space.
  - **Smooth Noise Suppression Slider:** 6 px rounded track (`#1C1C1E`), active fill `#30D158`, 14 px circular anti-aliased thumb with inner ring.
  - **Persistent Camera Option:** Visual preview pill button (`[ Preview ]`) always available to open/toggle floating camera preview window, even when camera is off/idle.
  - **Lens Flip Control:** Visual `[ Flip ]` pill button to trigger front/back camera switch remotely.
  - **Circular Action Buttons:** 28 px diameter round buttons in footer for logs, disconnect, and power.
  - **Embedded Camera Thumbnail:** 16:9 aspect box, `#000000` background, 1 px `#2C2C2E` border, 8 px corner radius.
- **Interaction Rules:**
  - Light dismiss: Closes on blur (`WM_KILLFOCUS`), `Esc` key, or tray icon re-click.
  - Hover state: 0.05 white alpha overlay (`#2C2C2E` highlight).
  - Pressed state: 0.08 white alpha overlay.

