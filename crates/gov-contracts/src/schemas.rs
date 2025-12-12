//! Embedded schema definitions.

/// Platform schema (IDP integration contract).
pub const PLATFORM_SCHEMA: &str = include_str!("../schemas/platform_schema.yaml");

/// UI contract schema.
pub const UI_CONTRACT_SCHEMA: &str = include_str!("../schemas/ui_contract.yaml");

/// Config schema template.
pub const CONFIG_SCHEMA: &str = include_str!("../schemas/config_schema.yaml");
