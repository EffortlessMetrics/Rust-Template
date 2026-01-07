//! Timeline receipt types for tracking PR evolution.
//!
//! The timeline receipt captures temporal topology - how a PR evolved,
//! friction zones, and convergence patterns.

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

    /// Sequence of actions showing the oscillation pattern.
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

    /// When the event occurred.
    pub timestamp: DateTime<Utc>,

    /// Who triggered the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,

    /// Brief summary of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl TimelineReceipt {
    /// Create a new timeline receipt builder.
    pub fn builder() -> TimelineReceiptBuilder {
        TimelineReceiptBuilder::default()
    }

    /// Check if the PR had high friction.
    pub fn is_high_friction(&self) -> bool {
        !self.friction_zones.is_empty() || !self.oscillations.is_empty()
    }

    /// Get the total number of commits across all sessions.
    pub fn total_commits(&self) -> u32 {
        self.sessions.iter().map(|s| s.commit_count).sum()
    }
}

/// Builder for constructing `TimelineReceipt` instances.
#[derive(Debug, Default)]
pub struct TimelineReceiptBuilder {
    schema_version: Option<String>,
    pr: Option<u64>,
    run_id: Option<String>,
    wall_clock: Option<WallClock>,
    sessions: Vec<Session>,
    friction_zones: Vec<FrictionZone>,
    oscillations: Vec<Oscillation>,
    convergence: Option<Convergence>,
    topology: Option<Topology>,
    topology_confidence: Option<TimelineConfidence>,
    topology_reasons: Vec<String>,
    events: Vec<Event>,
}

impl TimelineReceiptBuilder {
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

    /// Set the run ID.
    pub fn run_id(mut self, id: impl Into<String>) -> Self {
        self.run_id = Some(id.into());
        self
    }

    /// Set the wall clock.
    pub fn wall_clock(mut self, clock: WallClock) -> Self {
        self.wall_clock = Some(clock);
        self
    }

    /// Add a session.
    pub fn session(mut self, session: Session) -> Self {
        self.sessions.push(session);
        self
    }

    /// Set all sessions.
    pub fn sessions(mut self, sessions: Vec<Session>) -> Self {
        self.sessions = sessions;
        self
    }

    /// Add a friction zone.
    pub fn friction_zone(mut self, zone: FrictionZone) -> Self {
        self.friction_zones.push(zone);
        self
    }

    /// Add an oscillation.
    pub fn oscillation(mut self, osc: Oscillation) -> Self {
        self.oscillations.push(osc);
        self
    }

    /// Set the convergence.
    pub fn convergence(mut self, conv: Convergence) -> Self {
        self.convergence = Some(conv);
        self
    }

    /// Set the topology.
    pub fn topology(mut self, topo: Topology) -> Self {
        self.topology = Some(topo);
        self
    }

    /// Set the topology confidence.
    pub fn topology_confidence(mut self, conf: TimelineConfidence) -> Self {
        self.topology_confidence = Some(conf);
        self
    }

    /// Add a topology reason.
    pub fn topology_reason(mut self, reason: impl Into<String>) -> Self {
        self.topology_reasons.push(reason.into());
        self
    }

    /// Add an event.
    pub fn event(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    /// Build the timeline receipt.
    ///
    /// # Panics
    ///
    /// Panics if run_id, wall_clock, or topology is not set.
    pub fn build(self) -> TimelineReceipt {
        TimelineReceipt {
            schema_version: self.schema_version.unwrap_or_else(|| "1.0".to_string()),
            pr: self.pr,
            run_id: self.run_id.expect("run_id is required"),
            wall_clock: self.wall_clock.expect("wall_clock is required"),
            sessions: self.sessions,
            friction_zones: self.friction_zones,
            oscillations: self.oscillations,
            convergence: self.convergence,
            topology: self.topology.expect("topology is required"),
            topology_confidence: self.topology_confidence,
            topology_reasons: self.topology_reasons,
            events: self.events,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_receipt_roundtrip() {
        let receipt = TimelineReceipt {
            schema_version: "1.0".to_string(),
            pr: Some(123),
            run_id: "test-run".to_string(),
            wall_clock: WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: Some(240),
            },
            sessions: vec![Session {
                start: "2026-01-07T10:00:00Z".parse().unwrap(),
                end: "2026-01-07T12:00:00Z".parse().unwrap(),
                commit_count: 5,
                classification: Some(SessionClassification::HumanWork),
            }],
            friction_zones: vec![FrictionZone {
                path: "src/lib.rs".to_string(),
                touch_count: 3,
                commits: vec!["abc123".to_string()],
            }],
            oscillations: vec![],
            convergence: Some(Convergence {
                last_n_commits_stable: true,
                stable_categories: vec!["tests".to_string()],
                ..Default::default()
            }),
            topology: Topology::Linear,
            topology_confidence: Some(TimelineConfidence::High),
            topology_reasons: vec!["Clean progression".to_string()],
            events: vec![],
        };

        let json = serde_json::to_string_pretty(&receipt).unwrap();
        let parsed: TimelineReceipt = serde_json::from_str(&json).unwrap();

        assert_eq!(receipt, parsed);
    }

    #[test]
    fn test_timeline_receipt_builder() {
        let receipt = TimelineReceipt::builder()
            .run_id("test-run")
            .pr(123)
            .wall_clock(WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: None,
            })
            .topology(Topology::Linear)
            .session(Session {
                start: "2026-01-07T10:00:00Z".parse().unwrap(),
                end: "2026-01-07T12:00:00Z".parse().unwrap(),
                commit_count: 5,
                classification: None,
            })
            .build();

        assert_eq!(receipt.pr, Some(123));
        assert_eq!(receipt.topology, Topology::Linear);
        assert_eq!(receipt.sessions.len(), 1);
        assert_eq!(receipt.total_commits(), 5);
    }

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
        let low_friction = TimelineReceipt::builder()
            .run_id("test")
            .wall_clock(WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: None,
            })
            .topology(Topology::Linear)
            .build();

        assert!(!low_friction.is_high_friction());

        let high_friction = TimelineReceipt::builder()
            .run_id("test")
            .wall_clock(WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: None,
            })
            .topology(Topology::Cyclical)
            .friction_zone(FrictionZone {
                path: "lib.rs".to_string(),
                touch_count: 5,
                commits: vec![],
            })
            .build();

        assert!(high_friction.is_high_friction());
    }
}
