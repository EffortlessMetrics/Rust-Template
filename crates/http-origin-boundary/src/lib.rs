//! Boundary predicate primitives for CORS origin rule matching.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

/// Returns `true` when `value` ends with `suffix` and the suffix is preceded
/// by a `.` label boundary.
///
/// Examples:
/// - `api.example.com` has a label boundary before `example.com`
/// - `notexample.com` does not have a label boundary before `example.com`
pub fn has_label_boundary_before_suffix(value: &str, suffix: &str) -> bool {
    if suffix.is_empty() {
        return false;
    }

    if value.len() <= suffix.len() {
        return false;
    }

    if !value.ends_with(suffix) {
        return false;
    }

    value.as_bytes().get(value.len() - suffix.len() - 1).is_some_and(|byte| *byte == b'.')
}

/// Returns `true` when a prefix match remainder is empty or starts with `/`.
///
/// This is used to enforce that `https://example.com/*` only matches:
/// - `https://example.com`
/// - `https://example.com/...`
///
/// and rejects boundaryless authority suffixes such as
/// `https://example.com.evil/...`.
pub fn has_path_boundary_or_empty(remainder: &str) -> bool {
    remainder.is_empty() || remainder.starts_with('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_boundary_matches_subdomain() {
        assert!(has_label_boundary_before_suffix("api.example.com", "example.com"));
    }

    #[test]
    fn label_boundary_rejects_boundaryless_suffix() {
        assert!(!has_label_boundary_before_suffix("notexample.com", "example.com"));
    }

    #[test]
    fn label_boundary_rejects_equal_length_values() {
        assert!(!has_label_boundary_before_suffix("example.com", "example.com"));
    }

    #[test]
    fn label_boundary_rejects_empty_suffix() {
        assert!(!has_label_boundary_before_suffix("api.example.com", ""));
    }

    #[test]
    fn path_boundary_accepts_empty_remainder() {
        assert!(has_path_boundary_or_empty(""));
    }

    #[test]
    fn path_boundary_accepts_slash_prefixed_remainder() {
        assert!(has_path_boundary_or_empty("/admin"));
    }

    #[test]
    fn path_boundary_rejects_non_slash_remainder() {
        assert!(!has_path_boundary_or_empty(".evil/path"));
        assert!(!has_path_boundary_or_empty("?debug=true"));
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

        fn path_segment() -> impl Strategy<Value = String> {
            "[A-Za-z0-9._~-]{1,16}".prop_map(|value| value.to_string())
        }

        proptest! {
            #[test]
            fn prop_label_boundary_matches_subdomain(domain in domain(), subdomain in label()) {
                let value = format!("{subdomain}.{domain}");
                prop_assert!(has_label_boundary_before_suffix(value.as_str(), domain.as_str()));
            }

            #[test]
            fn prop_label_boundary_rejects_boundaryless_suffix(domain in domain(), prefix in label()) {
                let value = format!("{prefix}{domain}");
                prop_assert!(!has_label_boundary_before_suffix(value.as_str(), domain.as_str()));
            }

            #[test]
            fn prop_path_boundary_accepts_only_empty_or_slash_prefixed(path in path_segment()) {
                let slash_prefixed = format!("/{path}");
                let query_like = format!("?{path}");

                prop_assert!(has_path_boundary_or_empty(""));
                prop_assert!(has_path_boundary_or_empty(slash_prefixed.as_str()));
                prop_assert!(!has_path_boundary_or_empty(path.as_str()));
                prop_assert!(!has_path_boundary_or_empty(query_like.as_str()));
            }
        }
    }
}
