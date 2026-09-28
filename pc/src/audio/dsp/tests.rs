use super::*;

#[test]
fn test_noise_gate_attenuation_on_silence() {
    let mut dsp = AudioDsp::new();
    let mut silence = vec![0i16; 480 * 80];
    dsp.process(&mut silence, 0, false);

    assert_eq!(silence[0], 0);
    assert!(dsp.gate_gain < 0.15);
}

#[test]
fn test_audio_dsp_rnnoise_suppression() {
    let mut dsp = AudioDsp::new();
    let mut samples = vec![1000i16; 480 * 4];

    dsp.process(&mut samples, 100, false);

    assert_eq!(samples.len(), 480 * 4);
    assert!(samples.iter().any(|&s| s != 0));
}

#[test]
fn test_aec_loopback_cancellation() {
    let mut echo = EchoSuppressor::new();

    let mut ref_signal = Vec::with_capacity(14400);
    for i in 0..14400 {
        let val = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin();
        ref_signal.push(val * 8000.0);
    }
    echo.push_reference(&ref_signal);

    let delay = 960;
    let mut mic_frame = [0.0f32; 480];
    let start_idx = ref_signal.len() - delay - 480;
    for (i, item) in mic_frame.iter_mut().enumerate() {
        *item = ref_signal[start_idx + i] * 0.8;
    }

    let raw_energy: f32 = mic_frame.iter().map(|&s| s * s).sum();
    echo.process_chunk(&mut mic_frame);
    let cancelled_energy: f32 = mic_frame.iter().map(|&s| s * s).sum();

    assert!(
        cancelled_energy < raw_energy * 0.4,
        "AEC should cancel correlated speaker sound: raw={}, cancelled={}",
        raw_energy,
        cancelled_energy
    );
}

#[test]
fn test_aec_loopback_normalized_cancellation() {
    let mut echo = EchoSuppressor::new();

    // Reference from CPAL loopback is float in [-1.0, 1.0]
    let mut ref_signal = Vec::with_capacity(14400);
    for i in 0..14400 {
        let val = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin();
        ref_signal.push(val * 0.25);
    }
    echo.push_reference(&ref_signal);

    // Mic frame recorded by phone in PCM scale [-32768.0, 32767.0]
    let delay = 960;
    let mut mic_frame = [0.0f32; 480];
    let start_idx = ref_signal.len() - delay - 480;
    for (i, item) in mic_frame.iter_mut().enumerate() {
        *item = ref_signal[start_idx + i] * 32767.0 * 0.7;
    }

    let raw_energy: f32 = mic_frame.iter().map(|&s| s * s).sum();
    echo.process_chunk(&mut mic_frame);
    let cancelled_energy: f32 = mic_frame.iter().map(|&s| s * s).sum();

    assert!(
        cancelled_energy < raw_energy * 0.3,
        "AEC must cancel normalized speaker loopback bleed: raw={}, cancelled={}",
        raw_energy,
        cancelled_energy
    );
}
