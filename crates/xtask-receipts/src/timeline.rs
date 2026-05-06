//! Timeline analysis for receipt generation.
//!
//! This module provides utilities for:
//! - Analyzing commit history for development patterns
//! - Detecting friction zones (files touched multiple times)
//! - Detecting oscillations (add/remove/add patterns)
//! - Classifying development topology
//! - Identifying convergence points

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use gov_receipts::{
    Convergence, FrictionZone, MetaConfidence, Oscillation, OscillationAction, OscillationType,
    ReceiptMeta, Session, SessionClassification, TimelineConfidence, TimelineReceipt, Topology,
    WallClock,
};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{categorize_friction_zones, generate_run_id, get_current_commit_short};

// ============================================================================
// Default Exclude Patterns for Timeline Analysis
// ============================================================================

/// Paths to exclude from friction zone and oscillation analysis.
/// These are typically generated/ephemeral directories that pollute timeline signals.
pub const FRICTION_EXCLUDE_PATTERNS: &[&str] =
    &[".runs/", "target/", ".git/", "node_modules/", "vendor/", "dist/", "build/", ".cache/"];

/// Normalize Windows path separators to forward slashes.
/// This ensures path matching works consistently across platforms.
pub fn normalize_path_separators(path: &str) -> String {
    path.replace('\\', "/")
}

/// Check if a path should be excluded from friction/oscillation analysis.
/// Normalizes path separators first for cross-platform compatibility.
///
/// # Arguments
/// * `path` - The file path to check
/// * `extra_excludes` - Additional prefix patterns to exclude
/// * `include_ephemeral` - If true, skip default exclusions (for debugging)
pub fn should_exclude_path(path: &str, extra_excludes: &[String], include_ephemeral: bool) -> bool {
    let normalized = normalize_path_separators(path);

    // Check default exclusions unless include_ephemeral is set
    if !include_ephemeral
        && FRICTION_EXCLUDE_PATTERNS.iter().any(|pattern| normalized.starts_with(pattern))
    {
        return true;
    }

    // Check additional exclusions
    extra_excludes.iter().any(|prefix| {
        let normalized_prefix = normalize_path_separators(prefix);
        normalized.starts_with(&normalized_prefix)
    })
}

// ============================================================================
// TIMELINE RECEIPT
// ============================================================================

/// Arguments for the receipts-timeline command
#[derive(Debug, Clone)]
pub struct ReceiptsTimelineArgs {
    /// Pull request number (optional)
    pub pr: Option<u32>,
    /// Output directory for receipts
    pub output_dir: PathBuf,
    /// Base branch for comparison (default: origin/main)
    pub base_branch: String,
    /// Session gap threshold in minutes (default: 30)
    pub session_gap_minutes: u32,
    /// Shared run_id (for forensic orchestration); generated if None
    pub run_id: Option<String>,
    /// Additional path prefixes to exclude from friction/oscillation analysis
    pub exclude_prefixes: Vec<String>,
    /// Include ephemeral directories in analysis (for debugging)
    pub include_ephemeral: bool,
}

impl Default for ReceiptsTimelineArgs {
    fn default() -> Self {
        Self {
            pr: None,
            output_dir: PathBuf::from(".runs/current"),
            base_branch: "origin/main".to_string(),
            session_gap_minutes: 30,
            run_id: None,
            exclude_prefixes: Vec::new(),
            include_ephemeral: false,
        }
    }
}

/// Generate timeline.json receipt from commit history
pub fn run_timeline(args: ReceiptsTimelineArgs) -> Result<()> {
    println!("{}", "Generating timeline receipt...".blue().bold());

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create {}", args.output_dir.display()))?;

    let receipts_dir = args.output_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    // Use provided run_id (for forensic orchestration) or generate new one
    let run_id = args.run_id.clone().unwrap_or_else(|| generate_run_id(args.pr));

    // Get commit history
    let commits = get_commit_history(&args.base_branch);

    if commits.is_empty() {
        anyhow::bail!("No commits found between HEAD and {}", args.base_branch);
    }

    // Build wall clock from commits
    let first_commit = commits.last().unwrap(); // oldest
    let last_commit = commits.first().unwrap(); // newest

    let wall_clock = WallClock {
        first_commit: first_commit.timestamp,
        last_commit: last_commit.timestamp,
        pr_created: None,
        pr_merged: None,
        total_duration_minutes: Some(
            ((last_commit.timestamp - first_commit.timestamp).num_minutes().max(0)) as u64,
        ),
    };

    // Identify sessions (clusters of commits within gap threshold)
    let sessions = identify_sessions(&commits, args.session_gap_minutes);

    // Find friction zones (files touched multiple times)
    let friction_zones =
        find_friction_zones(&args.base_branch, &args.exclude_prefixes, args.include_ephemeral);

    // Detect oscillations (add/remove/add patterns indicating uncertainty)
    let oscillations =
        detect_oscillations(&args.base_branch, &args.exclude_prefixes, args.include_ephemeral);

    // Detect convergence (how the PR stabilized toward completion)
    let convergence = detect_convergence(&args.base_branch, &commits);

    // Classify topology
    let (topology, confidence, reasons) = classify_topology(&commits, &friction_zones, &sessions);

    let mut builder = TimelineReceipt::builder()
        .run_id(&run_id)
        .wall_clock(wall_clock)
        .topology(topology)
        .topology_confidence(confidence)
        .sessions(sessions);

    for zone in friction_zones {
        builder = builder.friction_zone(zone);
    }

    for osc in oscillations.iter().cloned() {
        builder = builder.oscillation(osc);
    }

    for reason in reasons {
        builder = builder.topology_reason(reason);
    }

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
    }

    if let Some(conv) = convergence {
        builder = builder.convergence(conv);
    }

    // Build meta provenance - timeline uses medium confidence by default
    // as it relies on git history analysis
    let mut meta_builder = ReceiptMeta::builder()
        .method_id("timeline-v1")
        .method_version(1)
        .analysis_run_id(&run_id)
        .input("git_log")
        .input("commit_history")
        .assumption(format!("session gap threshold: {} minutes", args.session_gap_minutes))
        .confidence(MetaConfidence::Medium);

    if let Some(commit) = get_current_commit_short() {
        meta_builder = meta_builder.evidence(commit);
    }

    // Add evidence pointers for commits analyzed
    for commit in commits.iter().take(5) {
        meta_builder = meta_builder.evidence(&commit.sha[..7.min(commit.sha.len())]);
    }

    builder = builder.meta(meta_builder.build());

    let receipt = builder.build();

    // Write receipt
    let timeline_path = receipts_dir.join("timeline.json");
    let json = serde_json::to_string_pretty(&receipt)?;
    std::fs::write(&timeline_path, &json)?;

    println!();
    println!("{} Receipt written to {}", "OK".green(), timeline_path.display());
    println!("  Commits: {}", commits.len());
    println!("  Sessions: {}", receipt.sessions.len());
    println!("  Friction zones: {}", receipt.friction_zones.len());

    // Show category rollup for friction zones
    if !receipt.friction_zones.is_empty() {
        let by_category = categorize_friction_zones(&receipt.friction_zones);
        let mut categories: Vec<_> = by_category.iter().collect();
        categories.sort_by_key(|(_, zones)| std::cmp::Reverse(zones.len()));
        print!("    Categories: ");
        for (i, (cat, zones)) in categories.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}={}", cat, zones.len());
        }
        println!();
    }

    println!("  Oscillations: {}", receipt.oscillations.len());
    println!("  Topology: {:?} ({:?} confidence)", receipt.topology, receipt.topology_confidence);
    if let Some(ref conv) = receipt.convergence {
        println!(
            "  Convergence: stable={}, categories={:?}",
            conv.last_n_commits_stable, conv.stable_categories
        );
    }

    Ok(())
}

/// Diff statistics.
pub struct DiffStat {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

/// Get diff statistics from git.
pub fn get_diff_stat(base_branch: &str) -> DiffStat {
    let output = std::process::Command::new("git").args(["diff", "--stat", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_diff_stat(&stdout)
        }
        Err(_) => DiffStat { files_changed: 0, insertions: 0, deletions: 0 },
    }
}

/// Parse git diff --stat output.
fn parse_diff_stat(output: &str) -> DiffStat {
    let mut files_changed = 0u32;
    let mut insertions = 0u32;
    let mut deletions = 0u32;

    // Look for summary line like "5 files changed, 100 insertions(+), 50 deletions(-)"
    for line in output.lines() {
        if line.contains("changed") && (line.contains("insertion") || line.contains("deletion")) {
            let parts: Vec<&str> = line.split(',').collect();
            for part in parts {
                let part = part.trim();
                if part.contains("file")
                    && let Some(num) = part.split_whitespace().next()
                {
                    files_changed = num.parse().unwrap_or(0);
                } else if part.contains("insertion")
                    && let Some(num) = part.split_whitespace().next()
                {
                    insertions = num.parse().unwrap_or(0);
                } else if part.contains("deletion")
                    && let Some(num) = part.split_whitespace().next()
                {
                    deletions = num.parse().unwrap_or(0);
                }
            }
        }
    }

    DiffStat { files_changed, insertions, deletions }
}

/// Count distinct modules (top-level directories) touched.
pub fn count_modules_touched(base_branch: &str) -> u32 {
    let output =
        std::process::Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let modules: std::collections::HashSet<_> =
                stdout.lines().filter_map(|line| line.split('/').next()).collect();
            modules.len() as u32
        }
        Err(_) => 0,
    }
}

/// Get module names touched.
pub fn get_modules_touched_names(base_branch: &str) -> Vec<String> {
    let output =
        std::process::Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let modules: std::collections::HashSet<_> = stdout
                .lines()
                .filter_map(|line| line.split('/').next())
                .map(|s| s.to_string())
                .collect();
            modules.into_iter().collect()
        }
        Err(_) => vec![],
    }
}

/// Get crate names touched.
pub fn get_crates_touched(base_branch: &str) -> Vec<String> {
    let output =
        std::process::Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let crates: std::collections::HashSet<_> = stdout
                .lines()
                .filter(|line| line.starts_with("crates/"))
                .filter_map(|line| line.split('/').nth(1))
                .map(|s| s.to_string())
                .collect();
            crates.into_iter().collect()
        }
        Err(_) => vec![],
    }
}

/// Find files with high churn (touched multiple times).
pub fn find_hotspots(base_branch: &str) -> Vec<String> {
    // For v0, we just return files changed - hotspot detection needs commit history analysis
    let output =
        std::process::Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter(|line| line.ends_with(".rs"))
                .take(5) // Top 5 files
                .map(|s| s.to_string())
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Count unsafe blocks added/removed.
pub fn count_unsafe_delta(base_branch: &str) -> gov_receipts::UnsafeDelta {
    let output = std::process::Command::new("git").args(["diff", "-U0", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut added = 0u32;
            let mut removed = 0u32;

            for line in stdout.lines() {
                if line.starts_with('+') && !line.starts_with("+++") && line.contains("unsafe") {
                    added += 1;
                } else if line.starts_with('-')
                    && !line.starts_with("---")
                    && line.contains("unsafe")
                {
                    removed += 1;
                }
            }

            gov_receipts::UnsafeDelta { added, removed }
        }
        Err(_) => gov_receipts::UnsafeDelta::default(),
    }
}

/// Count lines of code changes split by test vs impl.
pub fn count_loc_changes(base_branch: &str) -> (u32, u32) {
    let output =
        std::process::Command::new("git").args(["diff", "--numstat", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut tests_loc = 0u32;
            let mut impl_loc = 0u32;

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let added: u32 = parts[0].parse().unwrap_or(0);
                    let file = parts[2];

                    // Categorize by file path
                    if file.contains("/tests/")
                        || file.contains("_test.rs")
                        || file.ends_with("/mod.rs") && file.contains("tests")
                    {
                        tests_loc += added;
                    } else if file.ends_with(".rs") {
                        // Check if file has #[cfg(test)] or #[test] markers
                        // For simplicity, assume files in src/ are impl, tests/ are tests
                        if file.contains("src/") {
                            impl_loc += added;
                        } else {
                            tests_loc += added;
                        }
                    }
                }
            }

            (tests_loc, impl_loc)
        }
        Err(_) => (0, 0),
    }
}

/// Detect contract changes from git diff.
pub fn detect_contract_changes(base_branch: &str) -> gov_receipts::Contracts {
    let output =
        std::process::Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let files: Vec<&str> = stdout.lines().collect();

            let schema_changed =
                files.iter().any(|f| f.ends_with(".schema.json") || f.contains("specs/schemas/"));

            let cli_changed =
                files.iter().any(|f| f.contains("xtask/src/main.rs") || f.contains("cli/"));

            // Public API detection is complex - for v0, check if lib.rs changed
            let public_api_changed =
                files.iter().any(|f| f.ends_with("lib.rs") && f.contains("crates/"));

            gov_receipts::Contracts {
                schema_changed,
                public_api_changed,
                cli_changed,
                breaking: false, // Would need semver analysis
                diff_pointers: vec![],
            }
        }
        Err(_) => gov_receipts::Contracts::default(),
    }
}

/// Commit information.
pub struct CommitInfo {
    pub sha: String,
    pub timestamp: DateTime<Utc>,
    #[expect(dead_code, reason = "existing reviewed debt; tracked by lint policy ratchet")]
    pub author: String,
}

/// File category for convergence detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileCategory {
    /// Test files: **/tests/**, *_test.rs
    Tests,
    /// Documentation: docs/**, *.md
    Docs,
    /// Receipt files: .runs/**
    Receipts,
    /// Config files: *.toml, *.yaml, *.json in root or specs/
    Config,
    /// Implementation files
    Impl,
}

impl FileCategory {
    /// Categorize a file path.
    pub fn from_path(path: &str) -> Self {
        // Tests
        if path.contains("/tests/")
            || path.ends_with("_test.rs")
            || path.contains("tests.rs")
            || (path.contains("/mod.rs") && path.contains("tests"))
        {
            return FileCategory::Tests;
        }

        // Docs
        if path.starts_with("docs/") || path.ends_with(".md") {
            return FileCategory::Docs;
        }

        // Receipts
        if path.starts_with(".runs/") {
            return FileCategory::Receipts;
        }

        // Config
        if (path.ends_with(".toml") || path.ends_with(".yaml") || path.ends_with(".json"))
            && (path.starts_with("specs/")
                || !path.contains('/')
                || path.starts_with(".claude/")
                || path.starts_with(".llm/"))
        {
            return FileCategory::Config;
        }

        FileCategory::Impl
    }

    /// Check if this category is "stable" (non-implementation).
    pub fn is_stable(&self) -> bool {
        matches!(
            self,
            FileCategory::Tests
                | FileCategory::Docs
                | FileCategory::Receipts
                | FileCategory::Config
        )
    }

    /// Convert to string representation for receipt output.
    pub fn as_str(&self) -> &'static str {
        match self {
            FileCategory::Tests => "tests",
            FileCategory::Docs => "docs",
            FileCategory::Receipts => "receipts",
            FileCategory::Config => "config",
            FileCategory::Impl => "impl",
        }
    }
}

/// Get commit history between HEAD and base branch.
pub fn get_commit_history(base_branch: &str) -> Vec<CommitInfo> {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%H|%aI|%an", &format!("{}..HEAD", base_branch)])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        let timestamp = chrono::DateTime::parse_from_rfc3339(parts[1])
                            .ok()?
                            .with_timezone(&Utc);
                        Some(CommitInfo {
                            sha: parts[0].to_string(),
                            timestamp,
                            author: parts[2].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Get files changed in a specific commit.
pub fn get_files_changed_in_commit(sha: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", sha])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().map(|s| s.to_string()).collect()
        }
        Err(_) => vec![],
    }
}

/// Commit change summary for convergence analysis.
pub struct CommitChangeSummary {
    pub sha: String,
    pub file_count: usize,
    pub categories: std::collections::HashSet<FileCategory>,
    pub is_stable: bool,
}

/// Detect convergence in commit history.
///
/// Convergence detection looks for:
/// 1. An inflection point where churn collapsed (last commit touching many impl files)
/// 2. Whether final N commits only touch stable categories (tests, docs, receipts, config)
/// 3. What stable categories final commits touched
pub fn detect_convergence(_base_branch: &str, commits: &[CommitInfo]) -> Option<Convergence> {
    if commits.is_empty() {
        return None;
    }

    // Number of final commits to check for stability
    const STABLE_WINDOW: usize = 3;

    // Analyze each commit's file changes
    // Commits are ordered newest-first from get_commit_history
    let mut summaries: Vec<CommitChangeSummary> = Vec::with_capacity(commits.len());

    for commit in commits {
        let files = get_files_changed_in_commit(&commit.sha);
        let categories: std::collections::HashSet<FileCategory> =
            files.iter().map(|f| FileCategory::from_path(f)).collect();

        let is_stable = !categories.is_empty() && categories.iter().all(|c| c.is_stable());

        summaries.push(CommitChangeSummary {
            sha: commit.sha.clone(),
            file_count: files.len(),
            categories,
            is_stable,
        });
    }

    if summaries.is_empty() {
        return None;
    }

    // Check if last N commits are stable
    let window_size = std::cmp::min(STABLE_WINDOW, summaries.len());
    let final_commits = &summaries[..window_size];
    let last_n_commits_stable = final_commits.iter().all(|s| s.is_stable);

    // Collect stable categories from final commits
    let mut stable_categories_set: std::collections::HashSet<&str> =
        std::collections::HashSet::new();
    if last_n_commits_stable {
        for summary in final_commits {
            for cat in &summary.categories {
                if cat.is_stable() {
                    stable_categories_set.insert(cat.as_str());
                }
            }
        }
    }

    let stable_categories: Vec<String> =
        stable_categories_set.into_iter().map(String::from).collect();

    // Find inflection point: last commit that touched impl files before stabilization
    let mut inflection_commit: Option<String> = None;

    // If all commits are stable, no inflection point
    if !summaries.iter().all(|s| s.is_stable) {
        // Find newest commit that touched impl
        // Then inflection is commit right before stable tail started

        // First, find where stable tail begins (from newest)
        let stable_tail_start =
            summaries.iter().position(|s| !s.is_stable).unwrap_or(summaries.len());

        // If there's a stable tail and impl commits before it, inflection is first impl
        // commit
        if stable_tail_start > 0 && stable_tail_start < summaries.len() {
            // The inflection is commit at stable_tail_start (first non-stable from newest)
            inflection_commit = Some(summaries[stable_tail_start].sha.clone());
        } else if stable_tail_start == 0 {
            // No stable tail at all - look for where file count dropped
            // Find commit with most files as a potential inflection
            if let Some((idx, _)) = summaries
                .iter()
                .enumerate()
                .filter(|(_, s)| s.file_count > 0)
                .max_by_key(|(_, s)| s.file_count)
            {
                // The inflection is where exploration peaked
                inflection_commit = Some(summaries[idx].sha.clone());
            }
        }
    }

    // Only return convergence if there's meaningful data
    if inflection_commit.is_some() || last_n_commits_stable || !stable_categories.is_empty() {
        Some(Convergence { inflection_commit, last_n_commits_stable, stable_categories })
    } else {
        None
    }
}

/// Identify sessions from commit history.
pub fn identify_sessions(commits: &[CommitInfo], gap_minutes: u32) -> Vec<Session> {
    if commits.is_empty() {
        return vec![];
    }

    let mut sessions = Vec::new();
    let mut session_start = commits.last().unwrap().timestamp; // oldest first
    let mut session_end = session_start;
    let mut commit_count = 0u32;

    // Process commits from oldest to newest
    for commit in commits.iter().rev() {
        let gap = (commit.timestamp - session_end).num_minutes();

        if gap > gap_minutes as i64 && commit_count > 0 {
            // End current session, start new one
            sessions.push(Session {
                start: session_start,
                end: session_end,
                commit_count,
                classification: classify_session(
                    commit_count,
                    (session_end - session_start).num_minutes(),
                ),
            });
            session_start = commit.timestamp;
            commit_count = 0;
        }

        session_end = commit.timestamp;
        commit_count += 1;
    }

    // Don't forget last session
    if commit_count > 0 {
        sessions.push(Session {
            start: session_start,
            end: session_end,
            commit_count,
            classification: classify_session(
                commit_count,
                (session_end - session_start).num_minutes(),
            ),
        });
    }

    sessions
}

/// Classify a session based on commit frequency.
pub fn classify_session(commit_count: u32, duration_minutes: i64) -> Option<SessionClassification> {
    if duration_minutes <= 0 {
        return Some(SessionClassification::Mixed);
    }

    let commits_per_hour = (commit_count as f64 / duration_minutes as f64) * 60.0;

    if commits_per_hour > 10.0 {
        Some(SessionClassification::MachineGrind)
    } else if commits_per_hour < 2.0 {
        Some(SessionClassification::HumanWork)
    } else {
        Some(SessionClassification::Mixed)
    }
}

/// Find friction zones (files touched in multiple commits).
///
/// # Arguments
/// * `base_branch` - Base branch for comparison
/// * `extra_excludes` - Additional path prefixes to exclude
/// * `include_ephemeral` - If true, include ephemeral directories (for debugging)
pub fn find_friction_zones(
    base_branch: &str,
    extra_excludes: &[String],
    include_ephemeral: bool,
) -> Vec<FrictionZone> {
    let output = std::process::Command::new("git")
        .args(["log", "--name-only", "--pretty=format:%H", &format!("{}..HEAD", base_branch)])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut file_commits: HashMap<String, Vec<String>> = HashMap::new();
            let mut current_commit = String::new();

            for line in stdout.lines() {
                if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
                    current_commit = line.to_string();
                } else if !line.is_empty() && !current_commit.is_empty() {
                    file_commits.entry(line.to_string()).or_default().push(current_commit.clone());
                }
            }

            // Filter to files touched 2+ times, excluding ephemeral directories
            file_commits
                .into_iter()
                .filter(|(path, commits)| {
                    !should_exclude_path(path, extra_excludes, include_ephemeral)
                        && commits.len() >= 2
                })
                .map(|(path, commits)| FrictionZone {
                    path,
                    touch_count: commits.len() as u32,
                    commits: commits.into_iter().take(5).collect(), // Limit to 5 commits
                })
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Classify overall development topology.
pub fn classify_topology(
    commits: &[CommitInfo],
    friction_zones: &[FrictionZone],
    sessions: &[Session],
) -> (Topology, TimelineConfidence, Vec<String>) {
    let mut reasons = Vec::new();
    let commit_count = commits.len();
    let friction_count = friction_zones.len();
    let high_friction_files = friction_zones.iter().filter(|z| z.touch_count >= 3).count();

    // Heuristics for topology classification
    let has_machine_sessions =
        sessions.iter().any(|s| s.classification == Some(SessionClassification::MachineGrind));

    if friction_count == 0 && commit_count <= 5 {
        reasons.push("Clean progression with minimal commits".to_string());
        return (Topology::Linear, TimelineConfidence::High, reasons);
    }

    if high_friction_files >= 3 || friction_count > commit_count / 2 {
        reasons.push(format!("{} high-friction files detected", high_friction_files));
        reasons.push("Multiple files touched repeatedly across commits".to_string());
        return (Topology::Chaotic, TimelineConfidence::Medium, reasons);
    }

    if friction_count > 0 || has_machine_sessions {
        reasons.push(format!(
            "{} friction zones, {} machine sessions",
            friction_count,
            sessions
                .iter()
                .filter(|s| s.classification == Some(SessionClassification::MachineGrind))
                .count()
        ));
        return (Topology::Cyclical, TimelineConfidence::Medium, reasons);
    }

    reasons.push("Steady commit progression".to_string());
    (Topology::Linear, TimelineConfidence::Low, reasons)
}

/// Represents an action on a subject (file or dependency) in a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectAction {
    Add,
    Remove,
}

/// Track per-commit changes for a subject.
#[derive(Debug, Clone)]
pub struct SubjectHistory {
    pub actions: Vec<(String, SubjectAction)>, // (commit_sha, action)
}

impl SubjectHistory {
    /// Create a new SubjectHistory.
    pub fn new() -> Self {
        Self { actions: Vec::new() }
    }

    /// Push a new action to the history.
    pub fn push(&mut self, commit: String, action: SubjectAction) {
        self.actions.push((commit, action));
    }

    /// Check if this history shows an oscillation pattern (Add -> Remove or Remove -> Add -> Remove, etc.).
    /// Returns sequence of actions if it's an oscillation (2+ alternating actions).
    pub fn to_oscillation_sequence(&self) -> Option<Vec<OscillationAction>> {
        if self.actions.len() < 2 {
            return None;
        }

        // Convert to oscillation actions and check for alternation
        let mut sequence = Vec::new();
        let mut prev_action: Option<&SubjectAction> = None;
        let mut has_alternation = false;

        for (_, action) in &self.actions {
            let osc_action = match action {
                SubjectAction::Add => OscillationAction::Add,
                SubjectAction::Remove => OscillationAction::Remove,
            };

            // Check if this alternates from previous
            if let Some(prev) = prev_action
                && prev != action
            {
                has_alternation = true;
            }

            sequence.push(osc_action);
            prev_action = Some(action);
        }

        if has_alternation { Some(sequence) } else { None }
    }
}

impl Default for SubjectHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect oscillations in commit history between base_branch and HEAD.
///
/// This function detects:
/// - Dependency oscillations: deps added then removed (or vice versa)
/// - File oscillations: files created then deleted (or vice versa)
///
/// # Arguments
/// * `base_branch` - Base branch for comparison
/// * `extra_excludes` - Additional path prefixes to exclude from file oscillations
/// * `include_ephemeral` - If true, include ephemeral directories (for debugging)
pub fn detect_oscillations(
    base_branch: &str,
    extra_excludes: &[String],
    include_ephemeral: bool,
) -> Vec<Oscillation> {
    let mut oscillations = Vec::new();

    // Get commit list from oldest to newest
    let commits = get_commit_shas_oldest_first(base_branch);
    if commits.len() < 2 {
        return oscillations;
    }

    // Detect dependency oscillations
    let dep_oscillations = detect_dependency_oscillations(base_branch, &commits);
    oscillations.extend(dep_oscillations);

    // Detect file oscillations
    let file_oscillations =
        detect_file_oscillations(base_branch, &commits, extra_excludes, include_ephemeral);
    oscillations.extend(file_oscillations);

    oscillations
}

/// Get commit SHAs from oldest to newest.
pub fn get_commit_shas_oldest_first(base_branch: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%H", "--reverse", &format!("{}..HEAD", base_branch)])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().map(|s| s.to_string()).collect()
        }
        Err(_) => vec![],
    }
}

/// Detect dependency oscillations by parsing Cargo.toml changes across commits.
fn detect_dependency_oscillations(base_branch: &str, commits: &[String]) -> Vec<Oscillation> {
    let mut dep_history: HashMap<String, SubjectHistory> = HashMap::new();

    // For each commit, get diff of Cargo.toml files and track dependency changes
    for (i, commit) in commits.iter().enumerate() {
        let parent = if i == 0 { base_branch.to_string() } else { commits[i - 1].clone() };

        // Get diff for Cargo.toml files in this commit
        let output = std::process::Command::new("git")
            .args(["diff", "-U0", &parent, commit, "--", "**/Cargo.toml", "Cargo.toml"])
            .output();

        if let Ok(out) = output {
            let diff = String::from_utf8_lossy(&out.stdout);
            parse_cargo_toml_diff(&diff, commit, &mut dep_history);
        }
    }

    // Convert histories to oscillations
    dep_history
        .into_iter()
        .filter_map(|(dep_name, history)| {
            history.to_oscillation_sequence().map(|sequence| Oscillation {
                oscillation_type: OscillationType::Dependency,
                subject: dep_name,
                sequence,
            })
        })
        .collect()
}

/// Parse a Cargo.toml diff to extract dependency additions/removals.
fn parse_cargo_toml_diff(diff: &str, commit: &str, history: &mut HashMap<String, SubjectHistory>) {
    // We're looking for lines like:
    // +serde = "1.0"
    // -serde = "1.0"
    // +serde = { version = "1.0", features = ["derive"] }
    // -serde = { version = "1.0", features = ["derive"] }

    let mut in_dependencies_section = false;

    for line in diff.lines() {
        // Track if we're entering a dependencies section
        if line.contains("[dependencies]")
            || line.contains("[dev-dependencies]")
            || line.contains("[build-dependencies]")
            || line.contains(".dependencies]")
        {
            in_dependencies_section = true;
            continue;
        }

        // Exit dependency section on new section
        if line.starts_with('+') || line.starts_with('-') {
            let content = &line[1..];
            if content.starts_with('[') && !content.contains("dependencies") {
                in_dependencies_section = false;
                continue;
            }
        }

        // Skip non-diff lines and header lines
        if !line.starts_with('+') && !line.starts_with('-') {
            continue;
        }
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }

        // Parse dependency line
        let is_add = line.starts_with('+');
        let content = &line[1..].trim();

        // Skip empty lines and section headers
        if content.is_empty() || content.starts_with('[') {
            continue;
        }

        // Extract dependency name - it's part before '=' or '.'
        // e.g., "serde = ..." or "serde.workspace = true"
        if let Some(dep_name) = extract_dependency_name(content) {
            // Only track if we think we're in a dependencies section
            // or line looks like a dependency declaration
            if in_dependencies_section || looks_like_dependency_line(content) {
                let entry = history.entry(dep_name).or_default();
                let action = if is_add { SubjectAction::Add } else { SubjectAction::Remove };
                entry.push(commit.to_string(), action);
            }
        }
    }
}

/// Extract dependency name from a Cargo.toml dependency line.
fn extract_dependency_name(line: &str) -> Option<String> {
    // Handle "dep = ..." or "dep.feature = ..."
    let line = line.trim();

    // Skip if this doesn't look like a dep line
    if !line.contains('=') {
        return None;
    }

    // Get part before '='
    let before_eq = line.split('=').next()?.trim();

    // Handle "dep.workspace" -> "dep"
    let name = before_eq.split('.').next()?.trim();

    // Validate it looks like a crate name (alphanumeric, _, -)
    if name.is_empty() {
        return None;
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return None;
    }

    // Skip common non-dependency keys
    let skip_keys = [
        "version",
        "edition",
        "name",
        "authors",
        "description",
        "license",
        "repository",
        "readme",
        "keywords",
        "categories",
        "workspace",
        "package",
        "path",
        "features",
        "default-features",
        "optional",
        "default",
        "members",
        "exclude",
        "include",
        "resolver",
        "rust-version",
    ];
    if skip_keys.contains(&name) {
        return None;
    }

    Some(name.to_string())
}

/// Check if a line looks like a dependency declaration.
fn looks_like_dependency_line(line: &str) -> bool {
    let line = line.trim();
    // Dependency lines typically have format: name = "version" or name = { ... }
    if let Some(after_eq) = line.split('=').nth(1) {
        let after_eq = after_eq.trim();
        // Version string or table
        after_eq.starts_with('"') || after_eq.starts_with('{') || after_eq == "true"
    } else {
        false
    }
}

/// Detect file oscillations by tracking file additions/deletions across commits.
///
/// # Arguments
/// * `base_branch` - Base branch for comparison
/// * `commits` - List of commit SHAs
/// * `extra_excludes` - Additional path prefixes to exclude
/// * `include_ephemeral` - If true, include ephemeral directories (for debugging)
fn detect_file_oscillations(
    base_branch: &str,
    commits: &[String],
    extra_excludes: &[String],
    include_ephemeral: bool,
) -> Vec<Oscillation> {
    let mut file_history: HashMap<String, SubjectHistory> = HashMap::new();

    for (i, commit) in commits.iter().enumerate() {
        let parent = if i == 0 { base_branch.to_string() } else { commits[i - 1].clone() };

        // Get list of added and deleted files in this commit
        let output = std::process::Command::new("git")
            .args(["diff", "--name-status", "--diff-filter=AD", &parent, commit])
            .output();

        if let Ok(out) = output {
            let diff = String::from_utf8_lossy(&out.stdout);
            for line in diff.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let status = parts[0];
                    let file_path = parts[1];

                    // Skip ephemeral directories
                    if should_exclude_path(file_path, extra_excludes, include_ephemeral) {
                        continue;
                    }

                    let action = match status {
                        "A" => SubjectAction::Add,
                        "D" => SubjectAction::Remove,
                        _ => continue,
                    };

                    let entry = file_history.entry(file_path.to_string()).or_default();
                    entry.push(commit.to_string(), action);
                }
            }
        }
    }

    // Convert histories to oscillations (ephemeral paths already filtered above)
    file_history
        .into_iter()
        .filter_map(|(file_path, history)| {
            history.to_oscillation_sequence().map(|sequence| Oscillation {
                oscillation_type: OscillationType::File,
                subject: file_path,
                sequence,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_history_no_oscillation_single_action() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);

        assert!(history.to_oscillation_sequence().is_none());
    }

    #[test]
    fn subject_history_no_oscillation_same_actions() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);
        history.push("commit2".to_string(), SubjectAction::Add);

        assert!(history.to_oscillation_sequence().is_none());
    }

    #[test]
    fn subject_history_detects_add_remove_oscillation() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);
        history.push("commit2".to_string(), SubjectAction::Remove);

        let sequence = history.to_oscillation_sequence().expect("should detect oscillation");
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence[0], OscillationAction::Add);
        assert_eq!(sequence[1], OscillationAction::Remove);
    }

    #[test]
    fn subject_history_detects_add_remove_add_oscillation() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);
        history.push("commit2".to_string(), SubjectAction::Remove);
        history.push("commit3".to_string(), SubjectAction::Add);

        let sequence = history.to_oscillation_sequence().expect("should detect oscillation");
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence[0], OscillationAction::Add);
        assert_eq!(sequence[1], OscillationAction::Remove);
        assert_eq!(sequence[2], OscillationAction::Add);
    }

    #[test]
    fn extract_dependency_name_simple() {
        assert_eq!(extract_dependency_name("serde = \"1.0\""), Some("serde".to_string()));
        assert_eq!(
            extract_dependency_name("tokio = { version = \"1.0\" }"),
            Some("tokio".to_string())
        );
    }

    #[test]
    fn extract_dependency_name_workspace() {
        assert_eq!(extract_dependency_name("serde.workspace = true"), Some("serde".to_string()));
    }

    #[test]
    fn extract_dependency_name_skips_metadata() {
        assert_eq!(extract_dependency_name("version = \"0.1.0\""), None);
        assert_eq!(extract_dependency_name("edition = \"2021\""), None);
        assert_eq!(extract_dependency_name("name = \"my-crate\""), None);
    }

    #[test]
    fn extract_dependency_name_invalid() {
        assert_eq!(extract_dependency_name(""), None);
        assert_eq!(extract_dependency_name("no-equals-sign"), None);
        assert_eq!(extract_dependency_name("[dependencies]"), None);
    }

    #[test]
    fn looks_like_dependency_line_valid() {
        assert!(looks_like_dependency_line("serde = \"1.0\""));
        assert!(looks_like_dependency_line("tokio = { version = \"1.0\" }"));
        assert!(looks_like_dependency_line("serde.workspace = true"));
    }

    #[test]
    fn looks_like_dependency_line_invalid() {
        assert!(!looks_like_dependency_line("[dependencies]"));
        assert!(!looks_like_dependency_line(""));
        assert!(!looks_like_dependency_line("no-equals"));
    }

    #[test]
    fn parse_cargo_toml_diff_extracts_deps() {
        let diff = r#"
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -10,0 +11,2 @@
+[dependencies]
+serde = "1.0"
"#;
        let mut history = HashMap::new();
        parse_cargo_toml_diff(diff, "abc123", &mut history);

        assert!(history.contains_key("serde"));
        assert_eq!(history["serde"].actions.len(), 1);
        assert_eq!(history["serde"].actions[0].1, SubjectAction::Add);
    }

    #[test]
    fn parse_cargo_toml_diff_tracks_removal() {
        let diff = r#"
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -10,2 +10,0 @@
-[dependencies]
-serde = "1.0"
"#;
        let mut history = HashMap::new();
        parse_cargo_toml_diff(diff, "abc123", &mut history);

        assert!(history.contains_key("serde"));
        assert_eq!(history["serde"].actions.len(), 1);
        assert_eq!(history["serde"].actions[0].1, SubjectAction::Remove);
    }

    #[test]
    fn file_category_tests_detection() {
        assert_eq!(FileCategory::from_path("crates/foo/tests/integration.rs"), FileCategory::Tests);
        assert_eq!(FileCategory::from_path("src/lib_test.rs"), FileCategory::Tests);
        assert_eq!(FileCategory::from_path("crates/x/src/tests.rs"), FileCategory::Tests);
    }

    #[test]
    fn file_category_docs_detection() {
        assert_eq!(FileCategory::from_path("docs/README.md"), FileCategory::Docs);
        assert_eq!(FileCategory::from_path("CHANGELOG.md"), FileCategory::Docs);
        assert_eq!(FileCategory::from_path("docs/api/overview.md"), FileCategory::Docs);
    }

    #[test]
    fn file_category_receipts_detection() {
        assert_eq!(
            FileCategory::from_path(".runs/current/receipts/gate.json"),
            FileCategory::Receipts
        );
        assert_eq!(
            FileCategory::from_path(".runs/pr123/receipts/timeline.json"),
            FileCategory::Receipts
        );
    }

    #[test]
    fn file_category_config_detection() {
        assert_eq!(FileCategory::from_path("Cargo.toml"), FileCategory::Config);
        assert_eq!(FileCategory::from_path("specs/config.yaml"), FileCategory::Config);
        assert_eq!(FileCategory::from_path(".claude/settings.json"), FileCategory::Config);
    }

    #[test]
    fn file_category_impl_detection() {
        assert_eq!(FileCategory::from_path("crates/foo/src/lib.rs"), FileCategory::Impl);
        assert_eq!(FileCategory::from_path("src/main.rs"), FileCategory::Impl);
    }

    #[test]
    fn file_category_is_stable() {
        assert!(FileCategory::Tests.is_stable());
        assert!(FileCategory::Docs.is_stable());
        assert!(FileCategory::Receipts.is_stable());
        assert!(FileCategory::Config.is_stable());
        assert!(!FileCategory::Impl.is_stable());
    }

    #[test]
    fn file_category_as_str() {
        assert_eq!(FileCategory::Tests.as_str(), "tests");
        assert_eq!(FileCategory::Docs.as_str(), "docs");
        assert_eq!(FileCategory::Receipts.as_str(), "receipts");
        assert_eq!(FileCategory::Config.as_str(), "config");
        assert_eq!(FileCategory::Impl.as_str(), "impl");
    }

    // ========================================================================
    // Path Normalization Tests
    // ========================================================================

    #[test]
    fn normalize_path_separators_converts_backslashes() {
        // Windows-style paths
        assert_eq!(normalize_path_separators(r"crates\xtask\src"), "crates/xtask/src");
        assert_eq!(normalize_path_separators(r".runs\pr\123"), ".runs/pr/123");
        assert_eq!(normalize_path_separators(r"target\debug\foo"), "target/debug/foo");
    }

    #[test]
    fn normalize_path_separators_preserves_forward_slashes() {
        // Unix-style paths remain unchanged
        assert_eq!(normalize_path_separators("crates/xtask/src"), "crates/xtask/src");
        assert_eq!(normalize_path_separators(".runs/pr/123"), ".runs/pr/123");
    }

    #[test]
    fn should_exclude_path_with_defaults() {
        // Default exclusions with empty extra excludes
        let empty_excludes: Vec<String> = vec![];

        assert!(should_exclude_path(".runs/current/foo.json", &empty_excludes, false));
        assert!(should_exclude_path("target/debug/foo", &empty_excludes, false));
        assert!(should_exclude_path(".git/objects/abc", &empty_excludes, false));

        // Non-excluded paths
        assert!(!should_exclude_path("src/main.rs", &empty_excludes, false));
        assert!(!should_exclude_path("crates/foo/src/lib.rs", &empty_excludes, false));
    }

    #[test]
    fn should_exclude_path_with_windows_separators() {
        let empty_excludes: Vec<String> = vec![];

        // Windows-style .runs path should be excluded
        assert!(should_exclude_path(r".runs\current\foo.json", &empty_excludes, false));
        assert!(should_exclude_path(r"target\debug\foo", &empty_excludes, false));
    }

    #[test]
    fn should_exclude_path_with_extra_excludes() {
        let extra_excludes = vec!["vendor/".to_string(), "custom_dir/".to_string()];

        // Extra exclusions
        assert!(should_exclude_path("vendor/crate/src", &extra_excludes, false));
        assert!(should_exclude_path("custom_dir/file.txt", &extra_excludes, false));

        // Non-excluded
        assert!(!should_exclude_path("src/main.rs", &extra_excludes, false));
    }

    #[test]
    fn should_exclude_path_include_ephemeral_flag() {
        let empty_excludes: Vec<String> = vec![];

        // With include_ephemeral=true, default exclusions are skipped
        assert!(!should_exclude_path(".runs/current/foo.json", &empty_excludes, true));
        assert!(!should_exclude_path("target/debug/foo", &empty_excludes, true));

        // But extra exclusions still apply
        let extra = vec!["always_exclude/".to_string()];
        assert!(should_exclude_path("always_exclude/file.txt", &extra, true));
    }
}
