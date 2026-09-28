use mikey::protocol::{
    read_frame, write_frame, Frame, FrameType, MediaHeader, CODEC_JPEG, MEDIA_HEADER_LEN,
};
use mikey::video::{decode_jpeg, DecodedFrame, VideoPipeline};
use std::io::Cursor;

#[test]
fn test_decoded_frame_conversions() {
    let width = 2;
    let height = 2;
    let bgr = vec![0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255];
    let frame = DecodedFrame::new(width, height, bgr);

    // Test 32-bit RGB conversion for minifb (0x00RRGGBB)
    let mut rgb32 = Vec::new();
    frame.fill_rgb32(&mut rgb32);
    assert_eq!(rgb32, [0x00FF0000, 0x0000FF00, 0x000000FF, 0x00FFFFFF]);

    // The next frame reuses the same memory
    let ptr = rgb32.as_ptr();
    frame.fill_rgb32(&mut rgb32);
    assert_eq!(rgb32.as_ptr(), ptr);
    assert_eq!(rgb32.len(), 4);
}

#[test]
fn test_decode_jpeg_to_bgr() {
    // Left half red, right half blue: softcam and GDI want blue first in each pixel.
    let jpeg = include_bytes!("data/red-blue.jpg");
    let frame = decode_jpeg(jpeg, Vec::new()).unwrap();
    assert_eq!((frame.width, frame.height), (16, 8));
    assert_eq!(frame.bgr.len(), 16 * 8 * 3);
    let red = &frame.bgr[0..3];
    let blue = &frame.bgr[15 * 3..16 * 3];
    assert!(
        red[0] < 40 && red[2] > 215,
        "red pixel came out as {:?}",
        red
    );
    assert!(
        blue[0] > 215 && blue[2] < 40,
        "blue pixel came out as {:?}",
        blue
    );

    // A buffer that is big enough is decoded into, not replaced
    let spare = Vec::with_capacity(1024);
    let ptr = spare.as_ptr();
    let frame = decode_jpeg(jpeg, spare).unwrap();
    assert_eq!(frame.bgr.as_ptr(), ptr);
}

#[test]
fn test_decode_jpeg_rejects_bad_input() {
    assert!(decode_jpeg(&[], Vec::new()).is_err());
    assert!(decode_jpeg(b"not a jpeg", Vec::new()).is_err());
    // softcam reads width * height * 3 bytes, so a one-channel image must not get through
    assert!(decode_jpeg(include_bytes!("data/gray.jpg"), Vec::new()).is_err());
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
    assert_eq!(letterboxed.bgr.len(), 16 * 9 * 3);
}

#[test]
fn test_placeholder_generation() {
    let placeholder = DecodedFrame::placeholder(1280, 720);
    assert_eq!(placeholder.width, 1280);
    assert_eq!(placeholder.height, 720);
    assert_eq!(placeholder.bgr.len(), 1280 * 720 * 3);
    assert_eq!(placeholder.bgr[0], 0x11);
    assert_eq!(placeholder.bgr[1], 0x11);
    assert_eq!(placeholder.bgr[2], 0x11);
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
