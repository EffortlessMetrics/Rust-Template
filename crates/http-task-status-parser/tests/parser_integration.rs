use gov_model::TaskStatus;
use http_task_status_parser::{ParseUpdateTaskStatusError, parse_update_task_status};

#[test]
fn parses_json_with_charset_parameter() {
    let body = br#"{"status":"InProgress"}"#;

    let parsed = parse_update_task_status(Some("application/json; charset=utf-8"), body).unwrap();
    assert_eq!(parsed.status, TaskStatus::InProgress);
}

#[test]
fn falls_back_to_form_when_content_type_is_not_supported() {
    let parsed = parse_update_task_status(Some("text/plain"), b"status=Done").unwrap();
    assert_eq!(parsed.status, TaskStatus::Done);
}

#[test]
fn parses_form_with_case_insensitive_content_type_and_parameters() {
    let parsed = parse_update_task_status(
        Some(" Application/X-WWW-FORM-URLENCODED ; charset=utf-8 "),
        b"status=Review",
    )
    .unwrap();
    assert_eq!(parsed.status, TaskStatus::Review);
}

#[test]
fn rejects_unknown_format_when_no_supported_decoder_matches() {
    let error = parse_update_task_status(None, b"status:Done").expect_err("expected parse failure");
    assert_eq!(error, ParseUpdateTaskStatusError::UnsupportedBodyFormat);
}
