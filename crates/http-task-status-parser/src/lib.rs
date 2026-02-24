//! Parser for task status update payloads.
//!
//! This crate is intentionally small and framework-agnostic. It accepts
//! `content-type` and raw bytes, then parses an `UpdateTaskStatusRequest`
//! from either JSON or form-urlencoded payloads.

#![forbid(unsafe_code)]

use gov_model::TaskStatus;
use http_body_format::{BodyFormat, classify_body_format};
use serde::Deserialize;
use std::fmt;

/// Parsed request body for task status updates.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdateTaskStatusRequest {
    /// Canonical task status to transition to.
    pub status: TaskStatus,
}

/// Error returned when parsing a task status update payload fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseUpdateTaskStatusError {
    /// Body was declared as JSON but JSON deserialization failed.
    InvalidJson(String),
    /// Body was declared as form-urlencoded but form deserialization failed.
    InvalidFormData(String),
    /// Body could not be parsed as either supported format.
    UnsupportedBodyFormat,
}

impl fmt::Display for ParseUpdateTaskStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(f, "Invalid JSON: {message}"),
            Self::InvalidFormData(message) => write!(f, "Invalid form data: {message}"),
            Self::UnsupportedBodyFormat => {
                write!(f, "Unsupported body format; use JSON or x-www-form-urlencoded")
            }
        }
    }
}

impl std::error::Error for ParseUpdateTaskStatusError {}

/// Parse a task status update body from content-type and raw bytes.
///
/// Supported media types:
/// - `application/json`
/// - `application/x-www-form-urlencoded`
///
/// If content-type is absent or unsupported, this function falls back to
/// trying JSON first and form second to remain permissive for callers.
pub fn parse_update_task_status(
    content_type: Option<&str>,
    body: &[u8],
) -> Result<UpdateTaskStatusRequest, ParseUpdateTaskStatusError> {
    match classify_body_format(content_type) {
        BodyFormat::Json => parse_json(body),
        BodyFormat::FormUrlEncoded => parse_form(body),
        BodyFormat::Unknown => parse_fallback(body),
    }
}

fn parse_json(body: &[u8]) -> Result<UpdateTaskStatusRequest, ParseUpdateTaskStatusError> {
    serde_json::from_slice(body)
        .map_err(|error| ParseUpdateTaskStatusError::InvalidJson(error.to_string()))
}

fn parse_form(body: &[u8]) -> Result<UpdateTaskStatusRequest, ParseUpdateTaskStatusError> {
    serde_urlencoded::from_bytes(trim_ascii_whitespace(body))
        .map_err(|error| ParseUpdateTaskStatusError::InvalidFormData(error.to_string()))
}

fn parse_fallback(body: &[u8]) -> Result<UpdateTaskStatusRequest, ParseUpdateTaskStatusError> {
    if let Ok(value) = serde_json::from_slice::<UpdateTaskStatusRequest>(body) {
        return Ok(value);
    }

    if let Ok(value) =
        serde_urlencoded::from_bytes::<UpdateTaskStatusRequest>(trim_ascii_whitespace(body))
    {
        return Ok(value);
    }

    Err(ParseUpdateTaskStatusError::UnsupportedBodyFormat)
}

fn trim_ascii_whitespace(body: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = body.len();

    while start < end && body[start].is_ascii_whitespace() {
        start += 1;
    }

    while end > start && body[end - 1].is_ascii_whitespace() {
        end -= 1;
    }

    &body[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use gov_model::TaskStatus;

    #[test]
    fn parses_json_when_content_type_is_json() {
        let body = br#"{"status":"InProgress"}"#;

        let parsed = parse_update_task_status(Some("application/json"), body).unwrap();
        assert_eq!(parsed.status, TaskStatus::InProgress);
    }

    #[test]
    fn parses_form_when_content_type_is_form() {
        let parsed =
            parse_update_task_status(Some("application/x-www-form-urlencoded"), b"status=Done")
                .unwrap();

        assert_eq!(parsed.status, TaskStatus::Done);
    }

    #[test]
    fn parses_json_without_content_type_via_fallback() {
        let parsed = parse_update_task_status(None, br#"{"status":"Review"}"#).unwrap();
        assert_eq!(parsed.status, TaskStatus::Review);
    }

    #[test]
    fn parses_form_without_content_type_via_fallback() {
        let parsed = parse_update_task_status(None, b"status=Todo").unwrap();
        assert_eq!(parsed.status, TaskStatus::Todo);
    }

    #[test]
    fn parses_form_with_surrounding_whitespace() {
        let parsed = parse_update_task_status(
            Some("application/x-www-form-urlencoded"),
            b"\n  status=InProgress \r\n",
        )
        .unwrap();
        assert_eq!(parsed.status, TaskStatus::InProgress);
    }

    #[test]
    fn parses_media_type_with_parameters() {
        let body = br#"{"status":"InProgress"}"#;

        let parsed =
            parse_update_task_status(Some("application/json; charset=utf-8"), body).unwrap();
        assert_eq!(parsed.status, TaskStatus::InProgress);
    }

    #[test]
    fn handles_case_insensitive_media_type() {
        let body = br#"{"status":"InProgress"}"#;

        let parsed = parse_update_task_status(Some("Application/JSON"), body).unwrap();
        assert_eq!(parsed.status, TaskStatus::InProgress);
    }

    #[test]
    fn returns_specific_error_for_invalid_json_when_declared_json() {
        let error = parse_update_task_status(Some("application/json"), b"{not-json")
            .expect_err("expected invalid json");

        assert!(matches!(error, ParseUpdateTaskStatusError::InvalidJson(_)));
        assert!(error.to_string().contains("Invalid JSON"));
    }

    #[test]
    fn returns_specific_error_for_invalid_form_when_declared_form() {
        let error =
            parse_update_task_status(Some("application/x-www-form-urlencoded"), b"status=%ZZ")
                .expect_err("expected invalid form");

        assert!(matches!(error, ParseUpdateTaskStatusError::InvalidFormData(_)));
        assert!(error.to_string().contains("Invalid form data"));
    }

    #[test]
    fn returns_unsupported_error_when_body_matches_no_supported_format() {
        let error = parse_update_task_status(Some("text/plain"), b"status:InProgress")
            .expect_err("expected unsupported format");

        assert_eq!(error, ParseUpdateTaskStatusError::UnsupportedBodyFormat);
        assert_eq!(error.to_string(), "Unsupported body format; use JSON or x-www-form-urlencoded");
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn canonical_status() -> impl Strategy<Value = (&'static str, TaskStatus)> {
            prop_oneof![
                Just(("Todo", TaskStatus::Todo)),
                Just(("InProgress", TaskStatus::InProgress)),
                Just(("Review", TaskStatus::Review)),
                Just(("Done", TaskStatus::Done)),
            ]
        }

        proptest! {
            #[test]
            fn prop_json_roundtrip(status in canonical_status()) {
                let (raw, expected) = status;
                let body = format!(r#"{{"status":"{raw}"}}"#);

                let parsed = parse_update_task_status(Some("application/json"), body.as_bytes()).unwrap();
                prop_assert_eq!(parsed.status, expected);
            }

            #[test]
            fn prop_form_roundtrip(status in canonical_status()) {
                let (raw, expected) = status;
                let body = format!("status={raw}");

                let parsed = parse_update_task_status(Some("application/x-www-form-urlencoded"), body.as_bytes()).unwrap();
                prop_assert_eq!(parsed.status, expected);
            }

            #[test]
            fn prop_fallback_accepts_valid_json(status in canonical_status()) {
                let (raw, expected) = status;
                let body = format!(r#"{{"status":"{raw}"}}"#);

                let parsed = parse_update_task_status(Some("text/plain"), body.as_bytes()).unwrap();
                prop_assert_eq!(parsed.status, expected);
            }
        }
    }
}
