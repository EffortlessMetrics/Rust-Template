use http_body_format::{BodyFormat, classify_body_format};

#[test]
fn classifies_json_with_charset_parameter() {
    assert_eq!(classify_body_format(Some("application/json; charset=utf-8")), BodyFormat::Json);
}

#[test]
fn classifies_form_with_case_insensitive_media_type() {
    assert_eq!(
        classify_body_format(Some("Application/X-WWW-FORM-URLENCODED")),
        BodyFormat::FormUrlEncoded
    );
}

#[test]
fn classifies_unsupported_media_type_as_unknown() {
    assert_eq!(classify_body_format(Some("text/plain")), BodyFormat::Unknown);
}
