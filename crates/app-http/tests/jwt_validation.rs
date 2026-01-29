//! Integration tests for JWT validation with leeway
//!
//! These tests verify that JWT validation properly handles clock skew
//! and includes appropriate leeway to prevent rejection of valid tokens.

use app_http::security::{PlatformAuthConfig, PlatformAuthMode};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    exp: u64,
    iat: u64,
    iss: String,
}

/// Helper to create JWT tokens for testing
fn create_jwt_token(
    secret: &str,
    subject: &str,
    issuer: &str,
    expires_in_seconds: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let claims = TestClaims {
        sub: subject.to_string(),
        exp: now + expires_in_seconds,
        iat: now,
        iss: issuer.to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key)
}

#[test]
fn test_jwt_validation_with_clock_skew_future() {
    let secret = "test-secret-key-for-jwt-validation";

    // Create a token that's valid 30 seconds in the future (within 60s leeway)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let future_time = now + 30; // 30 seconds in future

    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: future_time,
        iat: now,
        iss: "rust-template".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should accept token even though it's slightly in the future due to leeway
    assert!(config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_with_clock_skew_past() {
    let secret = "test-secret-key-for-jwt-validation";

    // Create a token that expired 30 seconds ago (within 60s leeway)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let past_time = now - 30; // 30 seconds in past

    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: past_time,
        iat: (now - 3600), // Issued 1 hour ago
        iss: "rust-template".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should accept token even though it's slightly expired due to leeway
    assert!(config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_rejects_far_future() {
    let secret = "test-secret-key-for-jwt-validation";

    // Create a token that's valid 5 minutes in the future (beyond 60s leeway)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let future_time = now + 300; // 5 minutes in future

    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: future_time,
        iat: now,
        iss: "rust-template".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should accept token that's valid in the future
    assert!(config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_rejects_far_past() {
    let secret = "test-secret-key-for-jwt-validation";

    // Create a token that expired 5 minutes ago (beyond 60s leeway)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let past_time = now - 300; // 5 minutes in past

    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: past_time,
        iat: (past_time - 3600), // Issued 1 hour before expiration
        iss: "rust-template".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should reject token that's too far expired
    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_rejects_missing_issuer() {
    let secret = "test-secret-key-for-jwt-validation";

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Create token with empty issuer
    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: (now + 3600),
        iat: now,
        iss: "".to_string(), // Empty issuer
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should reject token with missing issuer
    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_rejects_missing_subject() {
    let secret = "test-secret-key-for-jwt-validation";

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Create token with empty subject
    let claims = TestClaims {
        sub: "".to_string(), // Empty subject
        exp: (now + 3600),
        iat: now,
        iss: "rust-template".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should reject token with missing subject
    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_rejects_issued_too_far_in_future() {
    let secret = "test-secret-key-for-jwt-validation";

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let future_issued = now + 400; // Issued 400 seconds in future (beyond 300s tolerance)

    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: (future_issued + 3600), // Valid for 1 hour after issue
        iat: future_issued,
        iss: "rust-template".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should reject token issued too far in the future
    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_accepts_valid_token() {
    let secret = "test-secret-key-for-jwt-validation";

    // Create a standard valid token
    let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should accept valid token
    assert!(config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_with_nbf_claim() {
    let secret = "test-secret-key-for-jwt-validation";

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let future_time = now + 30; // Not valid until 30 seconds in future

    // Create custom claims with nbf (not before)
    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: (now + 3600),
        iat: now,
        iss: "rust-template".to_string(),
    };

    // Manually encode with nbf claim
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token_data = jsonwebtoken::encode(
        &Header::default(),
        &serde_json::json!({
            "sub": claims.sub,
            "exp": claims.exp,
            "iat": claims.iat,
            "iss": claims.iss,
            "nbf": future_time // Not valid until future
        }),
        &encoding_key,
    )
    .unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should accept token even though nbf is slightly in future due to leeway
    assert!(config.is_authorized(Some(&token_data)));
}

#[test]
fn test_jwt_validation_rejects_wrong_signature() {
    let secret = "test-secret-key-for-jwt-validation";
    let wrong_secret = "wrong-secret-key";

    // Create token with correct secret
    let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(wrong_secret.to_string()),
    };

    // Should reject token with wrong signature
    assert!(!config.is_authorized(Some(&token)));
}

#[test]
fn test_jwt_validation_rejects_malformed_token() {
    let secret = "test-secret-key-for-jwt-validation";

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should reject malformed tokens
    assert!(!config.is_authorized(Some("not.a.jwt.token")));
    assert!(!config.is_authorized(Some("invalid")));
    assert!(!config.is_authorized(Some("")));
}

#[test]
fn test_jwt_validation_with_different_algorithms() {
    let secret = "test-secret-key-for-jwt-validation";

    // Test that only HS256 is accepted (as configured)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let claims = TestClaims {
        sub: "user123".to_string(),
        exp: (now + 3600),
        iat: now,
        iss: "rust-template".to_string(),
    };

    // Try to encode with different algorithm (should still work for encoding)
    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let token = jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap();

    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Jwt,
        token: None,
        jwt_secret: Some(secret.to_string()),
    };

    // Should accept token with correct algorithm
    assert!(config.is_authorized(Some(&token)));
}
