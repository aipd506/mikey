use super::*;

#[test]
fn test_audio_dsp_rnnoise_suppression() {
    let mut dsp = AudioDsp::new();
    let mut samples = vec![1000i16; 480 * 4];

    dsp.process(&mut samples, 100);

    assert_eq!(samples.len(), 480 * 4);
    assert!(samples.iter().any(|&s| s != 0));
}

#[test]
fn test_audio_dsp_zero_strength_passthrough() {
    let mut dsp = AudioDsp::new();
    let mut samples = vec![1234i16; 480 * 2];
    let copy = samples.clone();

    dsp.process(&mut samples, 0);

    assert_eq!(samples, copy);
}
