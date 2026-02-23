use http_origin_prefix::matches_prefix_wildcard_rule;

#[test]
fn integration_matches_exact_origin_for_prefix_rule() {
    assert!(matches_prefix_wildcard_rule("https://example.com/*", "https://example.com"));
}

#[test]
fn integration_matches_path_for_prefix_rule() {
    assert!(matches_prefix_wildcard_rule("https://example.com/*", "https://example.com/admin"));
}

#[test]
fn integration_rejects_boundaryless_authority_suffix() {
    assert!(!matches_prefix_wildcard_rule(
        "https://example.com/*",
        "https://example.com.evil/admin"
    ));
}

#[test]
fn integration_rejects_scheme_mismatch() {
    assert!(!matches_prefix_wildcard_rule("https://example.com/*", "http://example.com/admin"));
}

#[test]
fn integration_rejects_port_mismatch() {
    assert!(!matches_prefix_wildcard_rule(
        "https://example.com/*",
        "https://example.com:8443/admin"
    ));
}
