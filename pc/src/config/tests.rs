use super::*;

#[test]
fn test_random_hex_generation() {
    let hex16 = generate_random_hex(16);
    let hex32 = generate_random_hex(32);
    assert_eq!(hex16.len(), 32);
    assert_eq!(hex32.len(), 64);
    assert_ne!(hex16, generate_random_hex(16));
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut cfg = Config::default();
    cfg.add_or_update_device(
        "dev-123".into(),
        "Pixel 7".into(),
        "tok-abc".into(),
        Some(1),
    );

    let toml_str = toml::to_string_pretty(&cfg).unwrap();
    let loaded: Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(cfg.pc_id, loaded.pc_id);
    assert_eq!(loaded.trusted_devices.len(), 1);
    assert!(loaded.is_trusted("dev-123", "tok-abc"));
    assert!(!loaded.is_trusted("dev-123", "wrong-tok"));
}

#[test]
fn test_forget_device() {
    let mut cfg = Config::default();
    cfg.add_or_update_device(
        "dev-123".into(),
        "Pixel 7".into(),
        "tok-abc".into(),
        Some(1),
    );
    assert!(cfg.forget_device("dev-123"));
    assert!(!cfg.is_trusted("dev-123", "tok-abc"));
}
