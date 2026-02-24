//! Platform auth mode parsing and labeling primitives.
//!
//! This crate is intentionally tiny and framework-agnostic.

#![forbid(unsafe_code)]

/// Supported platform auth modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformAuthMode {
    /// No auth checks.
    Open,
    /// Shared bearer-style static token.
    Basic,
    /// JWT validation with shared secret.
    Jwt,
}

impl PlatformAuthMode {
    /// Strict parser for auth mode values.
    pub fn parse_strict(value: &str) -> Result<Self, String> {
        let normalized = value.to_ascii_lowercase();
        match normalized.as_str() {
            "basic" => Ok(Self::Basic),
            "jwt" => Ok(Self::Jwt),
            "none" | "open" => Ok(Self::Open),
            other => {
                Err(format!("Invalid auth mode '{}'. Valid options: basic, jwt, none, open", other))
            }
        }
    }

    /// Lowercase label for status/UI payloads.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Basic => "basic",
            Self::Jwt => "jwt",
        }
    }
}

impl From<&str> for PlatformAuthMode {
    fn from(value: &str) -> Self {
        let normalized = value.to_ascii_lowercase();
        match normalized.as_str() {
            "basic" => Self::Basic,
            "jwt" => Self::Jwt,
            "none" | "open" => Self::Open,
            other => {
                tracing::warn!(
                    "Invalid PLATFORM_AUTH_MODE '{}' falling back to 'open'. Valid options: basic, jwt, none, open",
                    other
                );
                Self::Open
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_strict_valid_values_are_supported() {
        assert_eq!(PlatformAuthMode::parse_strict("basic").unwrap(), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::parse_strict("jwt").unwrap(), PlatformAuthMode::Jwt);
        assert_eq!(PlatformAuthMode::parse_strict("none").unwrap(), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::parse_strict("open").unwrap(), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::parse_strict("BASIC").unwrap(), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::parse_strict("JWT").unwrap(), PlatformAuthMode::Jwt);
    }

    #[test]
    fn parse_strict_rejects_unknown_values() {
        assert!(PlatformAuthMode::parse_strict("invalid").is_err());
        assert!(PlatformAuthMode::parse_strict("bearer").is_err());
        assert!(PlatformAuthMode::parse_strict("").is_err());
    }

    #[test]
    fn fallback_from_str_maps_invalid_to_open() {
        assert_eq!(PlatformAuthMode::from("basic"), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::from("jwt"), PlatformAuthMode::Jwt);
        assert_eq!(PlatformAuthMode::from("none"), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::from("open"), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::from("something-else"), PlatformAuthMode::Open);
    }

    #[test]
    fn labels_are_stable() {
        assert_eq!(PlatformAuthMode::Open.label(), "open");
        assert_eq!(PlatformAuthMode::Basic.label(), "basic");
        assert_eq!(PlatformAuthMode::Jwt.label(), "jwt");
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_parse_strict_case_insensitive(
                raw in prop_oneof![
                    Just("basic".to_string()),
                    Just("jwt".to_string()),
                    Just("open".to_string()),
                    Just("none".to_string()),
                ],
                uppercase in any::<bool>(),
            ) {
                let input = if uppercase { raw.to_ascii_uppercase() } else { raw.clone() };
                let parsed = PlatformAuthMode::parse_strict(&input).unwrap();

                let expected = match raw.as_str() {
                    "basic" => PlatformAuthMode::Basic,
                    "jwt" => PlatformAuthMode::Jwt,
                    "open" | "none" => PlatformAuthMode::Open,
                    _ => unreachable!(),
                };

                prop_assert_eq!(parsed, expected);
            }

            #[test]
            fn prop_label_is_always_lowercase(mode in prop_oneof![
                Just(PlatformAuthMode::Open),
                Just(PlatformAuthMode::Basic),
                Just(PlatformAuthMode::Jwt),
            ]) {
                let label = mode.label();
                prop_assert_eq!(label, label.to_ascii_lowercase());
            }
        }
    }
}
