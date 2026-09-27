#[cfg(not(windows))]
use std::fs::File;
#[cfg(not(windows))]
use std::io::Read;
use std::time::SystemTime;

pub fn generate_random_hex(byte_count: usize) -> String {
    let mut bytes = vec![0u8; byte_count];
    fill_random_bytes(&mut bytes);
    let mut s = String::with_capacity(byte_count * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[cfg(windows)]
fn fill_random_bytes(buf: &mut [u8]) {
    #[link(name = "advapi32")]
    extern "system" {
        fn SystemFunction036(pbBuffer: *mut u8, dwLen: u32) -> u8;
    }
    let ok = unsafe { SystemFunction036(buf.as_mut_ptr(), buf.len() as u32) != 0 };
    if !ok {
        fill_fallback_pseudo_random(buf);
    }
}

#[cfg(not(windows))]
fn fill_random_bytes(buf: &mut [u8]) {
    if let Ok(mut f) = File::open("/dev/urandom") {
        if f.read_exact(buf).is_ok() {
            return;
        }
    }
    fill_fallback_pseudo_random(buf);
}

fn fill_fallback_pseudo_random(buf: &mut [u8]) {
    let mut seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0x1234_5678_9ABC_DEF0);

    for chunk in buf.chunks_mut(8) {
        seed ^= seed >> 12;
        seed ^= seed << 25;
        seed ^= seed >> 27;
        let val = seed.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let bytes = (val as u64).to_le_bytes();
        let copy_len = chunk.len().min(8);
        chunk[..copy_len].copy_from_slice(&bytes[..copy_len]);
    }
}
