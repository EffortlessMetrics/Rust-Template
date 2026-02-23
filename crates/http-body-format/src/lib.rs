//! Request body format classification for HTTP payload parsers.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

/// Supported request body formats used by HTTP payload parsers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyFormat {
    /// `application/json`
    Json,
    /// `application/x-www-form-urlencoded`
    FormUrlEncoded,
    /// Any unsupported or missing media type.
    Unknown,
}

/// Classify request body format from an optional `content-type` header value.
///
/// Classification rules:
/// - Case-insensitive media type matching
/// - Ignores media-type parameters (`; charset=utf-8`)
/// - Trims surrounding ASCII whitespace around media type
pub fn classify_body_format(content_type: Option<&str>) -> BodyFormat {
    let media_type = content_type.unwrap_or_default().split(';').next().unwrap_or_default().trim();

    if media_type.eq_ignore_ascii_case("application/json") {
        BodyFormat::Json
    } else if media_type.eq_ignore_ascii_case("application/x-www-form-urlencoded") {
        BodyFormat::FormUrlEncoded
    } else {
        BodyFormat::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_json_media_type() {
        assert_eq!(classify_body_format(Some("application/json")), BodyFormat::Json);
    }

    #[test]
    fn classifies_form_urlencoded_media_type() {
        assert_eq!(
            classify_body_format(Some("application/x-www-form-urlencoded")),
            BodyFormat::FormUrlEncoded
        );
    }

    #[test]
    fn classifies_case_insensitive_media_type() {
        assert_eq!(classify_body_format(Some("Application/JSON")), BodyFormat::Json);
    }

    #[test]
    fn classifies_media_type_with_parameters() {
        assert_eq!(classify_body_format(Some("application/json; charset=utf-8")), BodyFormat::Json);
    }

    #[test]
    fn classifies_media_type_with_surrounding_whitespace() {
        assert_eq!(
            classify_body_format(Some("\t application/json ; charset=utf-8 \r\n")),
            BodyFormat::Json
        );
    }

    #[test]
    fn classifies_unknown_media_type() {
        assert_eq!(classify_body_format(Some("text/plain")), BodyFormat::Unknown);
    }

    #[test]
    fn classifies_missing_media_type_as_unknown() {
        assert_eq!(classify_body_format(None), BodyFormat::Unknown);
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn random_case(base: &'static str) -> impl Strategy<Value = String> {
            prop::collection::vec(any::<bool>(), base.len()).prop_map(move |flags| {
                base.chars()
                    .zip(flags)
                    .map(
                        |(ch, uppercase)| {
                            if uppercase {
                                ch.to_ascii_uppercase()
                            } else {
                                ch.to_ascii_lowercase()
                            }
                        },
                    )
                    .collect()
            })
        }

        fn unknown_media_type() -> impl Strategy<Value = String> {
            "[A-Za-z0-9.+-]{1,12}/[A-Za-z0-9.+-]{1,20}"
                .prop_map(|value| value.to_string())
                .prop_filter("must not be a supported media type", |value| {
                    !value.eq_ignore_ascii_case("application/json")
                        && !value.eq_ignore_ascii_case("application/x-www-form-urlencoded")
                })
        }

        proptest! {
            #[test]
            fn prop_classifies_json_case_insensitively(
                media_type in random_case("application/json"),
                left_ws in "[ \\t]{0,3}",
                right_ws in "[ \\t]{0,3}",
                param_name in "[A-Za-z0-9_-]{0,8}",
                param_value in "[A-Za-z0-9._-]{0,8}",
            ) {
                let value = format!("{left_ws}{media_type}; {param_name}={param_value}{right_ws}");
                prop_assert_eq!(classify_body_format(Some(value.as_str())), BodyFormat::Json);
            }

            #[test]
            fn prop_classifies_form_case_insensitively(
                media_type in random_case("application/x-www-form-urlencoded"),
                left_ws in "[ \\t]{0,3}",
                right_ws in "[ \\t]{0,3}",
            ) {
                let value = format!("{left_ws}{media_type}{right_ws}");
                prop_assert_eq!(classify_body_format(Some(value.as_str())), BodyFormat::FormUrlEncoded);
            }

            #[test]
            fn prop_rejects_unknown_media_types_as_unknown(media_type in unknown_media_type()) {
                prop_assert_eq!(classify_body_format(Some(media_type.as_str())), BodyFormat::Unknown);
            }
        }
    }
}
