//! Token extraction primitives for platform HTTP authentication headers.
//!
//! This crate is intentionally small and framework-agnostic. It operates on
//! `http::HeaderMap` and raw header values to decide which token should be
//! validated by auth policy code.

#![forbid(unsafe_code)]

use http::HeaderMap;
pub use http_bearer_token::extract_bearer_token;

/// Standard authorization header name.
pub const AUTHORIZATION_HEADER: &str = "authorization";
/// Legacy platform token header name.
pub const PLATFORM_AUTH_HEADER: &str = "x-platform-token";

/// Extract the effective auth token from header values.
///
/// Precedence order:
/// 1. `Authorization: Bearer <token>` (if parsable)
/// 2. `x-platform-token` (legacy fallback)
pub fn extract_auth_token<'a>(
    authorization_header: Option<&'a str>,
    platform_auth_header: Option<&'a str>,
) -> Option<&'a str> {
    authorization_header.and_then(extract_bearer_token).or(platform_auth_header)
}

/// Extract auth token directly from an HTTP header map.
///
/// Invalid UTF-8 header values are treated as absent.
pub fn extract_auth_token_from_headers(headers: &HeaderMap) -> Option<&str> {
    let authorization = headers.get(AUTHORIZATION_HEADER).and_then(|value| value.to_str().ok());
    let platform = headers.get(PLATFORM_AUTH_HEADER).and_then(|value| value.to_str().ok());
    extract_auth_token(authorization, platform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, HeaderValue};

    #[test]
    fn extracts_bearer_token_from_authorization_header() {
        assert_eq!(extract_bearer_token("Bearer abc.def.ghi"), Some("abc.def.ghi"));
    }

    #[test]
    fn extracts_case_insensitive_bearer_scheme() {
        assert_eq!(extract_bearer_token("bEaReR token"), Some("token"));
    }

    #[test]
    fn falls_back_to_legacy_platform_header_when_authorization_is_absent() {
        assert_eq!(extract_auth_token(None, Some("legacy-token")), Some("legacy-token"));
    }

    #[test]
    fn falls_back_to_legacy_platform_header_when_authorization_is_not_bearer() {
        assert_eq!(
            extract_auth_token(Some("Basic dXNlcjpwYXNz"), Some("legacy-token")),
            Some("legacy-token")
        );
    }

    #[test]
    fn authorization_bearer_takes_precedence_over_legacy_platform_header() {
        assert_eq!(
            extract_auth_token(Some("Bearer token-from-authorization"), Some("legacy-token")),
            Some("token-from-authorization")
        );
    }

    #[test]
    fn extract_auth_token_from_headers_prefers_authorization_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION_HEADER, HeaderValue::from_static("Bearer auth-token"));
        headers.insert(PLATFORM_AUTH_HEADER, HeaderValue::from_static("legacy-token"));

        assert_eq!(extract_auth_token_from_headers(&headers), Some("auth-token"));
    }

    #[test]
    fn extract_auth_token_from_headers_uses_legacy_when_authorization_is_invalid_utf8() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION_HEADER, HeaderValue::from_bytes(&[0xFF]).unwrap());
        headers.insert(PLATFORM_AUTH_HEADER, HeaderValue::from_static("legacy-token"));

        assert_eq!(extract_auth_token_from_headers(&headers), Some("legacy-token"));
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn token() -> impl Strategy<Value = String> {
            "[A-Za-z0-9._~:-]{0,64}".prop_map(|s| s.to_string())
        }

        proptest! {
            #[test]
            fn prop_extract_auth_token_prefers_bearer(token in token(), legacy in token()) {
                let authorization = format!("Bearer {token}");
                let parsed = extract_auth_token(Some(authorization.as_str()), Some(legacy.as_str()));
                prop_assert_eq!(parsed, Some(token.as_str()));
            }

            #[test]
            fn prop_extract_auth_token_falls_back_when_authorization_not_bearer(auth in token(), legacy in token()) {
                let authorization = format!("Basic {auth}");
                let parsed = extract_auth_token(Some(authorization.as_str()), Some(legacy.as_str()));
                prop_assert_eq!(parsed, Some(legacy.as_str()));
            }

            #[test]
            fn prop_extract_bearer_round_trips(token in token()) {
                let authorization = format!("Bearer {token}");
                let parsed = extract_bearer_token(authorization.as_str());
                prop_assert_eq!(parsed, Some(token.as_str()));
            }
        }
    }
}
