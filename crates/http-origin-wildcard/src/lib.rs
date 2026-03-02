//! Wildcard origin-rule parser primitives used by CORS allowlist matchers.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

use http_origin_parser::{HttpOriginScheme, parse_http_origin};

/// Parsed components of a wildcard origin rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedWildcardOriginRule<'a> {
    /// Parsed HTTP scheme.
    pub scheme: HttpOriginScheme,
    /// Wildcard domain suffix (without `*.` prefix).
    pub wildcard_domain: &'a str,
}

/// Parse wildcard origin rules of the form `https://*.example.com`.
///
/// Accepted forms:
/// - `http://*.example.com`
/// - `https://*.example.com`
///
/// Rejected values include:
/// - Rules without wildcard prefix
/// - Empty wildcard suffix (`https://*.`)
/// - Invalid origin syntax
pub fn parse_subdomain_wildcard_rule(value: &str) -> Option<ParsedWildcardOriginRule<'_>> {
    let parsed = parse_http_origin(value)?;
    let wildcard_domain = parsed.authority.strip_prefix("*.")?;

    if wildcard_domain.is_empty() {
        return None;
    }

    Some(ParsedWildcardOriginRule { scheme: parsed.scheme, wildcard_domain })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_https_wildcard_rule() {
        let parsed = parse_subdomain_wildcard_rule("https://*.example.com").unwrap();
        assert_eq!(parsed.scheme, HttpOriginScheme::Https);
        assert_eq!(parsed.wildcard_domain, "example.com");
    }

    #[test]
    fn parses_http_wildcard_rule() {
        let parsed = parse_subdomain_wildcard_rule("http://*.localhost:3000").unwrap();
        assert_eq!(parsed.scheme, HttpOriginScheme::Http);
        assert_eq!(parsed.wildcard_domain, "localhost:3000");
    }

    #[test]
    fn rejects_rule_without_wildcard_prefix() {
        assert_eq!(parse_subdomain_wildcard_rule("https://example.com"), None);
    }

    #[test]
    fn rejects_rule_with_empty_wildcard_domain() {
        assert_eq!(parse_subdomain_wildcard_rule("https://*."), None);
    }

    #[test]
    fn rejects_non_http_scheme() {
        assert_eq!(parse_subdomain_wildcard_rule("ftp://*.example.com"), None);
    }

    #[test]
    fn rejects_path_suffix() {
        assert_eq!(parse_subdomain_wildcard_rule("https://*.example.com/path"), None);
    }

    fn domain() -> impl Strategy<Value = String> {
        "[a-z0-9]{1,10}\\.[a-z]{2,8}".prop_map(|value| value.to_string())
    }

    proptest! {
        #[test]
        fn prop_parses_wildcard_rule(domain in domain()) {
            let rule = format!("https://*.{domain}");
            let parsed = parse_subdomain_wildcard_rule(rule.as_str()).unwrap();
            prop_assert_eq!(parsed.scheme, HttpOriginScheme::Https);
            prop_assert_eq!(parsed.wildcard_domain, domain.as_str());
        }

        #[test]
        fn prop_rejects_non_wildcard_rule(domain in domain()) {
            let rule = format!("https://{domain}");
            prop_assert_eq!(parse_subdomain_wildcard_rule(rule.as_str()), None);
        }
    }
}
