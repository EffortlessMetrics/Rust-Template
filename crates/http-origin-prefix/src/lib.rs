//! Prefix wildcard matcher for CORS origin allowlist rules.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

use http_origin_boundary::has_path_boundary_or_empty;
use http_origin_parser::parse_http_origin;

/// Returns `true` when `origin` matches a prefix wildcard rule.
///
/// Supported form for `allowed`:
/// - `https://example.com/*`
/// - `http://localhost:3000/*`
///
/// Security semantics:
/// - The prefix before `/*` must be a valid HTTP/HTTPS origin
/// - Scheme and authority must match exactly
/// - Boundaryless authority suffixes do not match
///   (`https://example.com.evil` is rejected for `https://example.com/*`)
pub fn matches_prefix_wildcard_rule(allowed: &str, origin: &str) -> bool {
    let Some(prefix) = allowed.strip_suffix("/*") else {
        return false;
    };

    if parse_http_origin(prefix).is_none() {
        return false;
    }

    if origin.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return false;
    }

    let Some(suffix) = origin.strip_prefix(prefix) else {
        return false;
    };

    has_path_boundary_or_empty(suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_origin_with_prefix_rule() {
        assert!(matches_prefix_wildcard_rule("https://example.com/*", "https://example.com"));
    }

    #[test]
    fn matches_origin_path_with_prefix_rule() {
        assert!(matches_prefix_wildcard_rule("https://example.com/*", "https://example.com/path"));
    }

    #[test]
    fn rejects_rule_without_prefix_suffix() {
        assert!(!matches_prefix_wildcard_rule("https://example.com", "https://example.com/path"));
    }

    #[test]
    fn rejects_rule_with_non_origin_prefix() {
        assert!(!matches_prefix_wildcard_rule(
            "https://example.com/path/*",
            "https://example.com/path"
        ));
    }

    #[test]
    fn rejects_boundaryless_authority_suffix() {
        assert!(!matches_prefix_wildcard_rule(
            "https://example.com/*",
            "https://example.com.evil/path"
        ));
    }

    #[test]
    fn rejects_port_mismatch() {
        assert!(!matches_prefix_wildcard_rule(
            "https://example.com/*",
            "https://example.com:8443/path"
        ));
    }

    #[test]
    fn rejects_query_without_path_boundary() {
        assert!(!matches_prefix_wildcard_rule(
            "https://example.com/*",
            "https://example.com?debug=true"
        ));
    }

    #[test]
    fn rejects_ascii_whitespace() {
        assert!(!matches_prefix_wildcard_rule(
            "https://example.com/*",
            "https://example.com/path\n"
        ));
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn domain() -> impl Strategy<Value = String> {
            "[a-z0-9]{1,12}\\.[a-z]{2,8}".prop_map(|value| value.to_string())
        }

        fn label() -> impl Strategy<Value = String> {
            "[a-z0-9]{1,12}".prop_map(|value| value.to_string())
        }

        fn path_segment() -> impl Strategy<Value = String> {
            "[A-Za-z0-9._~-]{1,16}".prop_map(|value| value.to_string())
        }

        proptest! {
            #[test]
            fn prop_prefix_wildcard_matches_same_origin(domain in domain()) {
                let allowed = format!("https://{domain}/*");
                let origin = format!("https://{domain}");
                prop_assert!(matches_prefix_wildcard_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_prefix_wildcard_matches_path(domain in domain(), path in path_segment()) {
                let allowed = format!("https://{domain}/*");
                let origin = format!("https://{domain}/{path}");
                prop_assert!(matches_prefix_wildcard_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_prefix_wildcard_rejects_boundaryless_authority_suffix(
                domain in domain(),
                suffix in label(),
                path in path_segment(),
            ) {
                let allowed = format!("https://{domain}/*");
                let origin = format!("https://{domain}.{suffix}/{path}");
                prop_assert!(!matches_prefix_wildcard_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_prefix_wildcard_rejects_scheme_mismatch(domain in domain(), path in path_segment()) {
                let allowed = format!("https://{domain}/*");
                let origin = format!("http://{domain}/{path}");
                prop_assert!(!matches_prefix_wildcard_rule(allowed.as_str(), origin.as_str()));
            }
        }
    }
}
