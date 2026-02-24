use http_auth_mode::PlatformAuthMode;

#[test]
fn strict_parser_supports_none_alias() {
    assert_eq!(PlatformAuthMode::parse_strict("none").unwrap(), PlatformAuthMode::Open);
}

#[test]
fn strict_parser_rejects_unknown_mode() {
    let error = PlatformAuthMode::parse_strict("definitely-not-valid")
        .expect_err("invalid mode must return an error");
    assert!(error.contains("Invalid auth mode 'definitely-not-valid'"));
}

#[test]
fn tolerant_parser_falls_back_to_open() {
    assert_eq!(PlatformAuthMode::from("definitely-not-valid"), PlatformAuthMode::Open);
}

#[test]
fn labels_match_contract() {
    assert_eq!(PlatformAuthMode::Open.label(), "open");
    assert_eq!(PlatformAuthMode::Basic.label(), "basic");
    assert_eq!(PlatformAuthMode::Jwt.label(), "jwt");
}
