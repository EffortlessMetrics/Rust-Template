//! Scanner-safe fake secrets for test code.
//!
//! This module provides deterministic, low-entropy fake secrets that won't
//! trigger secret scanners (GitHub, gitleaks, etc.). Use these generators
//! instead of hardcoding fake secrets throughout test files.
//!
//! # Design Principles
//!
//! - **Low entropy**: Human-obvious, no random UUIDs/hashes/base64
//! - **No vendor patterns**: Never use `sk_live_`, `AKIA`, `ghp_`, `xoxb-`, etc.
//! - **Deterministic**: Same input produces same output (no randomness)
//! - **Clearly fake**: `EXAMPLE_*_DO_NOT_USE` pattern is unambiguous
//!
//! # Usage
//!
//! ```
//! use testing::fakes::{example_secret, example_token};
//!
//! // Basic secret generation
//! let secret = example_secret("database_password");
//! assert_eq!(secret, "EXAMPLE_DATABASE_PASSWORD_DO_NOT_USE");
//!
//! // Token with ID suffix for uniqueness
//! let token = example_token("api_key");
//! assert_eq!(token, "EXAMPLE_TOKEN_API_KEY_0000");
//! ```

/// Generate a scanner-safe fake secret.
///
/// Returns `EXAMPLE_{LABEL}_DO_NOT_USE` where label is uppercased
/// and hyphens are replaced with underscores.
///
/// # Example
///
/// ```
/// use testing::fakes::example_secret;
///
/// assert_eq!(
///     example_secret("jwt-secret"),
///     "EXAMPLE_JWT_SECRET_DO_NOT_USE"
/// );
/// ```
#[must_use]
pub fn example_secret(label: &str) -> String {
    format!("EXAMPLE_{}_DO_NOT_USE", label.to_ascii_uppercase().replace('-', "_"))
}

/// Generate a scanner-safe fake token.
///
/// Returns `EXAMPLE_TOKEN_{LABEL}_0000` where label is uppercased
/// and hyphens are replaced with underscores.
///
/// # Example
///
/// ```
/// use testing::fakes::example_token;
///
/// assert_eq!(
///     example_token("platform-auth"),
///     "EXAMPLE_TOKEN_PLATFORM_AUTH_0000"
/// );
/// ```
#[must_use]
pub fn example_token(label: &str) -> String {
    format!("EXAMPLE_TOKEN_{}_0000", label.to_ascii_uppercase().replace('-', "_"))
}

/// Generate a scanner-safe fake secret with a unique identifier.
///
/// Use this when you need multiple distinct secrets of the same type
/// (e.g., in parameterized tests).
///
/// Returns `EXAMPLE_{LABEL}_{ID}_DO_NOT_USE` where both label and id
/// are uppercased and hyphens are replaced with underscores.
///
/// # Example
///
/// ```
/// use testing::fakes::example_secret_for;
///
/// assert_eq!(
///     example_secret_for("api-key", "test-1"),
///     "EXAMPLE_API_KEY_TEST_1_DO_NOT_USE"
/// );
/// assert_eq!(
///     example_secret_for("api-key", "test-2"),
///     "EXAMPLE_API_KEY_TEST_2_DO_NOT_USE"
/// );
/// ```
#[must_use]
pub fn example_secret_for(label: &str, id: &str) -> String {
    format!(
        "EXAMPLE_{}_{}_DO_NOT_USE",
        label.to_ascii_uppercase().replace('-', "_"),
        id.to_ascii_uppercase().replace('-', "_"),
    )
}

/// Generate a scanner-safe fake API key.
///
/// Returns `EXAMPLE_APIKEY_{LABEL}_00000000` where label is uppercased.
/// The suffix provides a "key-like" appearance without triggering scanners.
///
/// # Example
///
/// ```
/// use testing::fakes::example_api_key;
///
/// assert_eq!(
///     example_api_key("stripe"),
///     "EXAMPLE_APIKEY_STRIPE_00000000"
/// );
/// ```
#[must_use]
pub fn example_api_key(label: &str) -> String {
    format!("EXAMPLE_APIKEY_{}_00000000", label.to_ascii_uppercase().replace('-', "_"))
}

/// Generate a scanner-safe fake database URL.
///
/// Returns a URL-like string that won't trigger secret scanners.
/// Uses `example.internal` domain and clearly fake credentials.
///
/// # Example
///
/// ```
/// use testing::fakes::example_database_url;
///
/// assert_eq!(
///     example_database_url("postgres"),
///     "postgresql://EXAMPLE_USER:EXAMPLE_PASS@example.internal:5432/EXAMPLE_DB"
/// );
/// ```
#[must_use]
pub fn example_database_url(db_type: &str) -> String {
    let scheme = match db_type.to_lowercase().as_str() {
        "postgres" | "postgresql" => "postgresql",
        "mysql" => "mysql",
        "redis" => "redis",
        _ => db_type,
    };
    format!("{scheme}://EXAMPLE_USER:EXAMPLE_PASS@example.internal:5432/EXAMPLE_DB")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_secret_formats_correctly() {
        assert_eq!(example_secret("test"), "EXAMPLE_TEST_DO_NOT_USE");
        assert_eq!(example_secret("jwt-secret"), "EXAMPLE_JWT_SECRET_DO_NOT_USE");
        assert_eq!(example_secret("ALREADY_UPPER"), "EXAMPLE_ALREADY_UPPER_DO_NOT_USE");
    }

    #[test]
    fn example_token_formats_correctly() {
        assert_eq!(example_token("auth"), "EXAMPLE_TOKEN_AUTH_0000");
        assert_eq!(example_token("platform-token"), "EXAMPLE_TOKEN_PLATFORM_TOKEN_0000");
    }

    #[test]
    fn example_secret_for_includes_id() {
        assert_eq!(example_secret_for("key", "1"), "EXAMPLE_KEY_1_DO_NOT_USE");
        assert_eq!(
            example_secret_for("api-key", "test-case"),
            "EXAMPLE_API_KEY_TEST_CASE_DO_NOT_USE"
        );
    }

    #[test]
    fn example_api_key_formats_correctly() {
        assert_eq!(example_api_key("service"), "EXAMPLE_APIKEY_SERVICE_00000000");
    }

    #[test]
    fn example_database_url_recognizes_types() {
        assert!(example_database_url("postgres").starts_with("postgresql://"));
        assert!(example_database_url("mysql").starts_with("mysql://"));
        assert!(example_database_url("redis").starts_with("redis://"));
        assert!(example_database_url("custom").starts_with("custom://"));
    }

    #[test]
    fn no_scanner_triggering_patterns() {
        // Verify none of our outputs contain vendor-specific patterns
        let outputs = [
            example_secret("test"),
            example_token("test"),
            example_secret_for("test", "1"),
            example_api_key("test"),
            example_database_url("postgres"),
        ];

        let banned_patterns =
            ["sk_live_", "sk_test_", "AKIA", "ghp_", "gho_", "github_pat_", "xoxb-", "xoxp-"];

        for output in &outputs {
            for pattern in &banned_patterns {
                assert!(
                    !output.contains(pattern),
                    "Output '{output}' contains banned pattern '{pattern}'"
                );
            }
        }
    }
}
