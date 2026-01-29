//! Input validation logic for governance artifacts and identifiers.

use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

/// Errors that occur during input validation.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Identifier format is invalid.
    #[error("Invalid {kind} ID format: '{id}'. Expected pattern: {expected}")]
    InvalidFormat { kind: String, id: String, expected: String },

    /// Value exceeds maximum length.
    #[error("Field '{field}' too long: {actual} > {max} characters")]
    TooLong { field: String, actual: usize, max: usize },

    /// Field is empty or only whitespace.
    #[error("Field '{0}' cannot be empty")]
    EmptyField(String),

    /// Invalid characters in string.
    #[error("Field '{field}' contains invalid characters")]
    InvalidCharacters { field: String },

    /// Value exceeds maximum YAML depth.
    #[error("YAML depth exceeds maximum limit of {max}")]
    TooDeep { max: usize },
}

static TASK_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^TASK-[A-Z0-9]+-[0-9]{3}$").unwrap());
static AC_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^AC-[A-Z0-9]+-[0-9]{3}$").unwrap());
static QUESTION_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^Q-[A-Z0-9]+-[0-9]{3}$").unwrap());
static FRICTION_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^FRICTION-[A-Z0-9]+-[0-9]{3}$").unwrap());
static FORK_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^FORK-[A-Z0-9]+-[0-9]{3}$").unwrap());
static REQ_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^REQ-[A-Z0-9]+-[A-Z0-9-]+$").unwrap());

/// Validate a task ID format (e.g., TASK-CORE-001).
pub fn validate_task_id(id: &str) -> Result<(), ValidationError> {
    if !TASK_ID_REGEX.is_match(id) {
        return Err(ValidationError::InvalidFormat {
            kind: "Task".to_string(),
            id: id.to_string(),
            expected: "TASK-[COMPONENT]-[NUMBER]".to_string(),
        });
    }
    Ok(())
}

/// Validate an Acceptance Criterion ID format (e.g., AC-TPL-001).
pub fn validate_ac_id(id: &str) -> Result<(), ValidationError> {
    if !AC_ID_REGEX.is_match(id) {
        return Err(ValidationError::InvalidFormat {
            kind: "AC".to_string(),
            id: id.to_string(),
            expected: "AC-[COMPONENT]-[NUMBER]".to_string(),
        });
    }
    Ok(())
}

/// Validate a Question ID format (e.g., Q-FLOW-001).
pub fn validate_question_id(id: &str) -> Result<(), ValidationError> {
    if !QUESTION_ID_REGEX.is_match(id) {
        return Err(ValidationError::InvalidFormat {
            kind: "Question".to_string(),
            id: id.to_string(),
            expected: "Q-[COMPONENT]-[NUMBER]".to_string(),
        });
    }
    Ok(())
}

/// Validate a Friction ID format (e.g., FRICTION-DEV-001).
pub fn validate_friction_id(id: &str) -> Result<(), ValidationError> {
    if !FRICTION_ID_REGEX.is_match(id) {
        return Err(ValidationError::InvalidFormat {
            kind: "Friction".to_string(),
            id: id.to_string(),
            expected: "FRICTION-[COMPONENT]-[NUMBER]".to_string(),
        });
    }
    Ok(())
}

/// Validate a Fork ID format (e.g., FORK-NAME-001).
pub fn validate_fork_id(id: &str) -> Result<(), ValidationError> {
    if !FORK_ID_REGEX.is_match(id) {
        return Err(ValidationError::InvalidFormat {
            kind: "Fork".to_string(),
            id: id.to_string(),
            expected: "FORK-[NAME]-[NUMBER]".to_string(),
        });
    }
    Ok(())
}

/// Validate a Requirement ID format (e.g., REQ-TPL-HEALTH).
pub fn validate_req_id(id: &str) -> Result<(), ValidationError> {
    if !REQ_ID_REGEX.is_match(id) {
        return Err(ValidationError::InvalidFormat {
            kind: "Requirement".to_string(),
            id: id.to_string(),
            expected: "REQ-[COMPONENT]-[NAME]".to_string(),
        });
    }
    Ok(())
}

/// Validate a project or service name.
pub fn validate_name(field: &str, name: &str) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = 100;
    if name.trim().is_empty() {
        return Err(ValidationError::EmptyField(field.to_string()));
    }
    if name.len() > MAX_LENGTH {
        return Err(ValidationError::TooLong {
            field: field.to_string(),
            actual: name.len(),
            max: MAX_LENGTH,
        });
    }
    Ok(())
}

/// Validate YAML depth to prevent stack overflow (deep nesting attacks).
pub fn validate_yaml_depth(
    value: &serde_yaml::Value,
    max_depth: usize,
) -> Result<(), ValidationError> {
    check_depth(value, 0, max_depth)
}

fn check_depth(
    value: &serde_yaml::Value,
    current_depth: usize,
    max_depth: usize,
) -> Result<(), ValidationError> {
    if current_depth > max_depth {
        return Err(ValidationError::TooDeep { max: max_depth });
    }

    match value {
        serde_yaml::Value::Mapping(map) => {
            for (_, v) in map {
                check_depth(v, current_depth + 1, max_depth)?;
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for v in seq {
                check_depth(v, current_depth + 1, max_depth)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Sanitize a string by removing control characters.
pub fn sanitize_string(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t').collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_task_id() {
        assert!(validate_task_id("TASK-CORE-001").is_ok());
        assert!(validate_task_id("TASK-MYSERV-123").is_ok());
        assert!(validate_task_id("invalid").is_err());
        assert!(validate_task_id("TASK-123").is_err());
        assert!(validate_task_id("TASK-CORE-1").is_err());
    }

    #[test]
    fn test_validate_ac_id() {
        assert!(validate_ac_id("AC-TPL-001").is_ok());
        assert!(validate_ac_id("AC-CONFIG-999").is_ok());
        assert!(validate_ac_id("AC-123").is_err());
    }

    #[test]
    fn test_validate_req_id() {
        assert!(validate_req_id("REQ-TPL-HEALTH").is_ok());
        assert!(validate_req_id("REQ-CORE-DB-CONN").is_ok());
        assert!(validate_req_id("REQ-123").is_err());
    }
}
