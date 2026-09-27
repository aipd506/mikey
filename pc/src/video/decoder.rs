use std::io::Cursor;
use zune_jpeg::JpegDecoder;

/// Represents a decoded raw RGB frame.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>, // 24-bit RGB (3 bytes per pixel)
}

impl DecodedFrame {
    pub fn new(width: usize, height: usize, rgb: Vec<u8>) -> Self {
        Self { width, height, rgb }
    }

    /// Converts RGB to BGR (required by DirectShow / Softcam).
    pub fn to_bgr(&self) -> Vec<u8> {
        let mut bgr = Vec::with_capacity(self.rgb.len());
        let (chunks, _) = self.rgb.as_chunks::<3>();
        for chunk in chunks {
            bgr.push(chunk[2]); // B
            bgr.push(chunk[1]); // G
            bgr.push(chunk[0]); // R
        }
        bgr
    }

    /// Converts RGB to 32-bit 0x00RRGGBB pixels (required by minifb window).
    pub fn to_rgb32(&self) -> Vec<u32> {
        let mut rgb32 = Vec::with_capacity(self.width * self.height);
        let (chunks, _) = self.rgb.as_chunks::<3>();
        for chunk in chunks {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            rgb32.push((r << 16) | (g << 8) | b);
        }
        rgb32
    }

    /// Letterboxes or pillarboxes this frame into a target dimension (dst_w, dst_h)
    /// preserving original aspect ratio with black bars.
    pub fn letterbox(&self, dst_w: usize, dst_h: usize) -> DecodedFrame {
        if self.width == dst_w && self.height == dst_h {
            return self.clone();
        }

        let mut out_rgb = vec![0u8; dst_w * dst_h * 3]; // black background

        let scale_w = dst_w as f32 / self.width as f32;
        let scale_h = dst_h as f32 / self.height as f32;
        let scale = scale_w.min(scale_h);

        let scaled_w = ((self.width as f32 * scale).round() as usize)
            .max(1)
            .min(dst_w);
        let scaled_h = ((self.height as f32 * scale).round() as usize)
            .max(1)
            .min(dst_h);

        let offset_x = (dst_w - scaled_w) / 2;
        let offset_y = (dst_h - scaled_h) / 2;

        // Bilinear or nearest-neighbor blit into target
        for dy in 0..scaled_h {
            let sy = (dy as f32 / scale) as usize;
            let sy = sy.min(self.height - 1);
            let target_y = offset_y + dy;

            for dx in 0..scaled_w {
                let sx = (dx as f32 / scale) as usize;
                let sx = sx.min(self.width - 1);
                let target_x = offset_x + dx;

                let src_idx = (sy * self.width + sx) * 3;
                let dst_idx = (target_y * dst_w + target_x) * 3;

                out_rgb[dst_idx..dst_idx + 3].copy_from_slice(&self.rgb[src_idx..src_idx + 3]);
            }
        }

        DecodedFrame {
            width: dst_w,
            height: dst_h,
            rgb: out_rgb,
        }
    }

    /// Generates a neutral placeholder frame for when the camera is off:
    /// Black background with a subtle, clean Mikey mark in the center (#3A3A3C / #8E8E93).
    pub fn placeholder(width: usize, height: usize) -> Self {
        let mut rgb = vec![0x11u8; width * height * 3]; // dark surface #111111

        let center_x = width / 2;
        let center_y = height / 2;
        let mark_size = (width.min(height) / 8).max(12);

        // Draw a minimalist camera glyph outline in #8E8E93 (142, 142, 147)
        let x_start = center_x.saturating_sub(mark_size);
        let x_end = (center_x + mark_size).min(width);
        let y_start = center_y.saturating_sub(mark_size * 2 / 3);
        let y_end = (center_y + mark_size * 2 / 3).min(height);

        for y in y_start..y_end {
            for x in x_start..x_end {
                let border = x == x_start || x == x_end - 1 || y == y_start || y == y_end - 1;
                let dx = x as isize - center_x as isize;
                let dy = y as isize - center_y as isize;
                let dist_sq = dx * dx + dy * dy;
                let inner_radius = (mark_size / 3) as isize;
                let lens = (dist_sq - inner_radius * inner_radius).abs() <= mark_size as isize;

                if border || lens {
                    let idx = (y * width + x) * 3;
                    rgb[idx] = 0x8E; // R
                    rgb[idx + 1] = 0x8E; // G
                    rgb[idx + 2] = 0x93; // B
                }
            }
        }

        DecodedFrame { width, height, rgb }
    }
}

/// Decodes a JPEG payload into a DecodedFrame.
pub fn decode_jpeg(jpeg_bytes: &[u8]) -> Result<DecodedFrame, String> {
    if jpeg_bytes.is_empty() {
        return Err("empty JPEG payload".to_string());
    }

    let mut decoder = JpegDecoder::new(Cursor::new(jpeg_bytes));
    let rgb = decoder
        .decode()
        .map_err(|e| format!("JPEG decode error: {:?}", e))?;
    let (width, height) = decoder
        .dimensions()
        .ok_or_else(|| "missing JPEG dimensions".to_string())?;

    Ok(DecodedFrame { width, height, rgb })
}
