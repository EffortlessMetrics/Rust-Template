//! Subdomain wildcard matcher for CORS origin allowlist rules.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

use http_origin_boundary::has_label_boundary_before_suffix;
use http_origin_parser::parse_http_origin;

/// Returns `true` when `origin` matches a subdomain wildcard rule.
///
/// Supported form for `allowed`:
/// - `https://*.example.com`
/// - `http://*.example.com`
///
/// Security semantics:
/// - Scheme must match exactly (`http://` vs `https://`)
/// - Root domain does not match wildcard (`https://example.com` is rejected)
/// - Boundaryless suffix does not match (`https://notexample.com` is rejected)
pub fn matches_subdomain_wildcard_rule(allowed: &str, origin: &str) -> bool {
    let Some(allowed_origin) = parse_http_origin(allowed) else {
        return false;
    };

    let Some(wildcard_domain) = allowed_origin.authority.strip_prefix("*.") else {
        return false;
    };

    if wildcard_domain.is_empty() {
        return false;
    }

    let Some(origin_parts) = parse_http_origin(origin) else {
        return false;
    };

    if allowed_origin.scheme != origin_parts.scheme {
        return false;
    }

    let origin_authority = origin_parts.authority;

    if origin_authority.len() <= wildcard_domain.len() {
        return false;
    }

    if !origin_authority.ends_with(wildcard_domain) {
        return false;
    }

    has_label_boundary_before_suffix(origin_authority, wildcard_domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_subdomain_wildcard() {
        assert!(matches_subdomain_wildcard_rule(
            "https://*.example.com",
            "https://api.example.com"
        ));
    }

    #[test]
    fn rejects_subdomain_wildcard_scheme_mismatch() {
        assert!(!matches_subdomain_wildcard_rule(
            "https://*.example.com",
            "http://api.example.com"
        ));
    }

    #[test]
    fn rejects_subdomain_wildcard_root_domain() {
        assert!(!matches_subdomain_wildcard_rule("https://*.example.com", "https://example.com"));
    }

    #[test]
    fn rejects_subdomain_wildcard_suffix_without_label_boundary() {
        assert!(!matches_subdomain_wildcard_rule(
            "https://*.example.com",
            "https://notexample.com"
        ));
    }

    #[test]
    fn rejects_rule_without_wildcard_prefix() {
        assert!(!matches_subdomain_wildcard_rule("https://example.com", "https://api.example.com"));
    }

    #[test]
    fn rejects_rule_with_empty_domain() {
        assert!(!matches_subdomain_wildcard_rule("https://*.", "https://api.example.com"));
    }

    #[test]
    fn rejects_origin_with_path_suffix() {
        assert!(!matches_subdomain_wildcard_rule(
            "https://*.example.com",
            "https://api.example.com/path"
        ));
    }

    #[test]
    fn rejects_origin_with_userinfo() {
        assert!(!matches_subdomain_wildcard_rule(
            "https://*.example.com",
            "https://user@api.example.com"
        ));
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
            fn prop_subdomain_wildcard_matches_subdomain(domain in domain(), subdomain in label()) {
                let allowed = format!("https://*.{domain}");
                let origin = format!("https://{subdomain}.{domain}");
                prop_assert!(matches_subdomain_wildcard_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_subdomain_wildcard_rejects_root_domain(domain in domain()) {
                let allowed = format!("https://*.{domain}");
                let origin = format!("https://{domain}");
                prop_assert!(!matches_subdomain_wildcard_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_subdomain_wildcard_rejects_boundaryless_suffix(domain in domain(), prefix in label()) {
                let allowed = format!("https://*.{domain}");
                let origin = format!("https://{prefix}{domain}");
                prop_assert!(!matches_subdomain_wildcard_rule(allowed.as_str(), origin.as_str()));
            }

            #[test]
            fn prop_subdomain_wildcard_rejects_scheme_mismatch(domain in domain(), subdomain in label()) {
                let allowed = format!("https://*.{domain}");
                let origin = format!("http://{subdomain}.{domain}");
                prop_assert!(!matches_subdomain_wildcard_rule(allowed.as_str(), origin.as_str()));
            }
        }
    }
}
