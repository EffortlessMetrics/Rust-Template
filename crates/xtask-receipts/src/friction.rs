//! Friction zone analysis for timeline receipts.
//!
//! This module provides utilities for:
//! - Categorizing friction zones by file type
//! - Rolling up friction zones by category
//! - Excluding ephemeral directories from analysis

use gov_receipts::FrictionZone;
use std::collections::HashMap;

/// Friction zone category for rollup reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrictionCategory {
    /// Test code and fixtures
    Tests,
    /// Documentation (docs/, *.md)
    Docs,
    /// Receipt artifacts (.runs/**)
    Receipts,
    /// Configuration files (Cargo.toml, *.yaml, *.toml, .claude/**)
    Config,
    /// Implementation code (src/, crates/, lib/)
    Impl,
    /// CI/CD and infrastructure (.github/, .gitlab-ci.yml, etc.)
    CiInfra,
    /// Specifications and schemas
    Specs,
    /// Other/unclassified
    Other,
}

impl FrictionCategory {
    /// Categorize a file path
    pub fn from_path(path: &str) -> Self {
        // Normalize path separators for consistent matching
        let normalized = path.replace('\\', "/");

        // Receipt artifacts
        if normalized.starts_with(".runs/") || normalized.contains("/receipts/") {
            return Self::Receipts;
        }

        // Test paths
        if normalized.contains("/tests/")
            || normalized.contains("/test_fixtures/")
            || normalized.contains("_test.rs")
            || normalized.ends_with("_tests.rs")
            || normalized.ends_with("/tests.rs")
            || normalized.contains("/benches/")
        {
            return Self::Tests;
        }

        // Core code paths
        if normalized.starts_with("src/")
            || normalized.starts_with("crates/")
            || normalized.starts_with("lib/")
            || normalized.starts_with("core/")
            || normalized.starts_with("packages/")
            || normalized.starts_with("apps/")
        {
            return Self::Impl;
        }

        // Documentation
        if normalized.starts_with("docs/") || normalized.ends_with(".md") {
            return Self::Docs;
        }

        // Config files
        if normalized == "Cargo.toml"
            || normalized.starts_with(".claude/")
            || normalized.starts_with(".config/")
            || normalized.ends_with(".toml")
            || normalized.ends_with(".yaml")
            || normalized.ends_with(".yml")
        {
            return Self::Config;
        }

        // CI/Infrastructure
        if normalized.starts_with(".github/")
            || normalized.starts_with(".gitlab-ci")
            || normalized.starts_with(".circleci/")
            || normalized.starts_with("ci/")
            || normalized == "Dockerfile"
            || normalized.ends_with(".Dockerfile")
            || normalized == "docker-compose.yml"
            || normalized.starts_with("scripts/")
        {
            return Self::CiInfra;
        }

        // Specs and schemas
        if normalized.ends_with(".schema.json") || normalized.starts_with("specs/") {
            return Self::Specs;
        }

        Self::Other
    }

    /// Display name for the category
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tests => "tests",
            Self::Docs => "docs",
            Self::Receipts => "receipts",
            Self::Config => "config",
            Self::Impl => "impl",
            Self::CiInfra => "ci_infra",
            Self::Specs => "specs",
            Self::Other => "other",
        }
    }

    /// Stable categories should not be penalized as friction.
    pub fn is_stable(&self) -> bool {
        matches!(self, Self::Tests | Self::Docs | Self::Receipts | Self::Config)
    }
}

/// Rollup friction zones by category.
pub fn categorize_friction_zones(
    zones: &[FrictionZone],
) -> HashMap<&'static str, Vec<&FrictionZone>> {
    let mut by_category: HashMap<&'static str, Vec<&FrictionZone>> = HashMap::new();

    for zone in zones {
        let category = FrictionCategory::from_path(&zone.path);
        by_category.entry(category.as_str()).or_default().push(zone);
    }

    by_category
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friction_category_tests_detection() {
        assert_eq!(
            FrictionCategory::from_path("crates/foo/tests/integration.rs"),
            FrictionCategory::Tests
        );
        assert_eq!(FrictionCategory::from_path("src/lib_test.rs"), FrictionCategory::Tests);
        assert_eq!(FrictionCategory::from_path("crates/x/src/tests.rs"), FrictionCategory::Tests);
    }

    #[test]
    fn friction_category_docs_detection() {
        assert_eq!(FrictionCategory::from_path("docs/README.md"), FrictionCategory::Docs);
        assert_eq!(FrictionCategory::from_path("CHANGELOG.md"), FrictionCategory::Docs);
        assert_eq!(FrictionCategory::from_path("docs/api/overview.md"), FrictionCategory::Docs);
    }

    #[test]
    fn friction_category_receipts_detection() {
        assert_eq!(
            FrictionCategory::from_path(".runs/current/receipts/gate.json"),
            FrictionCategory::Receipts
        );
        assert_eq!(
            FrictionCategory::from_path(".runs/pr123/receipts/timeline.json"),
            FrictionCategory::Receipts
        );
    }

    #[test]
    fn friction_category_config_detection() {
        assert_eq!(FrictionCategory::from_path("Cargo.toml"), FrictionCategory::Config);
        assert_eq!(FrictionCategory::from_path("specs/config.yaml"), FrictionCategory::Config);
        assert_eq!(FrictionCategory::from_path(".claude/settings.json"), FrictionCategory::Config);
    }

    #[test]
    fn friction_category_impl_detection() {
        assert_eq!(FrictionCategory::from_path("crates/foo/src/lib.rs"), FrictionCategory::Impl);
        assert_eq!(FrictionCategory::from_path("src/main.rs"), FrictionCategory::Impl);
    }

    #[test]
    fn friction_category_is_stable() {
        assert!(FrictionCategory::Tests.is_stable());
        assert!(FrictionCategory::Docs.is_stable());
        assert!(FrictionCategory::Receipts.is_stable());
        assert!(FrictionCategory::Config.is_stable());
        assert!(!FrictionCategory::Impl.is_stable());
    }

    #[test]
    fn friction_category_as_str() {
        assert_eq!(FrictionCategory::Tests.as_str(), "tests");
        assert_eq!(FrictionCategory::Docs.as_str(), "docs");
        assert_eq!(FrictionCategory::Receipts.as_str(), "receipts");
        assert_eq!(FrictionCategory::Config.as_str(), "config");
        assert_eq!(FrictionCategory::Impl.as_str(), "impl");
    }
}
