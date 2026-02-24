//! Request-level authorization policy for platform auth middleware.
//!
//! This microcrate isolates the decision that determines whether a given
//! request should proceed for platform auth-gated routes.

#![forbid(unsafe_code)]

use http::{HeaderMap, Method};
use http_auth_policy::{PlatformAuthConfig, PlatformAuthMode};
use http_auth_token::extract_auth_token_from_headers;

/// Return `true` when a request should be allowed through the platform auth guard.
pub fn is_request_authorized_for_platform(
    method: &Method,
    headers: &HeaderMap,
    config: &PlatformAuthConfig,
) -> bool {
    if !config.can_enforce_auth() {
        return true;
    }

    if matches!(method, &Method::OPTIONS) {
        return true;
    }

    let provided = extract_auth_token_from_headers(headers);
    config.is_authorized(provided)
}

/// Return `true` when a method should be evaluated by auth logic.
pub fn is_method_protected(method: &Method, config: &PlatformAuthConfig) -> bool {
    config.can_enforce_auth() && method != Method::OPTIONS
}

/// Convenience builder used by policy tests to construct auth config variants.
pub fn build_auth_config(
    mode: PlatformAuthMode,
    token: Option<&str>,
    jwt_secret: Option<&str>,
) -> PlatformAuthConfig {
    PlatformAuthConfig::new(mode, token.map(str::to_string), jwt_secret.map(str::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
    use proptest::prelude::*;

    fn header_token_input() -> impl Strategy<Value = String> {
        "[A-Za-z0-9._~-]{1,24}".prop_map(|value| value.to_string())
    }

    fn any_mode() -> impl Strategy<Value = PlatformAuthMode> {
        prop_oneof![
            Just(PlatformAuthMode::Open),
            Just(PlatformAuthMode::Basic),
            Just(PlatformAuthMode::Jwt),
        ]
    }

    fn sample_method() -> impl Strategy<Value = Method> {
        prop_oneof![
            Just(Method::OPTIONS),
            Just(Method::GET),
            Just(Method::POST),
            Just(Method::PUT),
            Just(Method::DELETE),
        ]
    }

    #[test]
    fn open_mode_allows_any_request() {
        let headers = HeaderMap::new();
        let config = build_auth_config(PlatformAuthMode::Open, None, None);
        assert!(is_request_authorized_for_platform(&Method::POST, &headers, &config));
        assert!(is_request_authorized_for_platform(&Method::GET, &headers, &config));
    }

    #[test]
    fn options_are_always_allowed() {
        let jwt_config = build_auth_config(PlatformAuthMode::Jwt, None, Some("jwt-secret"));
        let headers = HeaderMap::new();

        assert!(is_request_authorized_for_platform(&Method::OPTIONS, &headers, &jwt_config));
    }

    #[test]
    fn basic_mode_requires_matching_token() {
        let mut headers = HeaderMap::new();
        let token = "basic-token";
        let config = build_auth_config(PlatformAuthMode::Basic, Some(token), None);

        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str("Basic nope").unwrap(),
        );
        assert!(!is_request_authorized_for_platform(&Method::GET, &headers, &config));

        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {token}")).unwrap());
        assert!(is_request_authorized_for_platform(&Method::GET, &headers, &config));
    }

    #[test]
    fn jwt_mode_prefers_token_and_rejects_missing_credentials() {
        let mut headers = HeaderMap::new();
        let config = build_auth_config(PlatformAuthMode::Jwt, None, Some("jwt-secret"));

        headers.insert(
            HeaderName::from_static("x-platform-token"),
            HeaderValue::from_str("legacy").unwrap(),
        );
        assert!(!is_request_authorized_for_platform(&Method::POST, &headers, &config));

        let token = "legacy";
        let mode_with_secret =
            build_auth_config(PlatformAuthMode::Jwt, Some(token), Some("jwt-secret"));
        assert!(is_request_authorized_for_platform(&Method::POST, &headers, &mode_with_secret));
    }

    #[test]
    fn method_protection_follows_mode_requirements() {
        let config_none = build_auth_config(PlatformAuthMode::Open, None, None);
        assert!(!is_method_protected(&Method::GET, &config_none));
        assert!(!is_method_protected(&Method::OPTIONS, &config_none));

        let config_basic = build_auth_config(PlatformAuthMode::Basic, Some("token"), None);
        assert!(is_method_protected(&Method::GET, &config_basic));
        assert!(!is_method_protected(&Method::OPTIONS, &config_basic));
    }

    #[test]
    fn basic_header_precedence_respects_auth_token() {
        let mut headers = HeaderMap::new();
        let config = build_auth_config(PlatformAuthMode::Basic, Some("legacy"), Some("jwt-secret"));

        headers.insert(
            HeaderName::from_static("x-platform-token"),
            HeaderValue::from_str("legacy").unwrap(),
        );
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str("Bearer bad-token").unwrap(),
        );
        assert!(!is_request_authorized_for_platform(&Method::POST, &headers, &config));
    }

    #[test]
    fn basic_header_precedence_accepts_bearer_token_when_matching() {
        let mut headers = HeaderMap::new();
        let config = build_auth_config(PlatformAuthMode::Basic, Some("legacy"), Some("jwt-secret"));

        headers.insert(
            HeaderName::from_static("x-platform-token"),
            HeaderValue::from_str("legacy").unwrap(),
        );
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str("Bearer legacy").unwrap(),
        );
        assert!(is_request_authorized_for_platform(&Method::POST, &headers, &config));
    }

    mod proptests {
        use super::*;

        proptest! {
            #[test]
            fn prop_options_are_always_authorized(
                mode in any_mode(),
                token in proptest::option::of(header_token_input()),
                jwt_secret in proptest::option::of(header_token_input())
            ) {
                let config = build_auth_config(mode, token.as_deref(), jwt_secret.as_deref());
                let headers = HeaderMap::new();

                prop_assert!(is_request_authorized_for_platform(&Method::OPTIONS, &headers, &config));
            }
        }

        proptest! {
            #[test]
            fn prop_basic_mode_authorization_matches_expected_token(
                token in header_token_input(),
                candidate in header_token_input(),
                method in sample_method()
            ) {
                let config = build_auth_config(PlatformAuthMode::Basic, Some(&token), None);
                let mut headers = HeaderMap::new();
                headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {candidate}")).unwrap());
                let expected = if method == Method::OPTIONS { true } else { candidate == token };

                prop_assert_eq!(
                    is_request_authorized_for_platform(&method, &headers, &config),
                    expected
                );
            }
        }
    }
}
