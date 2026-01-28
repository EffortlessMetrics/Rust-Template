//! Timeline receipt types for tracking PR evolution.
//!
//! The timeline receipt captures temporal topology - how a PR evolved,
//! friction zones, and convergence patterns.

use crate::meta::ReceiptMeta;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Timeline receipt for tracking PR evolution.
///
/// This receipt captures wall clock timestamps, sessions, friction zones,
/// oscillations, convergence patterns, and overall topology.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineReceipt {
    /// Schema version for forward compatibility.
    pub schema_version: String,

    /// PR number, if this receipt is associated with a pull request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u64>,

    /// Run ID for correlation with other receipts.
    pub run_id: String,

    /// Wall clock timestamps for the PR lifecycle.
    pub wall_clock: WallClock,

    /// Burst sessions of activity.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sessions: Vec<Session>,

    /// Files/modules with repeated touches.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub friction_zones: Vec<FrictionZone>,

    /// Add/remove/add signals indicating uncertainty.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub oscillations: Vec<Oscillation>,

    /// How the PR stabilized toward completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convergence: Option<Convergence>,

    /// Overall evolution pattern.
    pub topology: Topology,

    /// Confidence level of the topology classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology_confidence: Option<TimelineConfidence>,

    /// Evidence supporting the topology classification.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub topology_reasons: Vec<String>,

    /// Human intervention events during PR lifecycle.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,

    /// Meta provenance for re-analysis and method versioning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ReceiptMeta>,
}

/// Overall evolution pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Topology {
    /// Steady progress, clean development.
    Linear,
    /// Repeated refinement, controlled iteration.
    Cyclical,
    /// Unclear direction, high churn.
    Chaotic,
}

/// Confidence level for classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimelineConfidence {
    /// Strong evidence supports the classification.
    High,
    /// Reasonable inference.
    Medium,
    /// Limited evidence.
    Low,
}

/// Classification of activity burst.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionClassification {
    /// Rapid automated commits.
    MachineGrind,
    /// Spaced deliberate commits.
    HumanWork,
    /// Mixed pattern.
    Mixed,
}

/// Type of oscillation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OscillationType {
    /// Dependencies added/removed.
    Dependency,
    /// Files created/deleted.
    File,
    /// Feature flags toggled.
    Feature,
    /// Architectural pivot.
    Approach,
}

/// Action in an oscillation sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OscillationAction {
    /// Item was added.
    Add,
    /// Item was removed.
    Remove,
    /// Item was modified.
    Modify,
}

/// Type of human intervention event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Review was requested.
    ReviewRequested,
    /// Changes were requested.
    ChangesRequested,
    /// PR was approved.
    Approved,
    /// Comment was left.
    Comment,
    /// PR description was edited.
    PrEdit,
}

/// Wall clock timestamps for the PR lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallClock {
    /// Timestamp of the first commit.
    pub first_commit: DateTime<Utc>,

    /// Timestamp of the last commit.
    pub last_commit: DateTime<Utc>,

    /// Timestamp when the PR was opened.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_created: Option<DateTime<Utc>>,

    /// Timestamp when the PR was merged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_merged: Option<DateTime<Utc>>,

    /// Total elapsed time in minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration_minutes: Option<u64>,
}

/// A burst session of activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Session start timestamp.
    pub start: DateTime<Utc>,

    /// Session end timestamp.
    pub end: DateTime<Utc>,

    /// Number of commits in this session.
    pub commit_count: u32,

    /// Inferred session type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<SessionClassification>,
}

/// A file or module with repeated touches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrictionZone {
    /// File path or module name.
    pub path: String,

    /// Number of times this path was modified.
    pub touch_count: u32,

    /// Commit SHAs that touched this path.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commits: Vec<String>,
}

/// An add/remove/add pattern indicating uncertainty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Oscillation {
    /// Type of oscillation.
    #[serde(rename = "type")]
    pub oscillation_type: OscillationType,

    /// The thing that oscillated.
    pub subject: String,

    /// Sequence of actions showing oscillation pattern.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sequence: Vec<OscillationAction>,
}

/// Signals of PR stabilization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Convergence {
    /// Commit SHA where stabilization began.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inflection_commit: Option<String>,

    /// Whether final commits show stable changes.
    #[serde(default)]
    pub last_n_commits_stable: bool,

    /// Categories of changes in stable final commits.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stable_categories: Vec<String>,
}

/// A human intervention event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Type of event.
    #[serde(rename = "type")]
    pub event_type: EventType,

    /// When event occurred.
    pub timestamp: DateTime<Utc>,

    /// Who triggered event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,

    /// Brief summary of event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl TimelineReceipt {
    /// Check if PR had high friction.
    pub fn is_high_friction(&self) -> bool {
        !self.friction_zones.is_empty() || !self.oscillations.is_empty()
    }

    /// Get the total number of commits across all sessions.
    pub fn total_commits(&self) -> u32 {
        self.sessions.iter().map(|s| s.commit_count).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_serde() {
        assert_eq!(serde_json::to_string(&Topology::Linear).unwrap(), r#""linear""#);
        assert_eq!(serde_json::to_string(&Topology::Cyclical).unwrap(), r#""cyclical""#);
        assert_eq!(serde_json::to_string(&Topology::Chaotic).unwrap(), r#""chaotic""#);
    }

    #[test]
    fn test_session_classification_serde() {
        assert_eq!(
            serde_json::to_string(&SessionClassification::MachineGrind).unwrap(),
            r#""machine_grind""#
        );
        assert_eq!(
            serde_json::to_string(&SessionClassification::HumanWork).unwrap(),
            r#""human_work""#
        );
        assert_eq!(serde_json::to_string(&SessionClassification::Mixed).unwrap(), r#""mixed""#);
    }

    #[test]
    fn test_is_high_friction() {
        let low_friction = TimelineReceipt {
            schema_version: "1.0".to_string(),
            pr: None,
            run_id: "test".to_string(),
            wall_clock: WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: None,
            },
            sessions: Vec::new(),
            friction_zones: Vec::new(),
            oscillations: Vec::new(),
            convergence: None,
            topology: Topology::Linear,
            topology_confidence: None,
            topology_reasons: Vec::new(),
            events: Vec::new(),
            meta: None,
        };

        assert!(!low_friction.is_high_friction());

        let high_friction = TimelineReceipt {
            schema_version: "1.0".to_string(),
            pr: None,
            run_id: "test".to_string(),
            wall_clock: WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: None,
            },
            sessions: Vec::new(),
            friction_zones: vec![FrictionZone {
                path: "lib.rs".to_string(),
                touch_count: 5,
                commits: vec![],
            }],
            oscillations: Vec::new(),
            convergence: None,
            topology: Topology::Cyclical,
            topology_confidence: None,
            topology_reasons: Vec::new(),
            events: Vec::new(),
            meta: None,
        };

        assert!(high_friction.is_high_friction());
    }

    #[test]
    fn test_total_commits() {
        let receipt = TimelineReceipt {
            schema_version: "1.0".to_string(),
            pr: None,
            run_id: "test".to_string(),
            wall_clock: WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: None,
            },
            sessions: vec![Session {
                start: "2026-01-07T10:00:00Z".parse().unwrap(),
                end: "2026-01-07T12:00:00Z".parse().unwrap(),
                commit_count: 5,
                classification: None,
            }],
            friction_zones: Vec::new(),
            oscillations: Vec::new(),
            convergence: None,
            topology: Topology::Linear,
            topology_confidence: None,
            topology_reasons: Vec::new(),
            events: Vec::new(),
            meta: None,
        };

        assert_eq!(receipt.total_commits(), 5);
    }
}
