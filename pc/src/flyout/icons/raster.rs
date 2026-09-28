//! High-definition anti-aliased icon rasterizer for Mikey based on Lucide SVG geometry.
//! Produces crystal-clear 32x32 RGBA buffers with dark contour for any taskbar theme.

use std::f32::consts::{PI, TAU};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIconMode {
    Idle,
    Active,
    Pending,
}

#[inline]
fn dist_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let vx = bx - ax;
    let vy = by - ay;
    let wx = px - ax;
    let wy = py - ay;
    let c1 = wx * vx + wy * vy;
    if c1 <= 0.0 {
        return (px - ax).hypot(py - ay);
    }
    let c2 = vx * vx + vy * vy;
    if c2 <= c1 {
        return (px - bx).hypot(py - by);
    }
    let t = c1 / c2;
    (px - (ax + t * vx)).hypot(py - (ay + t * vy))
}

#[inline]
fn dist_arc(px: f32, py: f32, cx: f32, cy: f32, r: f32, a_start: f32, a_end: f32) -> f32 {
    let dx = px - cx;
    let dy = py - cy;
    let d = dx.hypot(dy);
    let mut angle = dy.atan2(dx);
    if angle < 0.0 {
        angle += TAU;
    }
    if angle >= a_start && angle <= a_end {
        return (d - r).abs();
    }
    let d1 = (px - (cx + r * a_start.cos())).hypot(py - (cy + r * a_start.sin()));
    let d2 = (px - (cx + r * a_end.cos())).hypot(py - (cy + r * a_end.sin()));
    d1.min(d2)
}

#[inline]
fn dist_capsule(px: f32, py: f32, cx: f32, top_y: f32, bot_y: f32, r: f32) -> f32 {
    dist_segment(px, py, cx, top_y + r, cx, bot_y - r) - r
}

/// Generates 32x32 RGBA pixels for Mikey's microphone tray icon.
pub fn generate_mikey_tray_rgba(mode: TrayIconMode) -> Vec<u8> {
    const SIZE: usize = 32;
    let mut rgba = vec![0u8; SIZE * SIZE * 4];

    let cx = 16.0f32;
    let cap_top = 4.0f32;
    let cap_bot = 16.0f32;
    let cap_r = 4.5f32;

    let cradle_cy = 14.5f32;
    let cradle_r = 8.5f32;

    let stem_top = 23.0f32;
    let stem_bot = 27.5f32;

    let base_y = 27.5f32;
    let base_half = 5.5f32;
    let stroke = 2.2f32;

    let (mic_color, filled) = match mode {
        TrayIconMode::Active => ((48u8, 209u8, 88u8), true),
        TrayIconMode::Pending => ((255u8, 214u8, 10u8), true),
        TrayIconMode::Idle => ((255u8, 255u8, 255u8), false),
    };

    for y in 0..SIZE {
        for x in 0..SIZE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let d_cap_raw = dist_capsule(px, py, cx, cap_top, cap_bot, cap_r);
            let (cov_cap_fg, cov_cap_bg) = if filled {
                (
                    (0.5 - d_cap_raw).clamp(0.0, 1.0),
                    (0.5 - (d_cap_raw - 1.0)).clamp(0.0, 1.0),
                )
            } else {
                let d_cap = d_cap_raw.abs();
                (
                    (0.5 - (d_cap - stroke / 2.0)).clamp(0.0, 1.0),
                    (0.5 - (d_cap - (stroke / 2.0 + 1.0))).clamp(0.0, 1.0),
                )
            };

            let d_cradle_arc = dist_arc(px, py, cx, cradle_cy, cradle_r, 0.0, PI);
            let d_ext_r = dist_segment(
                px,
                py,
                cx + cradle_r,
                cradle_cy - 4.0,
                cx + cradle_r,
                cradle_cy,
            );
            let d_ext_l = dist_segment(
                px,
                py,
                cx - cradle_r,
                cradle_cy - 4.0,
                cx - cradle_r,
                cradle_cy,
            );
            let d_cradle = d_cradle_arc.min(d_ext_r).min(d_ext_l);
            let cov_cradle_fg = (0.5 - (d_cradle - stroke / 2.0)).clamp(0.0, 1.0);
            let cov_cradle_bg = (0.5 - (d_cradle - (stroke / 2.0 + 1.0))).clamp(0.0, 1.0);

            let d_stem = dist_segment(px, py, cx, stem_top, cx, stem_bot);
            let cov_stem_fg = (0.5 - (d_stem - stroke / 2.0)).clamp(0.0, 1.0);
            let cov_stem_bg = (0.5 - (d_stem - (stroke / 2.0 + 1.0))).clamp(0.0, 1.0);

            let d_base = dist_segment(px, py, cx - base_half, base_y, cx + base_half, base_y);
            let cov_base_fg = (0.5 - (d_base - stroke / 2.0)).clamp(0.0, 1.0);
            let cov_base_bg = (0.5 - (d_base - (stroke / 2.0 + 1.0))).clamp(0.0, 1.0);

            let cov_fg = cov_cap_fg
                .max(cov_cradle_fg)
                .max(cov_stem_fg)
                .max(cov_base_fg);
            let cov_bg = cov_cap_bg
                .max(cov_cradle_bg)
                .max(cov_stem_bg)
                .max(cov_base_bg);

            let idx = (y * SIZE + x) * 4;
            if cov_fg > 0.01 {
                let (mut r, mut g, mut b) = mic_color;
                let edge = cov_bg - cov_fg;
                if edge > 0.0 {
                    r = (r as f32 * (1.0 - 0.3 * edge)) as u8;
                    g = (g as f32 * (1.0 - 0.3 * edge)) as u8;
                    b = (b as f32 * (1.0 - 0.3 * edge)) as u8;
                }
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = (cov_fg * 255.0) as u8;
            } else if cov_bg > 0.05 {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = (cov_bg * 90.0) as u8;
            }
        }
    }

    rgba
}

#[cfg(windows)]
pub fn create_mikey_tray_icon(mode: TrayIconMode) -> tray_icon::Icon {
    let rgba = generate_mikey_tray_rgba(mode);
    tray_icon::Icon::from_rgba(rgba, 32, 32).expect("create Mikey tray icon")
}
