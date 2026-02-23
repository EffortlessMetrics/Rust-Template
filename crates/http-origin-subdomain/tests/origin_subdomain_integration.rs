use http_origin_subdomain::matches_subdomain_wildcard_rule;

#[test]
fn integration_matches_valid_subdomain() {
    assert!(matches_subdomain_wildcard_rule("https://*.example.com", "https://api.example.com"));
}

#[test]
fn integration_rejects_root_domain() {
    assert!(!matches_subdomain_wildcard_rule("https://*.example.com", "https://example.com"));
}

#[test]
fn integration_rejects_boundaryless_suffix() {
    assert!(!matches_subdomain_wildcard_rule("https://*.example.com", "https://notexample.com"));
}

#[test]
fn integration_rejects_scheme_mismatch() {
    assert!(!matches_subdomain_wildcard_rule("https://*.example.com", "http://api.example.com"));
}

#[test]
fn integration_rejects_malformed_origin_with_path_suffix() {
    assert!(!matches_subdomain_wildcard_rule(
        "https://*.example.com",
        "https://api.example.com/path"
    ));
}

#[test]
fn integration_rejects_malformed_origin_with_userinfo() {
    assert!(!matches_subdomain_wildcard_rule(
        "https://*.example.com",
        "https://user@api.example.com"
    ));
}
