use mikey::protocol::{
    read_frame, write_frame, Frame, FrameType, MediaHeader, CODEC_JPEG, MEDIA_HEADER_LEN,
};
use mikey::video::{DecodedFrame, VideoPipeline};
use std::io::Cursor;

#[test]
fn test_decoded_frame_conversions() {
    let width = 2;
    let height = 2;
    let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
    let frame = DecodedFrame::new(width, height, rgb);

    // Test BGR conversion for Softcam
    let bgr = frame.to_bgr();
    assert_eq!(bgr.len(), 12);
    assert_eq!(&bgr[0..3], &[0, 0, 255]);
    assert_eq!(&bgr[3..6], &[0, 255, 0]);
    assert_eq!(&bgr[6..9], &[255, 0, 0]);
    assert_eq!(&bgr[9..12], &[255, 255, 255]);

    // Test 32-bit RGB conversion for minifb (0x00RRGGBB)
    let rgb32 = frame.to_rgb32();
    assert_eq!(rgb32.len(), 4);
    assert_eq!(rgb32[0], 0x00FF0000);
    assert_eq!(rgb32[1], 0x0000FF00);
    assert_eq!(rgb32[2], 0x000000FF);
    assert_eq!(rgb32[3], 0x00FFFFFF);
}

#[test]
fn test_letterboxing() {
    let src_w = 4;
    let src_h = 3;
    let rgb = vec![0xFF; src_w * src_h * 3];
    let frame = DecodedFrame::new(src_w, src_h, rgb);

    let letterboxed = frame.letterbox(16, 9);
    assert_eq!(letterboxed.width, 16);
    assert_eq!(letterboxed.height, 9);
    assert_eq!(letterboxed.rgb.len(), 16 * 9 * 3);
}

#[test]
fn test_placeholder_generation() {
    let placeholder = DecodedFrame::placeholder(1280, 720);
    assert_eq!(placeholder.width, 1280);
    assert_eq!(placeholder.height, 720);
    assert_eq!(placeholder.rgb.len(), 1280 * 720 * 3);
    assert_eq!(placeholder.rgb[0], 0x11);
    assert_eq!(placeholder.rgb[1], 0x11);
    assert_eq!(placeholder.rgb[2], 0x11);
}

#[test]
fn test_video_wire_frame() {
    let header = MediaHeader {
        seq: 101,
        capture_ts: 1_700_000_000_000,
        codec: CODEC_JPEG,
        reserved: 0,
    };
    let mut payload = header.to_vec();
    let fake_jpeg = b"fake-jpeg-payload-data";
    payload.extend_from_slice(fake_jpeg);

    let frame = Frame::new(FrameType::Video, payload);
    let mut buf = Vec::new();
    write_frame(&mut buf, &frame).unwrap();

    let mut cursor = Cursor::new(buf);
    let parsed = read_frame(&mut cursor).unwrap();
    assert_eq!(parsed.frame_type, FrameType::Video);

    let parsed_header = MediaHeader::parse(&parsed.payload[0..MEDIA_HEADER_LEN]).unwrap();
    assert_eq!(parsed_header.seq, 101);
    assert_eq!(parsed_header.codec, CODEC_JPEG);
    assert_eq!(&parsed.payload[MEDIA_HEADER_LEN..], fake_jpeg);
}

#[test]
fn test_video_pipeline_state() {
    let pipeline = VideoPipeline::new();
    assert!(!pipeline.is_camera_on());
    assert!(!pipeline.is_preview_visible());

    pipeline.set_preview_visible(true);
    assert!(pipeline.is_preview_visible());

    pipeline.toggle_preview();
    assert!(!pipeline.is_preview_visible());

    pipeline.push_jpeg_frame(vec![1, 2, 3], 1, 1000);
    assert!(pipeline.is_camera_on());

    pipeline.set_camera_active(false);
    assert!(!pipeline.is_camera_on());
}
