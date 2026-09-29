use super::*;

#[test]
fn test_jitter_buffer_prebuffering_and_levels() {
    let jb = JitterBuffer::new();
    assert_eq!(jb.target_samples(), USB_TARGET_MS * SAMPLES_PER_MS);

    jb.set_level(3);
    assert_eq!(jb.target_samples(), WIFI_TARGET_MS * SAMPLES_PER_MS);

    jb.set_level(4);
    assert_eq!(jb.target_samples(), BT_TARGET_MS * SAMPLES_PER_MS);

    jb.set_level(1);
    assert_eq!(jb.target_samples(), USB_TARGET_MS * SAMPLES_PER_MS);

    let mut out = [1.0f32; 480];
    let mut tone_10ms = [0i16; 480];
    for (i, s) in tone_10ms.iter_mut().enumerate() {
        let val = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin();
        *s = (val * 4000.0) as i16;
    }

    jb.push_samples(&tone_10ms); // 10 ms
    jb.pop_samples(&mut out, 1);
    assert!(out.iter().all(|&v| v == 0.0)); // prebuffering not done

    jb.push_samples(&tone_10ms); // reaches 20 ms
    jb.pop_samples(&mut out, 1);
    // Output must be non-zero after prebuffer
    assert!(out.iter().any(|&v| v.abs() > 0.0));
    assert!(jb.get_auto_gain() >= MIN_AUTO_GAIN);
}

#[test]
fn test_auto_normalization_unity_gain() {
    let jb = JitterBuffer::new();
    assert!((jb.get_auto_gain() - 1.0).abs() < 0.01);

    // Push quiet audio tone (amplitude 800, well below target 4126)
    let mut quiet_tone = [0i16; 480];
    for (i, s) in quiet_tone.iter_mut().enumerate() {
        let val = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin();
        *s = (val * 800.0) as i16;
    }

    for _ in 0..50 {
        jb.push_samples(&quiet_tone);
    }
    let gain = jb.get_auto_gain();
    assert_eq!(gain, 1.0, "gain must stay at unity to prevent ducking");
}

#[test]
fn test_ns_controls() {
    let jb = JitterBuffer::new();
    assert_eq!(jb.get_ns_strength(), 100);

    jb.set_ns_strength(90);
    assert_eq!(jb.get_ns_strength(), 90);

    jb.set_ns_strength(150); // clamped
    assert_eq!(jb.get_ns_strength(), 100);
}

#[test]
fn test_jitter_buffer_max_cap() {
    let jb = JitterBuffer::new();
    let huge_samples = vec![500i16; MAX_SAMPLES + 1000];
    jb.push_samples(&huge_samples);

    assert_eq!(jb.len(), MAX_SAMPLES);
}

#[test]
fn test_drift_resampling_operation() {
    let jb = JitterBuffer::new();
    jb.set_level(1);
    let excess_samples = vec![1000i16; USB_TARGET_MS * SAMPLES_PER_MS + 2000];
    jb.push_samples(&excess_samples);

    let mut out = [0.0f32; 480];
    jb.pop_samples(&mut out, 1);
    assert!(jb.len() < excess_samples.len());
}

#[test]
fn test_output_resampled_to_device_rate() {
    // A 44.1 kHz device must still take the phone's 48 kHz at full speed, or the voice plays
    // slow and low and the buffer overflows. 10 ms there is 441 frames and 480 phone samples.
    let jb = JitterBuffer::new();
    jb.set_output_rate(44_100);
    jb.push_samples(&[1000i16; 4800]);
    let before = jb.len();
    let mut out = [0f32; 441];
    jb.pop_samples(&mut out, 1);
    let used = before - jb.len();
    assert!((478..=483).contains(&used), "used {} samples", used);
}
