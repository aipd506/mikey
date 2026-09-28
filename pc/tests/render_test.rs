#[cfg(windows)]
#[test]
fn test_export_icons_preview() {
    use mikey::flyout::heroicons::*;
    use mikey::flyout::win32::Gdiplus;
    use std::ffi::c_void;

    let g = Gdiplus::init().expect("gdiplus init");

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(hdc: usize) -> usize;
        fn CreateDIBSection(
            hdc: usize,
            pbmi: *const BITMAPINFO,
            usage: u32,
            ppvBits: *mut *mut u8,
            hSection: usize,
            offset: u32,
        ) -> usize;
        fn SelectObject(hdc: usize, h: usize) -> usize;
        fn DeleteObject(h: usize) -> i32;
        fn DeleteDC(hdc: usize) -> i32;
    }

    #[allow(clippy::upper_case_acronyms)] // Win32's own names
    #[repr(C)]
    struct BITMAPINFOHEADER {
        bi_size: u32,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: u32,
        bi_size_image: u32,
        bi_xpels_per_meter: i32,
        bi_ypels_per_meter: i32,
        bi_clr_used: u32,
        bi_clr_important: u32,
    }

    #[allow(clippy::upper_case_acronyms)]
    #[repr(C)]
    struct BITMAPINFO {
        bmi_header: BITMAPINFOHEADER,
        bmi_colors: [u32; 1],
    }

    let w = 320;
    let h = 240;

    let bmi = BITMAPINFO {
        bmi_header: BITMAPINFOHEADER {
            bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            bi_width: w,
            bi_height: -h,
            bi_planes: 1,
            bi_bit_count: 32,
            bi_compression: 0,
            bi_size_image: (w * h * 4) as u32,
            bi_xpels_per_meter: 0,
            bi_ypels_per_meter: 0,
            bi_clr_used: 0,
            bi_clr_important: 0,
        },
        bmi_colors: [0],
    };

    let mut bits: *mut u8 = std::ptr::null_mut();
    let mem_dc = unsafe { CreateCompatibleDC(0) };
    let dib = unsafe { CreateDIBSection(mem_dc, &bmi, 0, &mut bits, 0, 0) };
    let old_bmp = unsafe { SelectObject(mem_dc, dib) };

    // Fill background with #111111 (0xFF111111)
    let slice = unsafe { std::slice::from_raw_parts_mut(bits as *mut u32, (w * h) as usize) };
    slice.fill(0xFF111111);

    let mut graphics: *mut c_void = std::ptr::null_mut();
    unsafe {
        (g.fn_create_from_hdc)(mem_dc, &mut graphics);
        (g.fn_set_smoothing_mode)(graphics, 4);
    }

    // Row 1: Mic, Mic-off, Camera, Flip
    draw_hero_mic(&g, graphics, 40.0, 40.0, 0xFF30D158, false);
    draw_hero_mic(&g, graphics, 120.0, 40.0, 0xFFFF453A, true);
    draw_hero_camera(&g, graphics, 200.0, 40.0, 0xFF0A84FF);
    draw_hero_flip(&g, graphics, 280.0, 40.0, 0xFFFFFFFF);

    // Row 2: Sound, Sliders, Preview, Alert
    draw_hero_sound(&g, graphics, 40.0, 120.0, 0xFF8E8E93);
    draw_hero_sliders(&g, graphics, 120.0, 120.0, 0xFF8E8E93);
    draw_hero_preview(&g, graphics, 200.0, 120.0, 0xFF0A84FF);
    draw_hero_alert(&g, graphics, 280.0, 120.0, 0xFFFFD60A);

    // Row 3: Folder, Disconnect, Power
    draw_hero_folder(&g, graphics, 40.0, 180.0, 0xFF8E8E93);
    draw_hero_disconnect(&g, graphics, 120.0, 180.0, 0xFFFF453A);
    draw_hero_power(&g, graphics, 200.0, 180.0, 0xFFFF453A);

    // Saved to the temp folder, to look at by eye.
    let artifact_path = std::env::temp_dir().join("mikey-icons-preview.bmp");

    // BMP Header (14 bytes) + DIB Header (40 bytes) + pixel data
    let mut bmp_data = Vec::new();
    let file_size = 14 + 40 + (w * h * 4);

    // BMP Header
    bmp_data.extend_from_slice(b"BM");
    bmp_data.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp_data.extend_from_slice(&[0, 0, 0, 0]); // reserved
    bmp_data.extend_from_slice(&(54u32).to_le_bytes()); // offset to pixels

    // DIB Header
    bmp_data.extend_from_slice(&(40u32).to_le_bytes());
    bmp_data.extend_from_slice(&w.to_le_bytes());
    bmp_data.extend_from_slice(&(-h).to_le_bytes()); // top-down
    bmp_data.extend_from_slice(&(1u16).to_le_bytes()); // planes
    bmp_data.extend_from_slice(&(32u16).to_le_bytes()); // bpp
    bmp_data.extend_from_slice(&(0u32).to_le_bytes()); // BI_RGB
    bmp_data.extend_from_slice(&((w * h * 4) as u32).to_le_bytes());
    bmp_data.extend_from_slice(&(0i32).to_le_bytes());
    bmp_data.extend_from_slice(&(0i32).to_le_bytes());
    bmp_data.extend_from_slice(&(0u32).to_le_bytes());
    bmp_data.extend_from_slice(&(0u32).to_le_bytes());

    // Raw pixel data
    let raw_bytes = unsafe { std::slice::from_raw_parts(bits, (w * h * 4) as usize) };
    bmp_data.extend_from_slice(raw_bytes);

    std::fs::write(artifact_path, bmp_data).expect("write bmp");

    unsafe {
        (g.fn_delete_graphics)(graphics);
        SelectObject(mem_dc, old_bmp);
        DeleteObject(dib);
        DeleteDC(mem_dc);
    }
}
