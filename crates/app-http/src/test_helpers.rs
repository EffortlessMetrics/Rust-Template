//! Test helper functions to reduce panic!() usage
//!
//! This module provides utilities for writing more reliable tests that return
//! Result types instead of panicking, improving test robustness
//! and providing better error messages.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Parse a string into a specific type with context
pub fn parse_with_context<T: std::str::FromStr>(
    input: &str,
    context: &str,
) -> Result<T> 
where 
    T::Err: std::fmt::Display,
{
    input.parse::<T>()
        .with_context(|| format!("Failed to parse {}: '{}'", context, input))
}

/// Extract a field from JSON value with context
pub fn get_field<'a>(
    json: &'a Value,
    field: &str,
) -> Result<&'a Value> {
    json.get(field)
        .ok_or_else(|| anyhow!("Missing field: {}", field))
}

/// Extract a string field from JSON value with context
pub fn get_string_field<'a>(
    json: &'a Value,
    field: &str,
) -> Result<&'a str> {
    get_field(json, field)?
        .as_str()
        .ok_or_else(|| anyhow!("Field '{}' is not a string", field))
}

/// Extract an array field from JSON value with context
pub fn get_array_field<'a>(
    json: &'a Value,
    field: &str,
) -> Result<&'a Vec<Value>> {
    get_field(json, field)?
        .as_array()
        .ok_or_else(|| anyhow!("Field '{}' is not an array", field))
}

/// Assert that a string contains another string
pub fn assert_contains(actual: &str, expected: &str) -> Result<()> {
    if actual.contains(expected) {
        Ok(())
    } else {
        anyhow::bail!(
            "Expected '{}' to contain '{}', but it didn't. Actual: '{}'",
            expected, actual, actual
        )
    }
}

/// Assert that two values are equal with context
pub fn assert_eq_with_context<T: PartialEq + std::fmt::Debug>(
    actual: T,
    expected: T,
    context: &str,
) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        anyhow::bail!(
            "{}: expected {:?}, got {:?}",
            context, expected, actual
        )
    }
}

/// Assert that a value is not empty with context
pub fn assert_not_empty(value: &str, context: &str) -> Result<()> {
    if !value.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("{}: expected non-empty value, got '{}'", context, value)
    }
}

/// Assert that a status code indicates success
pub fn assert_success_status(status: u16) -> Result<()> {
    if (200..300).contains(&status) {
        Ok(())
    } else {
        anyhow::bail!("Expected success status (2xx), got {}", status)
    }
}

/// Create a test error for when a condition should be true
pub fn assert_condition(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        anyhow::bail!("{}", message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_context_success() {
        let result: Result<i32> = parse_with_context("123", "test number");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
    }

    #[test]
    fn test_parse_with_context_failure() {
        let result: Result<i32> = parse_with_context("invalid", "test number");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse test number"));
    }

    #[test]
    fn test_get_field_success() {
        let json = serde_json::json!({
            "name": "test",
            "value": 42
        });
        
        let name = get_string_field(&json, "name").unwrap();
        assert_eq!(name, "test");
    }

    #[test]
    fn test_get_field_failure() {
        let json = serde_json::json!({
            "name": "test"
        });
        
        let result = get_string_field(&json, "missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing field: missing"));
    }

    #[test]
    fn test_assert_contains_success() {
        assert_contains("hello world", "world").unwrap();
    }

    #[test]
    fn test_assert_contains_failure() {
        let result = assert_contains("hello", "world");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected 'hello' to contain 'world'"));
    }
}