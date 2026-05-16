use super::*;
use crate::test_support::{
    create_jwt_token, encode_claims, encode_json_claims, unix_timestamp_seconds,
};

#[test]
fn basic_mode_authorization_behaves_as_expected() {
    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Basic,
        token: Some("secret".into()),
        jwt_secret: None,
    };

    assert!(config.requires_auth());
    assert!(config.can_enforce_auth());
    assert!(config.is_authorized(Some("secret")));
    assert!(!config.is_authorized(Some("other")));
    assert!(!config.is_authorized(None));
}

#[test]
fn basic_mode_accepts_literal_token_with_jwt_like_shape() {
    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Basic,
        token: Some("dot.token.value".into()),
        jwt_secret: Some("unused-secret".into()),
    };

    assert!(config.is_authorized(Some("dot.token.value")));
}

#[test]
fn open_mode_is_always_authorized() {
    let config = PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };

    assert!(!config.requires_auth());
    assert!(!config.can_enforce_auth());
    assert!(config.is_authorized(None));
    assert!(config.is_authorized(Some("anything")));
}

#[test]
fn jwt_mode_accepts_valid_token() {
    let secret = "test-secret";
    let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    assert!(config.requires_auth());
    assert!(config.can_enforce_auth());
    assert!(config.is_authorized(Some(&token)));
    assert!(!config.is_authorized(Some("invalid-token")));
}

#[test]
fn jwt_mode_rejects_expired_token() {
    let secret = "test-secret";
    let now = unix_timestamp_seconds();
    let claims = Claims {
        sub: "user123".to_string(),
        exp: now - 3600,
        iat: now - 7200,
        iss: "rust-template".to_string(),
    };
    let token = encode_claims(secret, &claims).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn jwt_mode_rejects_invalid_signature() {
    let token = create_jwt_token("correct-secret", "user123", "rust-template", 3600).unwrap();
    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some("wrong-secret".to_string()),
    };

    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn jwt_mode_rejects_missing_claims() {
    let secret = "test-secret";
    let now = unix_timestamp_seconds();

    let token_missing_issuer = encode_json_claims(
        secret,
        &serde_json::json!({
            "sub": "user123",
            "exp": now + 3600,
            "iat": now,
            "iss": ""
        }),
    )
    .unwrap();

    let token_missing_subject = encode_json_claims(
        secret,
        &serde_json::json!({
            "sub": "",
            "exp": now + 3600,
            "iat": now,
            "iss": "rust-template"
        }),
    )
    .unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    assert!(!config.is_authorized(Some(&token_missing_issuer)));
    assert!(!config.is_authorized(Some(&token_missing_subject)));
}

#[test]
fn jwt_mode_rejects_iat_too_far_in_future() {
    let secret = "test-secret";
    let now = unix_timestamp_seconds();
    let claims = Claims {
        sub: "user123".to_string(),
        exp: now + 3600,
        iat: now + 301,
        iss: "rust-template".to_string(),
    };
    let token = encode_claims(secret, &claims).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn parse_strict_validates_auth_modes() {
    assert_eq!(PlatformAuthMode::parse_strict("basic").unwrap(), PlatformAuthMode::Basic);
    assert_eq!(PlatformAuthMode::parse_strict("jwt").unwrap(), PlatformAuthMode::Jwt);
    assert_eq!(PlatformAuthMode::parse_strict("none").unwrap(), PlatformAuthMode::Open);
    assert_eq!(PlatformAuthMode::parse_strict("open").unwrap(), PlatformAuthMode::Open);

    assert_eq!(PlatformAuthMode::parse_strict("BASIC").unwrap(), PlatformAuthMode::Basic);
    assert_eq!(PlatformAuthMode::parse_strict("JWT").unwrap(), PlatformAuthMode::Jwt);

    assert!(PlatformAuthMode::parse_strict("invalid").is_err());
    assert!(PlatformAuthMode::parse_strict("bearer").is_err());
    assert!(PlatformAuthMode::parse_strict("").is_err());
}

#[test]
fn from_str_falls_back_to_open_for_invalid() {
    assert_eq!(PlatformAuthMode::from("basic"), PlatformAuthMode::Basic);
    assert_eq!(PlatformAuthMode::from("jwt"), PlatformAuthMode::Jwt);
    assert_eq!(PlatformAuthMode::from("open"), PlatformAuthMode::Open);
    assert_eq!(PlatformAuthMode::from("none"), PlatformAuthMode::Open);
    assert_eq!(PlatformAuthMode::from("invalid"), PlatformAuthMode::Open);
}

#[test]
fn mode_label_returns_expected_values() {
    assert_eq!(
        PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None }
            .mode_label(),
        "open"
    );
    assert_eq!(
        PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None, jwt_secret: None }
            .mode_label(),
        "basic"
    );
    assert_eq!(
        PlatformAuthConfig { mode: PlatformAuthMode::Jwt, token: None, jwt_secret: None }
            .mode_label(),
        "jwt"
    );
}

#[test]
fn token_present_respects_mode() {
    let basic_with_token = PlatformAuthConfig {
        mode: PlatformAuthMode::Basic,
        token: Some("secret".into()),
        jwt_secret: None,
    };
    assert!(basic_with_token.token_present());

    let basic_with_only_secret = PlatformAuthConfig {
        mode: PlatformAuthMode::Basic,
        token: None,
        jwt_secret: Some("jwt-secret".into()),
    };
    assert!(!basic_with_only_secret.token_present());

    let jwt_with_secret = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some("jwt-secret".into()),
    };
    assert!(jwt_with_secret.token_present());

    let jwt_with_only_token = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: Some("basic-token".into()),
        jwt_secret: None,
    };
    assert!(!jwt_with_only_token.token_present());

    let open = PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };
    assert!(open.token_present());
}

#[test]
fn warn_if_misconfigured_only_when_required() {
    let missing_basic =
        PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None, jwt_secret: None };
    assert!(missing_basic.warn_if_misconfigured());

    let missing_jwt =
        PlatformAuthConfig { mode: PlatformAuthMode::Jwt, token: None, jwt_secret: None };
    assert!(missing_jwt.warn_if_misconfigured());

    let open = PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };
    assert!(!open.warn_if_misconfigured());
}

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_basic_mode_requires_exact_token(
            token in "[A-Za-z0-9_-]{1,32}",
            candidate in "[A-Za-z0-9_-]{1,32}",
        ) {
            let cfg = PlatformAuthConfig {
                mode: PlatformAuthMode::Basic,
                token: Some(token.clone()),
                jwt_secret: None,
            };

            prop_assert!(cfg.is_authorized(Some(&token)));
            prop_assert_eq!(cfg.is_authorized(Some(&candidate)), token == candidate);
        }

        #[test]
        fn prop_mode_parser_case_insensitive(
            raw in prop_oneof![
                Just("basic".to_string()),
                Just("jwt".to_string()),
                Just("open".to_string()),
                Just("none".to_string())
            ],
            upper in any::<bool>(),
        ) {
            let input = if upper { raw.to_ascii_uppercase() } else { raw.clone() };
            let parsed = PlatformAuthMode::parse_strict(&input).unwrap();

            let expected = match raw.as_str() {
                "basic" => PlatformAuthMode::Basic,
                "jwt" => PlatformAuthMode::Jwt,
                "open" | "none" => PlatformAuthMode::Open,
                _ => unreachable!(),
            };

            prop_assert_eq!(parsed, expected);
        }

        #[test]
        fn prop_open_mode_always_authorized(candidate in ".*") {
            let cfg = PlatformAuthConfig {
                mode: PlatformAuthMode::Open,
                token: None,
                jwt_secret: None,
            };

            prop_assert!(cfg.is_authorized(None));
            prop_assert!(cfg.is_authorized(Some(&candidate)));
        }
    }
}
