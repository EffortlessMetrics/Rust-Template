//! Parser for `Authorization: Bearer <token>` header values.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

/// Extract bearer token from a single `Authorization` header value.
///
/// Accepts case-insensitive `Bearer` scheme followed by a single space.
pub fn extract_bearer_token(value: &str) -> Option<&str> {
    let (scheme, token) = value.split_once(' ')?;
    if scheme.eq_ignore_ascii_case("bearer") { Some(token) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_bearer_token() {
        assert_eq!(extract_bearer_token("Bearer abc.def.ghi"), Some("abc.def.ghi"));
    }

    #[test]
    fn accepts_case_insensitive_scheme() {
        assert_eq!(extract_bearer_token("bEaReR token"), Some("token"));
    }

    #[test]
    fn returns_none_for_non_bearer_scheme() {
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn returns_none_when_separator_missing() {
        assert_eq!(extract_bearer_token("Bearer"), None);
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn token() -> impl Strategy<Value = String> {
            "[A-Za-z0-9._~:-]{0,128}".prop_map(|value| value.to_string())
        }

        fn non_bearer_scheme() -> impl Strategy<Value = String> {
            "[A-Za-z]{1,12}"
                .prop_map(|value| value.to_string())
                .prop_filter("scheme must not be bearer", |scheme| {
                    !scheme.eq_ignore_ascii_case("bearer")
                })
        }

        proptest! {
            #[test]
            fn prop_round_trips_token(token in token()) {
                let value = format!("Bearer {token}");
                let parsed = extract_bearer_token(value.as_str());
                prop_assert_eq!(parsed, Some(token.as_str()));
            }

            #[test]
            fn prop_rejects_non_bearer_schemes(scheme in non_bearer_scheme(), token in token()) {
                let value = format!("{scheme} {token}");
                let parsed = extract_bearer_token(value.as_str());
                prop_assert_eq!(parsed, None);
            }
        }
    }
}
