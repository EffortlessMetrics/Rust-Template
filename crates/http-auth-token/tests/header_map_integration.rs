use http::{HeaderMap, HeaderValue};
use http_auth_token::{
    AUTHORIZATION_HEADER, PLATFORM_AUTH_HEADER, extract_auth_token_from_headers,
};

#[test]
fn resolves_token_from_authorization_header() {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION_HEADER, HeaderValue::from_static("Bearer bearer-token"));

    assert_eq!(extract_auth_token_from_headers(&headers), Some("bearer-token"));
}

#[test]
fn falls_back_to_platform_header_when_authorization_is_not_bearer() {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION_HEADER, HeaderValue::from_static("Basic abc123"));
    headers.insert(PLATFORM_AUTH_HEADER, HeaderValue::from_static("legacy-token"));

    assert_eq!(extract_auth_token_from_headers(&headers), Some("legacy-token"));
}

#[test]
fn preserves_authorization_precedence_over_legacy_header() {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION_HEADER, HeaderValue::from_static("Bearer invalid.jwt.token"));
    headers.insert(PLATFORM_AUTH_HEADER, HeaderValue::from_static("legacy-token"));

    assert_eq!(extract_auth_token_from_headers(&headers), Some("invalid.jwt.token"));
}
