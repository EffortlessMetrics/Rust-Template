//! Single-rule origin allowlist matcher for CORS checks.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

use http_origin_prefix::matches_prefix_wildcard_rule;
use http_origin_subdomain::matches_subdomain_wildcard_rule;

/// Returns `true` when `origin` matches one allowlist rule.
///
/// Supported forms:
/// - `*` (allow all)
/// - exact origin match
/// - prefix wildcard with `/*`
/// - subdomain wildcard with `*.` (for example `https://*.example.com`)
pub fn is_origin_allowed_by_rule(allowed: &str, origin: &str) -> bool {
    if allowed == "*" || allowed == origin {
        return true;
    }

    if matches_prefix_wildcard_rule(allowed, origin) {
        return true;
    }

    matches_subdomain_wildcard_rule(allowed, origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_wildcard_all_rule() {
        assert!(is_origin_allowed_by_rule("*", "https://any.example.com"));
    }

    #[test]
    fn matches_exact_origin() {
        assert!(is_origin_allowed_by_rule("https://api.example.com", "https://api.example.com"));
    }

    #[test]
    fn matches_prefix_wildcard() {
        assert!(is_origin_allowed_by_rule("https://example.com/*", "https://example.com/path"));
    }

    #[test]
    fn matches_prefix_wildcard_for_exact_origin() {
        assert!(is_origin_allowed_by_rule("https://example.com/*", "https://example.com"));
    }

    #[test]
    fn rejects_prefix_wildcard_boundaryless_authority_suffix() {
        assert!(!is_origin_allowed_by_rule(
            "https://example.com/*",
            "https://example.com.evil/path"
        ));
    }

    #[test]
    fn rejects_prefix_wildcard_port_mismatch() {
        assert!(!is_origin_allowed_by_rule(
            "https://example.com/*",
            "https://example.com:8443/path"
        ));
    }

    #[test]
    fn matches_subdomain_wildcard() {
        assert!(is_origin_allowed_by_rule("https://*.example.com", "https://api.example.com"));
    }

    #[test]
    fn rejects_subdomain_wildcard_scheme_mismatch() {
        assert!(!is_origin_allowed_by_rule("https://*.example.com", "http://api.example.com"));
    }

    #[test]
    fn rejects_subdomain_wildcard_root_domain() {
        assert!(!is_origin_allowed_by_rule("https://*.example.com", "https://example.com"));
    }

    #[test]
    fn rejects_subdomain_wildcard_suffix_without_label_boundary() {
        assert!(!is_origin_allowed_by_rule("https://*.example.com", "https://notexample.com"));
    }

    #[test]
    fn rejects_non_matching_origin() {
        assert!(!is_origin_allowed_by_rule("https://example.com", "https://other.com"));
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn domain() -> impl Strategy<Value = String> {
            "[a-z0-9]{1,10}\\.[a-z]{2,8}".prop_map(|value| value.to_string())
        }

        fn label() -> impl Strategy<Value = String> {
            "[a-z0-9]{1,10}".prop_map(|value| value.to_string())
        }

        proptest! {
            #[test]
            fn prop_exact_rule_always_matches(origin in "https://[a-z0-9]{1,12}\\.[a-z]{2,8}") {
                prop_assert!(is_origin_allowed_by_rule(origin.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_wildcard_rule_allows_any_origin(origin in "https?://[a-z0-9.-]{1,32}") {
                prop_assert!(is_origin_allowed_by_rule("*", origin.as_str()));
            }

            #[test]
            fn prop_prefix_wildcard_rejects_boundaryless_authority_suffix(
                domain in domain(),
                suffix in label(),
            ) {
                let allowed = format!("https://{domain}/*");
                let origin = format!("https://{domain}.{suffix}/path");
                prop_assert!(!is_origin_allowed_by_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_subdomain_wildcard_matches_subdomain(domain in domain(), subdomain in label()) {
                let allowed = format!("https://*.{domain}");
                let origin = format!("https://{subdomain}.{domain}");
                prop_assert!(is_origin_allowed_by_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_subdomain_wildcard_rejects_boundaryless_suffix(domain in domain(), prefix in label()) {
                let allowed = format!("https://*.{domain}");
                let origin = format!("https://{prefix}{domain}");
                prop_assert!(!is_origin_allowed_by_rule(allowed.as_str(), origin.as_str()));
            }
        }
    }
}
