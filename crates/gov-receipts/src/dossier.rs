//! Dossier types for structured PR analysis.
//!
//! A dossier provides structured analysis of a PR for casebook generation
//! and audit purposes. It captures scope, intent, findings, and quality scores.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Structured PR analysis for casebook generation.
///
/// A dossier captures the complete analysis of a merged PR including
/// scope, intent, findings, errata, and quality scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dossier {
    /// Schema version for forward compatibility.
    pub schema_version: String,

    /// PR number.
    pub pr: u64,

    /// PR title.
    pub title: String,

    /// When the PR was merged.
    pub merged_at: DateTime<Utc>,

    /// Scope of changes.
    pub scope: Scope,

    /// Intent and linkage.
    pub intent: Intent,

    /// Findings from analysis.
    #[serde(default)]
    pub findings: Vec<Finding>,

    /// Errata or corrections.
    #[serde(default)]
    pub errata: Vec<Erratum>,

    /// Quality scores for the PR.
    pub exhibit_score: ExhibitScore,

    /// Changes to the governance factory.
    pub factory_delta: FactoryDelta,
}

/// Scope of changes in the PR.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    /// Top-level directories affected.
    #[serde(default)]
    pub top_dirs: Vec<String>,

    /// Number of files changed.
    #[serde(default)]
    pub files_changed: u32,

    /// Lines added.
    #[serde(default)]
    pub lines_added: u32,

    /// Lines removed.
    #[serde(default)]
    pub lines_removed: u32,
}

/// Intent and linkage for the PR.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intent {
    /// Related issue numbers (e.g., ["#76"]).
    #[serde(default)]
    pub issue_links: Vec<String>,

    /// Related specification IDs (e.g., ["REQ-PLT-ISSUES-001"]).
    #[serde(default)]
    pub spec_links: Vec<String>,

    /// Related acceptance criteria IDs.
    #[serde(default)]
    pub ac_links: Vec<String>,
}

/// A finding from PR analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Finding severity or category.
    pub category: String,

    /// Description of the finding.
    pub description: String,

    /// Files or locations affected.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<String>,
}

/// An erratum or correction for a previous claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Erratum {
    /// What was originally stated or claimed.
    pub original: String,

    /// The correction.
    pub correction: String,

    /// When the correction was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected_at: Option<DateTime<Utc>>,
}

/// Quality scores for the exhibit.
///
/// Scores are on a 0-5 scale where 5 is best.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExhibitScore {
    /// Clarity of scope definition (0-5).
    pub scope_clarity: u8,

    /// Completeness of proof/evidence (0-5).
    pub proof_completeness: u8,

    /// Quality of errata handling (0-5).
    pub errata_quality: u8,

    /// Quality of factory delta documentation (0-5).
    pub factory_delta: u8,

    /// Total score.
    pub total: u8,

    /// Maximum possible score.
    pub max: u8,
}

/// Changes to the governance factory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactoryDelta {
    /// Gates added or enhanced.
    #[serde(default)]
    pub gates_added: Vec<String>,

    /// Contracts tightened or added.
    #[serde(default)]
    pub contracts_tightened: Vec<String>,

    /// Documentation updated.
    #[serde(default)]
    pub docs_updated: Vec<String>,
}

impl Dossier {
    /// Create a new dossier builder.
    pub fn builder() -> DossierBuilder {
        DossierBuilder::default()
    }

    /// Check if the dossier has any findings.
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }

    /// Check if the dossier has any errata.
    pub fn has_errata(&self) -> bool {
        !self.errata.is_empty()
    }

    /// Get the score as a percentage.
    pub fn score_percentage(&self) -> f64 {
        if self.exhibit_score.max == 0 {
            return 0.0;
        }
        f64::from(self.exhibit_score.total) / f64::from(self.exhibit_score.max) * 100.0
    }
}

impl Default for ExhibitScore {
    fn default() -> Self {
        Self {
            scope_clarity: 0,
            proof_completeness: 0,
            errata_quality: 0,
            factory_delta: 0,
            total: 0,
            max: 25,
        }
    }
}

/// Builder for constructing `Dossier` instances.
#[derive(Debug, Default)]
pub struct DossierBuilder {
    schema_version: Option<String>,
    pr: Option<u64>,
    title: Option<String>,
    merged_at: Option<DateTime<Utc>>,
    scope: Scope,
    intent: Intent,
    findings: Vec<Finding>,
    errata: Vec<Erratum>,
    exhibit_score: ExhibitScore,
    factory_delta: FactoryDelta,
}

impl DossierBuilder {
    /// Set the schema version.
    pub fn schema_version(mut self, version: impl Into<String>) -> Self {
        self.schema_version = Some(version.into());
        self
    }

    /// Set the PR number.
    pub fn pr(mut self, pr: u64) -> Self {
        self.pr = Some(pr);
        self
    }

    /// Set the PR title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the merge time.
    pub fn merged_at(mut self, time: DateTime<Utc>) -> Self {
        self.merged_at = Some(time);
        self
    }

    /// Set the scope.
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Set the intent.
    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// Add a finding.
    pub fn finding(mut self, finding: Finding) -> Self {
        self.findings.push(finding);
        self
    }

    /// Set all findings.
    pub fn findings(mut self, findings: Vec<Finding>) -> Self {
        self.findings = findings;
        self
    }

    /// Add an erratum.
    pub fn erratum(mut self, erratum: Erratum) -> Self {
        self.errata.push(erratum);
        self
    }

    /// Set all errata.
    pub fn errata(mut self, errata: Vec<Erratum>) -> Self {
        self.errata = errata;
        self
    }

    /// Set the exhibit score.
    pub fn exhibit_score(mut self, score: ExhibitScore) -> Self {
        self.exhibit_score = score;
        self
    }

    /// Set the factory delta.
    pub fn factory_delta(mut self, delta: FactoryDelta) -> Self {
        self.factory_delta = delta;
        self
    }

    /// Build the dossier.
    ///
    /// # Panics
    ///
    /// Panics if required fields are not set.
    pub fn build(self) -> Dossier {
        Dossier {
            schema_version: self.schema_version.unwrap_or_else(|| "1.0".to_string()),
            pr: self.pr.expect("pr is required"),
            title: self.title.expect("title is required"),
            merged_at: self.merged_at.expect("merged_at is required"),
            scope: self.scope,
            intent: self.intent,
            findings: self.findings,
            errata: self.errata,
            exhibit_score: self.exhibit_score,
            factory_delta: self.factory_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dossier_roundtrip() {
        let dossier = Dossier {
            schema_version: "1.0".to_string(),
            pr: 209,
            title: "Add pagination error contract BDD scenarios".to_string(),
            merged_at: "2026-01-07T15:00:00Z".parse().unwrap(),
            scope: Scope {
                top_dirs: vec!["specs/features".to_string(), "crates/gov-http".to_string()],
                files_changed: 5,
                lines_added: 120,
                lines_removed: 15,
            },
            intent: Intent {
                issue_links: vec!["#76".to_string()],
                spec_links: vec!["REQ-PLT-ISSUES-001".to_string()],
                ac_links: vec!["AC-PLT-ISSUES-PAGINATION".to_string()],
            },
            findings: Vec::new(),
            errata: Vec::new(),
            exhibit_score: ExhibitScore {
                scope_clarity: 5,
                proof_completeness: 5,
                errata_quality: 5,
                factory_delta: 3,
                total: 18,
                max: 25,
            },
            factory_delta: FactoryDelta {
                gates_added: vec!["BDD pagination scenarios".to_string()],
                contracts_tightened: vec!["400 error responses".to_string()],
                docs_updated: Vec::new(),
            },
        };

        let json = serde_json::to_string_pretty(&dossier).unwrap();
        let parsed: Dossier = serde_json::from_str(&json).unwrap();

        assert_eq!(dossier, parsed);
    }

    #[test]
    fn test_dossier_builder() {
        let dossier = Dossier::builder()
            .pr(123)
            .title("Test PR")
            .merged_at("2026-01-07T15:00:00Z".parse().unwrap())
            .scope(Scope { files_changed: 3, ..Default::default() })
            .build();

        assert_eq!(dossier.pr, 123);
        assert_eq!(dossier.title, "Test PR");
        assert_eq!(dossier.scope.files_changed, 3);
    }

    #[test]
    fn test_score_percentage() {
        let dossier = Dossier::builder()
            .pr(123)
            .title("Test PR")
            .merged_at("2026-01-07T15:00:00Z".parse().unwrap())
            .exhibit_score(ExhibitScore { total: 18, max: 25, ..Default::default() })
            .build();

        assert!((dossier.score_percentage() - 72.0).abs() < 0.01);
    }

    #[test]
    fn test_has_findings_and_errata() {
        let empty_dossier = Dossier::builder()
            .pr(123)
            .title("Test PR")
            .merged_at("2026-01-07T15:00:00Z".parse().unwrap())
            .build();

        assert!(!empty_dossier.has_findings());
        assert!(!empty_dossier.has_errata());

        let with_findings = Dossier::builder()
            .pr(123)
            .title("Test PR")
            .merged_at("2026-01-07T15:00:00Z".parse().unwrap())
            .finding(Finding {
                category: "warning".to_string(),
                description: "Test finding".to_string(),
                locations: Vec::new(),
            })
            .build();

        assert!(with_findings.has_findings());

        let with_errata = Dossier::builder()
            .pr(123)
            .title("Test PR")
            .merged_at("2026-01-07T15:00:00Z".parse().unwrap())
            .erratum(Erratum {
                original: "Original claim".to_string(),
                correction: "Corrected claim".to_string(),
                corrected_at: None,
            })
            .build();

        assert!(with_errata.has_errata());
    }

    #[test]
    fn test_empty_vectors_serialization() {
        let dossier = Dossier::builder()
            .pr(123)
            .title("Test PR")
            .merged_at("2026-01-07T15:00:00Z".parse().unwrap())
            .build();

        let json = serde_json::to_string(&dossier).unwrap();

        // Empty findings and errata should be serialized as empty arrays (due to #[serde(default)])
        assert!(json.contains(r#""findings":[]"#));
        assert!(json.contains(r#""errata":[]"#));
    }

    #[test]
    fn test_finding_with_locations() {
        let finding = Finding {
            category: "warning".to_string(),
            description: "Test finding".to_string(),
            locations: vec!["src/lib.rs:42".to_string()],
        };

        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("locations"));

        let empty_finding = Finding {
            category: "info".to_string(),
            description: "Empty test".to_string(),
            locations: Vec::new(),
        };

        let json = serde_json::to_string(&empty_finding).unwrap();
        // Empty locations should be skipped (check for the key, not any substring)
        assert!(!json.contains(r#""locations""#));
    }

    #[test]
    fn test_default_values() {
        let scope = Scope::default();
        assert!(scope.top_dirs.is_empty());
        assert_eq!(scope.files_changed, 0);

        let intent = Intent::default();
        assert!(intent.issue_links.is_empty());
        assert!(intent.spec_links.is_empty());
        assert!(intent.ac_links.is_empty());

        let score = ExhibitScore::default();
        assert_eq!(score.total, 0);
        assert_eq!(score.max, 25);

        let delta = FactoryDelta::default();
        assert!(delta.gates_added.is_empty());
        assert!(delta.contracts_tightened.is_empty());
        assert!(delta.docs_updated.is_empty());
    }
}
