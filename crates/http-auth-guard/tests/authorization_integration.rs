use http::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use http_auth_guard::{is_method_protected, is_request_authorized_for_platform};
use http_auth_policy::{PlatformAuthConfig, PlatformAuthMode};

fn headers_with_authorization(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).expect("valid bearer header value"),
    );
    headers
}

#[test]
fn integration_allows_options_even_without_credentials() {
    let config = PlatformAuthConfig::new(
        PlatformAuthMode::Jwt,
        Some("token".to_string()),
        Some("secret".to_string()),
    );
    let headers = HeaderMap::new();

    assert!(is_request_authorized_for_platform(&http::Method::OPTIONS, &headers, &config));
    assert!(!is_method_protected(&http::Method::OPTIONS, &config));
}

#[test]
fn integration_accepts_matching_basic_token_from_authorization_header() {
    let config = PlatformAuthConfig::new(PlatformAuthMode::Basic, Some("expected".into()), None);
    let headers = headers_with_authorization("expected");

    assert!(is_request_authorized_for_platform(&http::Method::GET, &headers, &config));
    assert!(is_request_authorized_for_platform(&http::Method::POST, &headers, &config));
}

#[test]
fn integration_rejects_wrong_token_in_basic_mode() {
    let config = PlatformAuthConfig::new(PlatformAuthMode::Basic, Some("expected".into()), None);
    let headers = headers_with_authorization("wrong");

    assert!(!is_request_authorized_for_platform(&http::Method::POST, &headers, &config));
}
