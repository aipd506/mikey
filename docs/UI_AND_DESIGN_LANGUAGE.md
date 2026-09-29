# UI Design Language & Styling Specification

Mikey’s visual design is governed by **pure functional minimalism**. Both the Android client and the PC companion flyout share an identical, disciplined design system rooted in **OLED pure black**, crisp typographic hierarchy, and zero visual bloat.

---

## 1. Design Principles & Anti-Bloat Invariants

1. **Zero Skeuomorphism**: No glassmorphism, no frosted glass blurs, no glowing drop shadows, and no multi-stop gradients.
2. **Zero Decorative Motion**: No spring physics, no bouncing cards, and no distracting layout animations. Micro-transitions are immediate or linear (≤ 150 ms).
3. **Calm, Actionable Microcopy**: Never display raw error codes or cryptic stack traces. Copy is polite, concise, and tells the user exactly what physical action to take:
   - *Bad*: `Error 0x80004005: WinSock connection refused on port 7653`
   - *Good*: `Plug in USB cable or connect to the same Wi-Fi`

---

## 2. Shared Color Palette

The color system is unified across Kotlin Compose on Android (`Palette.kt`) and the native Win32 GDI/GDI+ renderer on PC (`palette.rs`):

| Color Token | Hex Code | Android Representation | PC GDI+ Representation | Semantic Usage |
| :--- | :---: | :--- | :--- | :--- |
| **`bg`** | `#000000` | `Color(0xFF000000)` | `ARGB(255, 0, 0, 0)` | Root screen and card canvas background |
| **`surface`** | `#111111` | `Color(0xFF111111)` | `ARGB(255, 17, 17, 17)` | Settings bottom sheets and card elevated surfaces |
| **`tile`** | `#1C1C1C` | `Color(0xFF1C1C1C)` | `ARGB(255, 28, 28, 28)` | Interactive toggle tiles and buttons |
| **`hairline`** | `#2A2A2A` | `Color(0xFF2A2A2A)` | `ARGB(255, 42, 42, 42)` | Subtle 1px dividers and borders |
| **`text`** | `#FFFFFF` | `Color(0xFFFFFFFF)` | `ARGB(255, 255, 255, 255)` | Primary titles, icons, and active values |
| **`textSecondary`**| `#8E8E93` | `Color(0xFF8E8E93)` | `ARGB(255, 142, 142, 147)` | Captions, connection level subtitles, and hints |
| **`inactive`** | `#3A3A3C` | `Color(0xFF3A3A3C)` | `ARGB(255, 58, 58, 60)` | Disabled icons and unselected states |
| **`live`** | `#D71921` | `Color(0xFFD71921)` | `ARGB(255, 215, 25, 33)` | **On-Air indicator**: Active when PC is actively receiving |
| **`statusOk`** | `#30D158` | `Color(0xFF30D158)` | `ARGB(255, 48, 209, 88)` | Connection live, healthy stream |
| **`statusWait`** | `#FFD60A` | `Color(0xFFFFD60A)` | `ARGB(255, 255, 214, 10)` | Pending authorization, searching for PC |
| **`statusErr`** | `#FF453A` | `Color(0xFFFF453A)` | `ARGB(255, 255, 69, 58)` | Connection lost, permission denied |

---

## 3. Android UI Implementation (`android/app/src/main/java/com/mikey/ui/`)

### 3.1 Split Screen Layout (`MainScreen.kt`)
The primary screen is split into two massive, thumb-friendly touch targets:
- **Top Half (Microphone Tile)**: Toggles the microphone ON/OFF. Features a real-time reactive volume level meter bar that mirrors physical microphone input levels.
- **Bottom Half (Camera Tile)**: Toggles the camera ON/OFF. Features a subtle camera lens flip glyph and an elegant dark dot-grid texture (`#1A1A1A`).
- **Header Bar**: Displays the connection status dot (`StatusDot.kt`), the active transport level badge (e.g. `USB 1`, `Wi-Fi 3`), and a cog button to slide up settings.

### 3.2 Typography & Custom Vector Glyphs (`Type.kt`, `Glyphs.kt`)
- **Typography**: Clean sans-serif hierarchy using Android's native system font with strict weights: Large Header (28sp, Bold), Section Title (17sp, SemiBold), Body (14sp, Regular), and Caption (12sp, Medium).
- **Custom Vector Glyphs**: Clean, geometric SVG-based vector glyphs for Microphone, Muted Mic, Camera, Flip Lens, USB Cable, Wi-Fi Wave, and Bluetooth Icon.

---

## 4. PC Companion Flyout (`pc/src/flyout/`)

Unlike modern desktop utilities that package multi-hundred-megabyte Chromium/Electron bundles, Mikey’s PC companion features a **custom, ultra-lightweight native Win32 GDI/GDI+ software rendering engine**.

```text
 ┌───────────────────────────────────────────┐
 │  ● Pixel 8 Pro · Level 1 (USB)      [⚙]   │  ◄── Header (36px)
 ├───────────────────────────────────────────┤
 │  [ 🎙 Microphone: ON ]                    │
 │  Level: [■■■■■■■■■■■■□□□□□□□□]    [MUTE]   │  ◄── Mic Section (76px)
 ├───────────────────────────────────────────┤
 │  [ 📹 Camera: ON (1080p30) ]              │
 │  [ Preview Window ]        [ Flip Lens ]  │  ◄── Video Section (68px)
 ├───────────────────────────────────────────┤
 │  Noise Reduction: 80%                     │
 │  [══════════════════════●═════════]       │  ◄── DSP Section (48px)
 ├───────────────────────────────────────────┤
 │  ▼ Settings & Advanced Transports         │  ◄── Collapsible Footer
 └───────────────────────────────────────────┘
```

### 4.1 Native Double-Buffered Rendering (`render.rs`, `gdi.rs`)
To prevent window flicker during fast volume meter repaints:
1. `CreateCompatibleDC` creates an off-screen memory device context.
2. `CreateCompatibleBitmap` allocates an off-screen surface matching the flyout's dimensions.
3. GDI+ renders smooth anti-aliased text (`ClearTypeGridFit`) and geometric cards.
4. A single `BitBlt` transfers the completed buffer to the screen during `WM_PAINT`.
5. The flyout binary overhead is **zero megabytes of extra webview dependencies**.

### 4.2 Dynamic Flyout Geometry (`layout.rs`)
- **Width**: Fixed at **320 px** (`FLYOUT_WIDTH = 320`).
- **Corner Radius**: **8 px** (`FLYOUT_CORNER_RADIUS = 8`), clipped via `CreateRoundRectRgn`.
- **Collapsed Height**: **184 px** (`FLYOUT_HEIGHT_COLLAPSED = 184`).
- **Expanded Height**: Dynamically calculated up to **480 px** depending on active camera preview buttons, DSP controls, and pending TOFU prompts (`compute_flyout_height`).
- **Positioning (`calculate_flyout_position`)**: Automatically anchored adjacent to the Windows taskbar notification tray rect with fallback to cursor coordinates.

### 4.3 Asset-Driven Vector Icons (`pc/src/flyout/icons/`)
Vector icons in the PC flyout are not hardcoded coordinate arrays; they are embedded at compile time directly from official Lucide SVG source files in `pc/assets/icons/` (`include_str!`). A zero-dependency, real-time SVG micro-parser and renderer (`pc/src/flyout/icons/svg.rs` and `path.rs`) maps standard 24×24 SVG elements (`<line>`, `<circle>`, `<rect>`, `<polygon>`, and `<path>` commands `M/m`, `L/l`, `H/h`, `V/v`, `A/a`, and `Z/z`) directly to anti-aliased GDI+ vector draw primitives (`GdipDrawLine`, `GdipDrawArc`, `GdipDrawEllipse`) scaled to target bounds.
