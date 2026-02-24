use http_bearer_token::extract_bearer_token;

#[test]
fn integration_extracts_token_from_authorization_value() {
    assert_eq!(extract_bearer_token("Bearer integration-token"), Some("integration-token"));
}

#[test]
fn integration_accepts_mixed_case_bearer_scheme() {
    assert_eq!(extract_bearer_token("BeArEr mixed-case"), Some("mixed-case"));
}

#[test]
fn integration_rejects_other_auth_schemes() {
    assert_eq!(extract_bearer_token("Digest abc123"), None);
}
