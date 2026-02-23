use http_origin_rule::is_origin_allowed_by_rule;

#[test]
fn integration_matches_exact_origin_rule() {
    assert!(is_origin_allowed_by_rule("https://api.example.com", "https://api.example.com"));
}

#[test]
fn integration_matches_prefix_rule_for_exact_origin() {
    assert!(is_origin_allowed_by_rule("https://example.com/*", "https://example.com"));
}

#[test]
fn integration_matches_prefix_rule_for_path() {
    assert!(is_origin_allowed_by_rule("https://example.com/*", "https://example.com/path"));
}

#[test]
fn integration_rejects_prefix_rule_for_boundaryless_authority_suffix() {
    assert!(!is_origin_allowed_by_rule("https://example.com/*", "https://example.com.evil/path"));
}

#[test]
fn integration_rejects_prefix_rule_for_port_mismatch() {
    assert!(!is_origin_allowed_by_rule("https://example.com/*", "https://example.com:8443/path"));
}

#[test]
fn integration_matches_subdomain_rule_for_valid_subdomain() {
    assert!(is_origin_allowed_by_rule("https://*.example.com", "https://api.example.com"));
}

#[test]
fn integration_rejects_subdomain_rule_for_root_domain() {
    assert!(!is_origin_allowed_by_rule("https://*.example.com", "https://example.com"));
}

#[test]
fn integration_rejects_subdomain_rule_for_sibling_suffix() {
    assert!(!is_origin_allowed_by_rule("https://*.example.com", "https://notexample.com"));
}

#[test]
fn integration_rejects_subdomain_rule_for_malformed_origin_with_path() {
    assert!(!is_origin_allowed_by_rule("https://*.example.com", "https://api.example.com/path"));
}
