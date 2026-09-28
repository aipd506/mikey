use zune_jpeg::zune_core::bytestream::ZCursor;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

/// A decoded frame in BGR order, 3 bytes per pixel: what softcam and GDI take as is.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub width: usize,
    pub height: usize,
    pub bgr: Vec<u8>,
}

impl DecodedFrame {
    pub fn new(width: usize, height: usize, bgr: Vec<u8>) -> Self {
        Self { width, height, bgr }
    }

    /// Fills `out` with 0x00RRGGBB pixels for the preview window, reusing its memory.
    pub fn fill_rgb32(&self, out: &mut Vec<u32>) {
        out.clear();
        let (chunks, _) = self.bgr.as_chunks::<3>();
        out.extend(
            chunks
                .iter()
                .map(|p| (p[2] as u32) << 16 | (p[1] as u32) << 8 | p[0] as u32),
        );
    }

    /// Letterboxes or pillarboxes this frame into a target dimension (dst_w, dst_h)
    /// preserving original aspect ratio with black bars.
    pub fn letterbox(&self, dst_w: usize, dst_h: usize) -> DecodedFrame {
        if self.width == dst_w && self.height == dst_h {
            return self.clone();
        }

        let mut out = vec![0u8; dst_w * dst_h * 3]; // black background

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

                out[dst_idx..dst_idx + 3].copy_from_slice(&self.bgr[src_idx..src_idx + 3]);
            }
        }

        DecodedFrame {
            width: dst_w,
            height: dst_h,
            bgr: out,
        }
    }

    /// Generates a neutral placeholder frame for when the camera is off:
    /// Black background with a subtle, clean Mikey mark in the center (#3A3A3C / #8E8E93).
    pub fn placeholder(width: usize, height: usize) -> Self {
        let mut bgr = vec![0x11u8; width * height * 3]; // dark surface #111111

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
                    bgr[idx] = 0x93; // B
                    bgr[idx + 1] = 0x8E; // G
                    bgr[idx + 2] = 0x8E; // R
                }
            }
        }

        DecodedFrame { width, height, bgr }
    }
}

/// Decodes a JPEG payload straight to BGR, into `buf` when it's big enough, so a steady stream
/// of frames allocates nothing.
pub fn decode_jpeg(jpeg_bytes: &[u8], mut buf: Vec<u8>) -> Result<DecodedFrame, String> {
    if jpeg_bytes.is_empty() {
        return Err("empty JPEG payload".to_string());
    }

    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::BGR);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(jpeg_bytes), options);
    decoder
        .decode_headers()
        .map_err(|e| format!("JPEG decode error: {:?}", e))?;
    let (width, height) = decoder
        .dimensions()
        .ok_or_else(|| "missing JPEG dimensions".to_string())?;
    // softcam and GDI read exactly width * height * 3 bytes; a grayscale JPEG would come out short.
    let size = width * height * 3;
    if decoder.output_buffer_size() != Some(size) {
        return Err("JPEG is not a color image".to_string());
    }
    buf.resize(size, 0);
    decoder
        .decode_into(&mut buf)
        .map_err(|e| format!("JPEG decode error: {:?}", e))?;

    Ok(DecodedFrame {
        width,
        height,
        bgr: buf,
    })
}
