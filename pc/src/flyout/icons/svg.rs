//! SVG Vector parser and renderer for Mikey Flyout.
//! Parses 24x24 standard Lucide SVG assets from pc/assets/icons.

#![cfg(windows)]

use super::canvas::{create_vector_pen, SvgCanvas};
use super::path::parse_and_draw_path;
use crate::flyout::win32::Gdiplus;
use std::ffi::c_void;

pub fn render_svg(
    g: &Gdiplus,
    graphics: *mut c_void,
    svg_str: &str,
    (cx, cy): (f32, f32),
    size: f32,
    color: u32,
    stroke_width: f32,
) {
    let pen = create_vector_pen(g, color, stroke_width);
    if pen.is_null() {
        return;
    }
    let canvas = SvgCanvas::new(g, graphics, pen, cx, cy, size);
    render_svg_canvas(&canvas, svg_str);
    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

fn find_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let bytes = tag.as_bytes();
    let n = name.as_bytes();
    let mut i = 0;
    while i + n.len() + 2 <= bytes.len() {
        if (i == 0 || bytes[i - 1].is_ascii_whitespace())
            && &bytes[i..i + n.len()] == n
            && bytes[i + n.len()] == b'='
            && bytes[i + n.len() + 1] == b'"'
        {
            let start = i + n.len() + 2;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'"' {
                end += 1;
            }
            return tag.get(start..end);
        }
        i += 1;
    }
    None
}

fn render_svg_canvas(canvas: &SvgCanvas, svg: &str) {
    let mut i = 0;
    let bytes = svg.as_bytes();
    while let Some(start) = svg[i..].find('<') {
        let tag_start = i + start + 1;
        if tag_start >= bytes.len() {
            break;
        }
        if bytes[tag_start] == b'?' || bytes[tag_start] == b'!' || bytes[tag_start] == b'/' {
            i = tag_start;
            continue;
        }
        if let Some(tag_len) = svg[tag_start..].find('>') {
            let tag = &svg[tag_start..tag_start + tag_len];
            let name = tag
                .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
                .next()
                .unwrap_or("");
            match name {
                "line" => {
                    let x1: f32 = find_attr(tag, "x1")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let y1: f32 = find_attr(tag, "y1")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let x2: f32 = find_attr(tag, "x2")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let y2: f32 = find_attr(tag, "y2")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    if (x2 - x1).hypot(y2 - y1) < 0.05 {
                        canvas.line(x1, y1, x1 + 0.1, y1);
                    } else {
                        canvas.line(x1, y1, x2, y2);
                    }
                }
                "circle" => {
                    let cx: f32 = find_attr(tag, "cx")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let cy: f32 = find_attr(tag, "cy")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let r: f32 = find_attr(tag, "r")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    canvas.circle(cx, cy, r);
                }
                "rect" => {
                    let x: f32 = find_attr(tag, "x")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let y: f32 = find_attr(tag, "y")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let w: f32 = find_attr(tag, "width")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let h: f32 = find_attr(tag, "height")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let rx: f32 = find_attr(tag, "rx")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    if rx > 0.0 {
                        canvas.round_rect(x, y, w, h, rx);
                    } else {
                        canvas.line(x, y, x + w, y);
                        canvas.line(x + w, y, x + w, y + h);
                        canvas.line(x + w, y + h, x, y + h);
                        canvas.line(x, y + h, x, y);
                    }
                }
                "polygon" => {
                    if let Some(pts_str) = find_attr(tag, "points") {
                        let pts: Vec<f32> = pts_str
                            .split_whitespace()
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        let count = pts.len() / 2;
                        for idx in 0..count {
                            let (x1, y1) = (pts[idx * 2], pts[idx * 2 + 1]);
                            let (x2, y2) = (
                                pts[((idx + 1) % count) * 2],
                                pts[((idx + 1) % count) * 2 + 1],
                            );
                            if (x2 - x1).hypot(y2 - y1) > 0.001 {
                                canvas.line(x1, y1, x2, y2);
                            }
                        }
                    }
                }
                "path" => {
                    if let Some(d) = find_attr(tag, "d") {
                        parse_and_draw_path(canvas, d);
                    }
                }
                _ => {}
            }
            i = tag_start + tag_len + 1;
        } else {
            break;
        }
    }
}
