mod device;
mod stream;

pub use device::{
    check_virtual_device_status, find_output_device, is_mikey_branded, is_virtual_device,
    SAMPLE_RATE,
};
pub use stream::{start_audio_stream, start_loopback_stream};
