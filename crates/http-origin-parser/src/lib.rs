//! HTTP/HTTPS origin parser primitives used by CORS allowlist matchers.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

/// Supported schemes for parsed origins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpOriginScheme {
    /// `http://`
    Http,
    /// `https://`
    Https,
}

impl HttpOriginScheme {
    /// Canonical scheme prefix.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http://",
            Self::Https => "https://",
        }
    }
}

/// Parsed components of an HTTP/HTTPS origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedHttpOrigin<'a> {
    /// Parsed scheme.
    pub scheme: HttpOriginScheme,
    /// Authority component (host or host:port).
    pub authority: &'a str,
}

/// Parse an HTTP/HTTPS origin into scheme and authority.
///
/// Accepted form:
/// - `http://<authority>`
/// - `https://<authority>`
///
/// Rejected values include:
/// - Non-HTTP schemes
/// - Missing authority
/// - Path/query/fragment suffixes
/// - Userinfo (`@`)
/// - ASCII whitespace
pub fn parse_http_origin(value: &str) -> Option<ParsedHttpOrigin<'_>> {
    let (scheme, authority) =
        if let Some(rest) = value.strip_prefix(HttpOriginScheme::Https.as_str()) {
            (HttpOriginScheme::Https, rest)
        } else if let Some(rest) = value.strip_prefix(HttpOriginScheme::Http.as_str()) {
            (HttpOriginScheme::Http, rest)
        } else {
            return None;
        };

    if authority.is_empty() {
        return None;
    }

    if authority.chars().any(|ch| matches!(ch, '/' | '?' | '#' | '@')) {
        return None;
    }

    if authority.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return None;
    }

    Some(ParsedHttpOrigin { scheme, authority })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_origin() {
        let parsed = parse_http_origin("https://api.example.com").unwrap();
        assert_eq!(parsed.scheme, HttpOriginScheme::Https);
        assert_eq!(parsed.authority, "api.example.com");
    }

    #[test]
    fn parses_http_origin_with_port() {
        let parsed = parse_http_origin("http://localhost:8080").unwrap();
        assert_eq!(parsed.scheme, HttpOriginScheme::Http);
        assert_eq!(parsed.authority, "localhost:8080");
    }

    #[test]
    fn rejects_non_http_scheme() {
        assert_eq!(parse_http_origin("ftp://example.com"), None);
    }

    #[test]
    fn rejects_missing_authority() {
        assert_eq!(parse_http_origin("https://"), None);
    }

    #[test]
    fn rejects_path_query_and_fragment_suffixes() {
        assert_eq!(parse_http_origin("https://api.example.com/path"), None);
        assert_eq!(parse_http_origin("https://api.example.com?debug=true"), None);
        assert_eq!(parse_http_origin("https://api.example.com#section"), None);
    }

    #[test]
    fn rejects_userinfo() {
        assert_eq!(parse_http_origin("https://user@api.example.com"), None);
    }

    #[test]
    fn rejects_ascii_whitespace() {
        assert_eq!(parse_http_origin("https://api.example.com\n"), None);
        assert_eq!(parse_http_origin("https://api example.com"), None);
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn authority() -> impl Strategy<Value = String> {
            "[A-Za-z0-9.-]{1,24}(:[0-9]{1,5})?".prop_map(|value| value.to_string())
        }

        fn non_http_scheme() -> impl Strategy<Value = String> {
            "[a-z]{2,12}"
                .prop_map(|value| value.to_string())
                .prop_filter("scheme must not be http or https", |scheme| {
                    scheme != "http" && scheme != "https"
                })
        }

        proptest! {
            #[test]
            fn prop_parses_https_round_trip(authority in authority()) {
                let origin = format!("https://{authority}");
                let parsed = parse_http_origin(origin.as_str()).unwrap();
                prop_assert_eq!(parsed.scheme, HttpOriginScheme::Https);
                prop_assert_eq!(parsed.authority, authority.as_str());
            }

            #[test]
            fn prop_parses_http_round_trip(authority in authority()) {
                let origin = format!("http://{authority}");
                let parsed = parse_http_origin(origin.as_str()).unwrap();
                prop_assert_eq!(parsed.scheme, HttpOriginScheme::Http);
                prop_assert_eq!(parsed.authority, authority.as_str());
            }

            #[test]
            fn prop_rejects_non_http_scheme(scheme in non_http_scheme(), authority in authority()) {
                let origin = format!("{scheme}://{authority}");
                prop_assert_eq!(parse_http_origin(origin.as_str()), None);
            }

            #[test]
            fn prop_rejects_path_suffix(authority in authority(), path in "[A-Za-z0-9/_-]{1,16}") {
                let origin = format!("https://{authority}/{path}");
                prop_assert_eq!(parse_http_origin(origin.as_str()), None);
            }
        }
    }
}
