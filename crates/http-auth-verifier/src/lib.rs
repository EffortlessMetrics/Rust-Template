//! Token verification primitives for platform HTTP authentication.
//!
//! This crate is intentionally small and framework-agnostic.

#![forbid(unsafe_code)]

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
#[cfg(test)]
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims used by platform auth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject identifier.
    pub sub: String,
    /// Expiration timestamp (unix seconds).
    pub exp: u64,
    /// Issued-at timestamp (unix seconds).
    pub iat: u64,
    /// Issuer identifier.
    pub iss: String,
}

/// Validate an optional provided token against configured credentials.
///
/// Authorization succeeds when either:
/// - the provided token exactly matches `expected_basic_token` (constant-time), or
/// - the provided token is a valid JWT signed by `jwt_secret`.
///
/// Basic token matching is evaluated before JWT validation to allow literal token
/// values that contain `.` separators.
pub fn authorize_token(
    provided: Option<&str>,
    expected_basic_token: Option<&str>,
    jwt_secret: Option<&str>,
) -> bool {
    let Some(candidate) = provided else {
        return false;
    };

    if expected_basic_token.is_some_and(|expected| constant_time_eq(expected, candidate)) {
        return true;
    }

    jwt_secret.is_some_and(|secret| validate_jwt_token(candidate, secret))
}

/// Validate JWT token with HMAC-SHA256 and strict claim checks.
pub fn validate_jwt_token(token: &str, secret: &str) -> bool {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.insert("exp".to_string());
    validation.validate_exp = true;
    validation.leeway = 60;
    validation.validate_nbf = true;

    let decoding_key = DecodingKey::from_secret(secret.as_ref());

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            let claims = token_data.claims;

            if claims.iss.is_empty() {
                tracing::debug!("JWT validation failed: missing issuer");
                return false;
            }

            if claims.sub.is_empty() {
                tracing::debug!("JWT validation failed: missing subject");
                return false;
            }

            let now = jsonwebtoken::get_current_timestamp();
            if claims.iat > now + 300 {
                tracing::debug!("JWT validation failed: token issued too far in the future");
                return false;
            }

            true
        }
        Err(error) => {
            tracing::debug!("JWT validation failed: {}", error);
            false
        }
    }
}

/// Constant-time compare for equal-length strings.
pub fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.bytes().zip(right.bytes()).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
fn create_jwt_token(
    secret: &str,
    subject: &str,
    issuer: &str,
    expires_in_seconds: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let claims = Claims {
        sub: subject.to_string(),
        exp: now + expires_in_seconds,
        iat: now,
        iss: issuer.to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorizes_matching_basic_token() {
        assert!(authorize_token(Some("secret"), Some("secret"), None));
    }

    #[test]
    fn rejects_non_matching_basic_token() {
        assert!(!authorize_token(Some("other"), Some("secret"), None));
    }

    #[test]
    fn authorizes_jwt_like_basic_token_value() {
        assert!(authorize_token(Some("dot.token.value"), Some("dot.token.value"), None));
    }

    #[test]
    fn authorizes_jwt_like_basic_token_even_when_jwt_secret_is_present() {
        assert!(authorize_token(
            Some("dot.token.value"),
            Some("dot.token.value"),
            Some("wrong-secret")
        ));
    }

    #[test]
    fn rejects_when_token_is_missing() {
        assert!(!authorize_token(None, Some("secret"), Some("jwt-secret")));
    }

    #[test]
    fn authorizes_valid_jwt_token() {
        let secret = "test-secret";
        let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();

        assert!(authorize_token(Some(token.as_str()), None, Some(secret)));
    }

    #[test]
    fn rejects_jwt_with_invalid_signature() {
        let token = create_jwt_token("correct-secret", "user123", "rust-template", 3600).unwrap();

        assert!(!authorize_token(Some(token.as_str()), None, Some("wrong-secret")));
    }

    #[test]
    fn rejects_expired_jwt_token() {
        let secret = "test-secret";
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let claims = Claims {
            sub: "user123".to_string(),
            exp: now - 3600,
            iat: now - 7200,
            iss: "rust-template".to_string(),
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
            .unwrap();

        assert!(!authorize_token(Some(token.as_str()), None, Some(secret)));
    }

    #[test]
    fn rejects_jwt_with_missing_required_claim_content() {
        let secret = "test-secret";
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let encoding_key = EncodingKey::from_secret(secret.as_ref());

        let missing_issuer = encode(
            &Header::default(),
            &serde_json::json!({
                "sub": "user123",
                "exp": now + 3600,
                "iat": now,
                "iss": ""
            }),
            &encoding_key,
        )
        .unwrap();

        let missing_subject = encode(
            &Header::default(),
            &serde_json::json!({
                "sub": "",
                "exp": now + 3600,
                "iat": now,
                "iss": "rust-template"
            }),
            &encoding_key,
        )
        .unwrap();

        assert!(!authorize_token(Some(missing_issuer.as_str()), None, Some(secret)));
        assert!(!authorize_token(Some(missing_subject.as_str()), None, Some(secret)));
    }

    #[test]
    fn rejects_jwt_issued_too_far_in_future() {
        let secret = "test-secret";
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Use a significantly larger offset to avoid flakiness on slow CI runners
        // where the time difference between test setup and validation execution
        // could exceed 1 second.
        let future_offset = 600;

        let claims = Claims {
            sub: "user123".to_string(),
            exp: now + 3600,
            iat: now + future_offset,
            iss: "rust-template".to_string(),
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
            .unwrap();

        assert!(!authorize_token(Some(token.as_str()), None, Some(secret)));
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_constant_time_eq_matches_standard_equality(
                left in ".*",
                right in ".*",
            ) {
                prop_assert_eq!(constant_time_eq(left.as_str(), right.as_str()), left == right);
            }

            #[test]
            fn prop_authorize_token_accepts_exact_basic_token(
                token in "[A-Za-z0-9._~-]{1,64}",
            ) {
                prop_assert!(authorize_token(Some(token.as_str()), Some(token.as_str()), None));
            }

            #[test]
            fn prop_authorize_token_rejects_non_matching_basic_token(
                expected in "[A-Za-z0-9._~-]{1,64}",
                candidate in "[A-Za-z0-9._~-]{1,64}",
            ) {
                prop_assume!(expected != candidate);
                prop_assert!(!authorize_token(Some(candidate.as_str()), Some(expected.as_str()), None));
            }
        }
    }
}
