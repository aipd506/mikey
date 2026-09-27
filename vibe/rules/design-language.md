# 8. Design Language

## 8.1 Principles

- **Flat, not glossy.** No glassmorphism, gradients or drop shadows.
- **Honest, binary states.** On or off. The only exceptions are the amber "waiting" state and the mic level ring.
- **The screen is the button.** Each half of the phone screen is a tap target.
- **Rotation aware, not rotation reactive.** Glyphs rotate; the layout doesn't.
- **System font only.** No custom typefaces.
- **No decorative animation.** State changes snap. The only motion: drawer slide (standard bottom-sheet), level ring.

## 8.2 Color palette

| Token | Hex | Usage |
|-------|-----|-------|
| `bg` | `#000000` | Background — pure black, OLED-friendly |
| `surface` | `#111111` | Drawer, dialogs |
| `divider` | `#2C2C2E` | Separators, split line |
| `icon-off` | `#3A3A3C` | Dimmed mic/camera/flip/chevron |
| `mic-on` | `#30D158` | Mic on + level ring |
| `cam-on` | `#0A84FF` | Camera on |
| `status-ok` | `#30D158` | Connected & streaming path live |
| `status-wait` | `#FFD60A` | Waiting for approval / authorization |
| `status-err` | `#FF453A` | No PC |
| `text-primary` | `#FFFFFF` | Readable text |
| `text-secondary` | `#8E8E93` | Sub-labels, version string |

Pressed state: 0.08 white alpha overlay. Nothing else.

## 8.3 Typography

- System default (Roboto on most Android).
- Three sizes only: `17sp` (reserved), `14sp` drawer items, `12sp` metadata.
- Regular 400 everywhere; Medium 500 for drawer section headers.
- Sentence case. Never all-caps.

## 8.4 Icons & sizes

- Mic/camera icons 56 dp; outline 2 dp stroke when off, filled when on.
- Flip button 40 dp touch target (24 dp glyph), 16 dp from top-left edges.
- Status dot 10 dp rounded square (3 dp radius), 16 dp from bottom-right edges.
- Chevron 24 dp glyph, 48 dp touch target, centered on the split line.
- All touch targets ≥ 48 dp.

## 8.5 PC Tray Flyout Design Specifications

- **Container:**
  - Compact borderless floating dialog (~350 px width, ~440–540 px dynamic height).
  - Background `#111111` (`surface`), outer border 1 px solid `#2C2C2E` (`divider`), 8 px corner radius.
  - No drop shadows, no blur/glassmorphism, no OS title bar.
- **Controls & Elements:**
  - **Live Audio VU Meter:** Horizontal bar (height 6 px), track `#2C2C2E`. Level bar renders `#30D158` (normal), `#FFD60A` (peak > -6 dB), `#FF453A` (clipping 0 dB). Instant attack, smooth decay.
  - **Gain / Volume Slider:** 4 px flat track (`#2C2C2E`), 12 px circular thumb (`#FFFFFF`), active fill `#30D158`.
  - **Toggle Switches:** Compact flat switches (32×18 px). Active background `#30D158`, inactive `#3A3A3C`, white circular thumb.
  - **Pill Badges:** Height 20 px (10 px radius), background `#2C2C2E`, text 10 px bold uppercase (`L1 USB`, `L4 WI-FI`).
  - **Embedded Camera Thumbnail:** 16:9 aspect box, `#000000` background, 1 px `#2C2C2E` border, 4 px corner radius.
- **Interaction Rules:**
  - Light dismiss: Closes on blur (`WM_KILLFOCUS`), `Esc` key, or tray icon re-click.
  - Hover state: 0.04 white alpha overlay.
  - Pressed state: 0.08 white alpha overlay.

