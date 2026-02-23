use http_auth_verifier::{Claims, authorize_token, constant_time_eq, validate_jwt_token};
use jsonwebtoken::{EncodingKey, Header, encode};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_jwt(secret: &str, subject: &str, issuer: &str, expires_in_seconds: u64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let claims = Claims {
        sub: subject.to_string(),
        exp: now + expires_in_seconds,
        iat: now,
        iss: issuer.to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("JWT should encode")
}

#[test]
fn integration_authorizes_exact_basic_token_with_dots() {
    assert!(authorize_token(
        Some("alpha.beta.gamma"),
        Some("alpha.beta.gamma"),
        Some("unused-secret")
    ));
}

#[test]
fn integration_rejects_mismatched_basic_token() {
    assert!(!authorize_token(Some("other"), Some("expected"), None));
}

#[test]
fn integration_authorizes_valid_jwt() {
    let secret = "integration-secret";
    let token = create_jwt(secret, "user1", "integration-tests", 3600);

    assert!(validate_jwt_token(token.as_str(), secret));
    assert!(authorize_token(Some(token.as_str()), None, Some(secret)));
}

#[test]
fn integration_rejects_invalid_jwt_signature() {
    let token = create_jwt("signing-secret", "user1", "integration-tests", 3600);

    assert!(!validate_jwt_token(token.as_str(), "different-secret"));
    assert!(!authorize_token(Some(token.as_str()), None, Some("different-secret")));
}

#[test]
fn integration_constant_time_eq_aligns_with_equality() {
    assert!(constant_time_eq("same-value", "same-value"));
    assert!(!constant_time_eq("same-value", "different-value"));
}
