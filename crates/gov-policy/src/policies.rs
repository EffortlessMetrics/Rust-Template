//! Embedded Rego policy definitions.

/// Spec ledger validation policy.
pub const LEDGER_REGO: &str = include_str!("../policies/ledger.rego");

/// Feature flag validation policy.
pub const FEATURES_REGO: &str = include_str!("../policies/features.rego");

/// Configuration flags policy.
pub const FLAGS_REGO: &str = include_str!("../policies/flags.rego");

/// Privacy compliance policy.
pub const PRIVACY_REGO: &str = include_str!("../policies/privacy.rego");

/// Template core structure policy.
pub const TEMPLATE_CORE_REGO: &str = include_str!("../policies/template_core.rego");

/// LLM context validation policy.
pub const LLM_REGO: &str = include_str!("../policies/llm.rego");

/// Kubernetes security policy.
pub const K8S_REGO: &str = include_str!("../policies/k8s.rego");
