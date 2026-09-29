//! SVG path syntax tokenizer, command interpreter, and arc math.

#![cfg(windows)]

use super::canvas::SvgCanvas;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token {
    Cmd(char),
    Num(f32),
}

fn tokenize_path(d: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = d.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() || b == b',' {
            i += 1;
            continue;
        }
        if b.is_ascii_alphabetic() {
            tokens.push(Token::Cmd(b as char));
            i += 1;
            continue;
        }
        if b == b'-' || b == b'+' || b == b'.' || b.is_ascii_digit() {
            let start = i;
            if b == b'-' || b == b'+' {
                i += 1;
            }
            let mut has_dot = false;
            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_digit() {
                    i += 1;
                } else if c == b'.' && !has_dot {
                    has_dot = true;
                    i += 1;
                } else if (c == b'e' || c == b'E') && i + 1 < bytes.len() {
                    i += 1;
                    if bytes[i] == b'+' || bytes[i] == b'-' {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            if let Ok(val) = d[start..i].parse::<f32>() {
                tokens.push(Token::Num(val));
            }
            continue;
        }
        i += 1;
    }
    tokens
}

pub fn parse_and_draw_path(canvas: &SvgCanvas, d: &str) {
    let tokens = tokenize_path(d);
    let mut curr_x = 0.0f32;
    let mut curr_y = 0.0f32;
    let mut start_x = 0.0f32;
    let mut start_y = 0.0f32;
    let mut idx = 0;
    let mut active_cmd = ' ';

    while idx < tokens.len() {
        match tokens[idx] {
            Token::Cmd(c) => {
                active_cmd = c;
                idx += 1;
            }
            Token::Num(_) => {
                if active_cmd == 'm' {
                    active_cmd = 'l';
                } else if active_cmd == 'M' {
                    active_cmd = 'L';
                }
            }
        }

        match active_cmd {
            'M' => {
                if idx + 1 < tokens.len() {
                    if let (Token::Num(x), Token::Num(y)) = (tokens[idx], tokens[idx + 1]) {
                        curr_x = x;
                        curr_y = y;
                        start_x = x;
                        start_y = y;
                        idx += 2;
                    }
                } else {
                    break;
                }
            }
            'm' => {
                if idx + 1 < tokens.len() {
                    if let (Token::Num(dx), Token::Num(dy)) = (tokens[idx], tokens[idx + 1]) {
                        curr_x += dx;
                        curr_y += dy;
                        start_x = curr_x;
                        start_y = curr_y;
                        idx += 2;
                    }
                } else {
                    break;
                }
            }
            'L' => {
                if idx + 1 < tokens.len() {
                    if let (Token::Num(x), Token::Num(y)) = (tokens[idx], tokens[idx + 1]) {
                        canvas.line(curr_x, curr_y, x, y);
                        curr_x = x;
                        curr_y = y;
                        idx += 2;
                    }
                } else {
                    break;
                }
            }
            'l' => {
                if idx + 1 < tokens.len() {
                    if let (Token::Num(dx), Token::Num(dy)) = (tokens[idx], tokens[idx + 1]) {
                        let nx = curr_x + dx;
                        let ny = curr_y + dy;
                        canvas.line(curr_x, curr_y, nx, ny);
                        curr_x = nx;
                        curr_y = ny;
                        idx += 2;
                    }
                } else {
                    break;
                }
            }
            'H' => {
                if idx < tokens.len() {
                    if let Token::Num(x) = tokens[idx] {
                        canvas.line(curr_x, curr_y, x, curr_y);
                        curr_x = x;
                        idx += 1;
                    }
                } else {
                    break;
                }
            }
            'h' => {
                if idx < tokens.len() {
                    if let Token::Num(dx) = tokens[idx] {
                        let nx = curr_x + dx;
                        canvas.line(curr_x, curr_y, nx, curr_y);
                        curr_x = nx;
                        idx += 1;
                    }
                } else {
                    break;
                }
            }
            'V' => {
                if idx < tokens.len() {
                    if let Token::Num(y) = tokens[idx] {
                        canvas.line(curr_x, curr_y, curr_x, y);
                        curr_y = y;
                        idx += 1;
                    }
                } else {
                    break;
                }
            }
            'v' => {
                if idx < tokens.len() {
                    if let Token::Num(dy) = tokens[idx] {
                        let ny = curr_y + dy;
                        canvas.line(curr_x, curr_y, curr_x, ny);
                        curr_y = ny;
                        idx += 1;
                    }
                } else {
                    break;
                }
            }
            'Z' | 'z' => {
                canvas.line(curr_x, curr_y, start_x, start_y);
                curr_x = start_x;
                curr_y = start_y;
            }
            'A' => {
                if idx + 6 < tokens.len() {
                    if let (
                        Token::Num(rx),
                        Token::Num(ry),
                        Token::Num(_),
                        Token::Num(large),
                        Token::Num(sweep),
                        Token::Num(x),
                        Token::Num(y),
                    ) = (
                        tokens[idx],
                        tokens[idx + 1],
                        tokens[idx + 2],
                        tokens[idx + 3],
                        tokens[idx + 4],
                        tokens[idx + 5],
                        tokens[idx + 6],
                    ) {
                        draw_svg_arc(
                            canvas,
                            (curr_x, curr_y),
                            (x, y),
                            rx,
                            ry,
                            large != 0.0,
                            sweep != 0.0,
                        );
                        curr_x = x;
                        curr_y = y;
                        idx += 7;
                    }
                } else {
                    break;
                }
            }
            'a' => {
                if idx + 6 < tokens.len() {
                    if let (
                        Token::Num(rx),
                        Token::Num(ry),
                        Token::Num(_),
                        Token::Num(large),
                        Token::Num(sweep),
                        Token::Num(dx),
                        Token::Num(dy),
                    ) = (
                        tokens[idx],
                        tokens[idx + 1],
                        tokens[idx + 2],
                        tokens[idx + 3],
                        tokens[idx + 4],
                        tokens[idx + 5],
                        tokens[idx + 6],
                    ) {
                        let nx = curr_x + dx;
                        let ny = curr_y + dy;
                        draw_svg_arc(
                            canvas,
                            (curr_x, curr_y),
                            (nx, ny),
                            rx,
                            ry,
                            large != 0.0,
                            sweep != 0.0,
                        );
                        curr_x = nx;
                        curr_y = ny;
                        idx += 7;
                    }
                } else {
                    break;
                }
            }
            _ => {
                idx += 1;
            }
        }
    }
}

fn draw_svg_arc(
    canvas: &SvgCanvas,
    (x1, y1): (f32, f32),
    (x2, y2): (f32, f32),
    rx: f32,
    ry: f32,
    large_arc: bool,
    sweep: bool,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let d = dx.hypot(dy);
    if d < 1e-4 {
        return;
    }
    let mut r = rx.abs().max(ry.abs());
    if r < d / 2.0 {
        r = d / 2.0;
    }
    let mx = (x1 + x2) / 2.0;
    let my = (y1 + y2) / 2.0;
    let h_sq = (r * r - (d / 2.0) * (d / 2.0)).max(0.0);
    let h = h_sq.sqrt();
    let nx = -dy / d;
    let ny = dx / d;
    let sign = if large_arc != sweep { 1.0 } else { -1.0 };
    let cx = mx + sign * h * nx;
    let cy = my + sign * h * ny;

    let a1 = (y1 - cy).atan2(x1 - cx);
    let a2 = (y2 - cy).atan2(x2 - cx);
    let mut delta = a2 - a1;
    if sweep && delta <= 0.0 {
        delta += std::f32::consts::TAU;
    } else if !sweep && delta >= 0.0 {
        delta -= std::f32::consts::TAU;
    }
    canvas.arc(
        cx - r,
        cy - r,
        2.0 * r,
        2.0 * r,
        a1.to_degrees(),
        delta.to_degrees(),
    );
}
