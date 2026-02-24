use http_origin_policy::{is_origin_allowed, is_origin_allowed_by_any};

#[test]
fn integration_matches_exact_origin() {
    assert!(is_origin_allowed("https://api.example.com", "https://api.example.com"));
}

#[test]
fn integration_matches_allow_all_wildcard() {
    assert!(is_origin_allowed("*", "https://any.example.com"));
}

#[test]
fn integration_matches_path_prefix_wildcard() {
    assert!(is_origin_allowed("https://example.com/*", "https://example.com/admin"));
}

#[test]
fn integration_matches_exact_origin_for_prefix_wildcard() {
    assert!(is_origin_allowed("https://example.com/*", "https://example.com"));
}

#[test]
fn integration_rejects_prefix_wildcard_for_boundaryless_authority_suffix() {
    assert!(!is_origin_allowed("https://example.com/*", "https://example.com.evil/admin"));
}

#[test]
fn integration_rejects_prefix_wildcard_for_port_mismatch() {
    assert!(!is_origin_allowed("https://example.com/*", "https://example.com:8443/admin"));
}

#[test]
fn integration_matches_subdomain_wildcard() {
    assert!(is_origin_allowed("https://*.example.com", "https://api.example.com"));
}

#[test]
fn integration_rejects_mismatched_origin() {
    assert!(!is_origin_allowed("https://*.example.com", "https://example.org"));
}

#[test]
fn integration_rejects_subdomain_rule_for_root_domain() {
    assert!(!is_origin_allowed("https://*.example.com", "https://example.com"));
}

#[test]
fn integration_rejects_subdomain_rule_for_boundaryless_suffix() {
    assert!(!is_origin_allowed("https://*.example.com", "https://notexample.com"));
}

#[test]
fn integration_rejects_subdomain_rule_for_malformed_origin_with_path() {
    assert!(!is_origin_allowed("https://*.example.com", "https://api.example.com/path"));
}

#[test]
fn integration_matches_when_any_entry_allows_origin() {
    let allowed = vec!["https://api.example.com".to_string(), "https://*.example.org".to_string()];
    assert!(is_origin_allowed_by_any(&allowed, "https://dev.example.org"));
}
