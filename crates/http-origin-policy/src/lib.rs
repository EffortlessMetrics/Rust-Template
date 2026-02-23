//! Origin allowlist matching primitives for CORS policy checks.
//!
//! This crate is intentionally small and framework-agnostic.

#![forbid(unsafe_code)]

pub use http_origin_rule::is_origin_allowed_by_rule;

/// Returns `true` when `origin` matches at least one allowlist entry.
pub fn is_origin_allowed_by_any(allowed_origins: &[String], origin: &str) -> bool {
    allowed_origins.iter().any(|allowed| is_origin_allowed(allowed, origin))
}

/// Returns `true` when `origin` matches a single allowlist rule.
///
/// Supported forms:
/// - `*` (allow all)
/// - exact origin match
/// - prefix wildcard with `/*`
/// - subdomain wildcard with `*.` (for example `https://*.example.com`)
pub fn is_origin_allowed(allowed: &str, origin: &str) -> bool {
    is_origin_allowed_by_rule(allowed, origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_wildcard() {
        assert!(is_origin_allowed("*", "https://any.example.com"));
    }

    #[test]
    fn matches_exact_origin() {
        assert!(is_origin_allowed("https://api.example.com", "https://api.example.com"));
    }

    #[test]
    fn matches_prefix_wildcard() {
        assert!(is_origin_allowed("https://example.com/*", "https://example.com/path"));
    }

    #[test]
    fn matches_subdomain_wildcard() {
        assert!(is_origin_allowed("https://*.example.com", "https://api.example.com"));
    }

    #[test]
    fn rejects_subdomain_wildcard_root_domain() {
        assert!(!is_origin_allowed("https://*.example.com", "https://example.com"));
    }

    #[test]
    fn rejects_subdomain_wildcard_suffix_without_label_boundary() {
        assert!(!is_origin_allowed("https://*.example.com", "https://notexample.com"));
    }

    #[test]
    fn rejects_subdomain_wildcard_scheme_mismatch() {
        assert!(!is_origin_allowed("https://*.example.com", "http://api.example.com"));
    }

    #[test]
    fn rejects_non_matching_origin() {
        assert!(!is_origin_allowed("https://example.com", "https://other.com"));
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn origin() -> impl Strategy<Value = String> {
            "[a-z0-9]{1,12}\\.[a-z]{2,8}".prop_map(|host| format!("https://{host}"))
        }

        proptest! {
            #[test]
            fn prop_exact_origin_match_is_allowed(origin in origin()) {
                prop_assert!(is_origin_allowed(origin.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_wildcard_allows_any_origin(origin in origin()) {
                prop_assert!(is_origin_allowed("*", origin.as_str()));
            }

            #[test]
            fn prop_is_origin_allowed_by_any_matches_member(origin in origin()) {
                let allowed = vec!["https://a.example.com".to_string(), origin.clone()];
                prop_assert!(is_origin_allowed_by_any(&allowed, origin.as_str()));
            }
        }
    }
}
