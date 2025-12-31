use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Friction entry representing process/tooling issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionEntry {
    pub id: String,
    pub date: String,
    pub category: String,
    pub severity: String,
    pub summary: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_behavior: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workaround: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<FrictionContext>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_items: Option<RelatedItems>,
}

fn default_status() -> String {
    "open".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_involved: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub resolved_by: String,
    pub resolved_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pr_links: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItems {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<String>,
}

/// Friction statistics
#[derive(Debug, Default)]
pub struct FrictionStats {
    pub open_count: usize,
    pub investigating_count: usize,
    pub in_progress_count: usize,
    pub resolved_count: usize,
    pub wont_fix_count: usize,
    pub total_count: usize,
    pub by_severity: SeverityCounts,
}

#[derive(Debug, Default)]
pub struct SeverityCounts {
    pub low: usize,
    pub medium: usize,
    pub high: usize,
    pub critical: usize,
}

impl FrictionEntry {
    /// Load friction entry from YAML file
    pub fn load(filepath: &Path) -> Result<Self> {
        let content = fs::read_to_string(filepath).with_context(|| {
            format!("Failed to read friction entry file: {}", filepath.display())
        })?;

        let entry: FrictionEntry = serde_yaml::from_str(&content).with_context(|| {
            format!("Failed to parse friction entry YAML: {}", filepath.display())
        })?;

        Ok(entry)
    }

    /// Save friction entry to YAML file.
    ///
    /// Used by `create_friction_entry()` to persist new entries.
    pub fn save(&self) -> Result<PathBuf> {
        let friction_dir = Path::new("friction");
        fs::create_dir_all(friction_dir)
            .with_context(|| format!("Failed to create directory: {}", friction_dir.display()))?;

        let filename = format!("{}.yaml", self.id);
        let filepath = friction_dir.join(&filename);

        let yaml_content = serde_yaml::to_string(&self)
            .with_context(|| format!("Failed to serialize friction entry: {}", self.id))?;

        // Add header comment
        let content = format!(
            "# Friction Entry: {}\n# Created at {}\n# Status: {}\n\n{}",
            self.summary, self.date, self.status, yaml_content
        );

        fs::write(&filepath, content).with_context(|| {
            format!("Failed to write friction entry file: {}", filepath.display())
        })?;

        Ok(filepath)
    }
}

/// Load all friction entries from friction/ directory
pub fn load_all_friction_entries() -> Result<Vec<FrictionEntry>> {
    let friction_dir = Path::new("friction");
    if !friction_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(friction_dir)
        .with_context(|| format!("Failed to read directory: {}", friction_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match FrictionEntry::load(&path) {
            Ok(friction) => entries.push(friction),
            Err(e) => {
                eprintln!("Warning: Failed to load friction entry from {}: {}", path.display(), e);
            }
        }
    }

    // Sort by date (most recent first)
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(entries)
}

/// Calculate friction statistics
pub fn calculate_stats(entries: &[FrictionEntry]) -> FrictionStats {
    let mut stats = FrictionStats::default();

    for entry in entries {
        stats.total_count += 1;

        // Count by status
        match entry.status.as_str() {
            "open" => stats.open_count += 1,
            "investigating" => stats.investigating_count += 1,
            "in_progress" => stats.in_progress_count += 1,
            "resolved" => stats.resolved_count += 1,
            "wont_fix" => stats.wont_fix_count += 1,
            _ => {}
        }

        // Count by severity
        match entry.severity.as_str() {
            "low" => stats.by_severity.low += 1,
            "medium" => stats.by_severity.medium += 1,
            "high" => stats.by_severity.high += 1,
            "critical" => stats.by_severity.critical += 1,
            _ => {}
        }
    }

    stats
}

/// JSON output structure for friction-list
#[derive(Debug, Serialize)]
struct FrictionListJson {
    timestamp: String,
    total_count: usize,
    stats: FrictionStatsJson,
    entries: Vec<FrictionEntry>,
}

#[derive(Debug, Serialize)]
struct FrictionStatsJson {
    open: usize,
    investigating: usize,
    in_progress: usize,
    resolved: usize,
    wont_fix: usize,
    severity: FrictionSeverityStatsJson,
}

#[derive(Debug, Serialize)]
struct FrictionSeverityStatsJson {
    low: usize,
    medium: usize,
    high: usize,
    critical: usize,
}

/// List friction entries filtered by status or severity
pub fn list_friction_entries(
    status_filter: Option<&str>,
    severity_filter: Option<&str>,
    json: bool,
) -> Result<()> {
    let entries = load_all_friction_entries()?;

    let filtered: Vec<&FrictionEntry> = entries
        .iter()
        .filter(|e| {
            let status_match = status_filter.is_none() || Some(e.status.as_str()) == status_filter;
            let severity_match =
                severity_filter.is_none() || Some(e.severity.as_str()) == severity_filter;
            status_match && severity_match
        })
        .collect();

    if filtered.is_empty() {
        if json {
            // Empty JSON output
            let output = FrictionListJson {
                timestamp: chrono::Utc::now().to_rfc3339(),
                total_count: 0,
                stats: FrictionStatsJson {
                    open: 0,
                    investigating: 0,
                    in_progress: 0,
                    resolved: 0,
                    wont_fix: 0,
                    severity: FrictionSeverityStatsJson { low: 0, medium: 0, high: 0, critical: 0 },
                },
                entries: Vec::new(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No friction entries found.");
        }
        return Ok(());
    }

    let stats = calculate_stats(&entries);

    if json {
        // JSON output
        let output = FrictionListJson {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_count: filtered.len(),
            stats: FrictionStatsJson {
                open: stats.open_count,
                investigating: stats.investigating_count,
                in_progress: stats.in_progress_count,
                resolved: stats.resolved_count,
                wont_fix: stats.wont_fix_count,
                severity: FrictionSeverityStatsJson {
                    low: stats.by_severity.low,
                    medium: stats.by_severity.medium,
                    high: stats.by_severity.high,
                    critical: stats.by_severity.critical,
                },
            },
            entries: filtered.into_iter().cloned().collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable output
        println!(
            "\n{} Friction Entries:\n",
            if status_filter.is_some() || severity_filter.is_some() { "Filtered" } else { "All" }
        );

        for entry in filtered {
            let status_badge = match entry.status.as_str() {
                "open" => "🔴 OPEN",
                "investigating" => "🔍 INVESTIGATING",
                "in_progress" => "🔧 IN PROGRESS",
                "resolved" => "✅ RESOLVED",
                "wont_fix" => "⛔ WONT FIX",
                _ => "❓ UNKNOWN",
            };

            let severity_badge = match entry.severity.as_str() {
                "critical" => "🔥 CRITICAL",
                "high" => "❗ HIGH",
                "medium" => "⚠️  MEDIUM",
                "low" => "ℹ️  LOW",
                _ => "",
            };

            println!("  {} {} {} - {}", status_badge, severity_badge, entry.id, entry.summary);
            println!("     Date: {} | Category: {}", entry.date, entry.category);
            if let Some(context) = &entry.context
                && let Some(flow) = &context.flow
            {
                println!("     Flow: {}", flow);
            }
            println!();
        }

        println!("Total: {} entries", stats.total_count);
        println!(
            "Status: open: {}, investigating: {}, in_progress: {}, resolved: {}, wont_fix: {}",
            stats.open_count,
            stats.investigating_count,
            stats.in_progress_count,
            stats.resolved_count,
            stats.wont_fix_count
        );
        println!(
            "Severity: critical: {}, high: {}, medium: {}, low: {}\n",
            stats.by_severity.critical,
            stats.by_severity.high,
            stats.by_severity.medium,
            stats.by_severity.low
        );
    }

    Ok(())
}

/// Get the category prefix for friction ID generation
fn get_category_prefix(category: &str) -> &str {
    match category {
        "tooling" => "TOOL",
        "process" => "PROC",
        "documentation" => "DOCS",
        "devex" => "DEVEX",
        "ci_cd" => "CI",
        "platform" => "PLAT",
        "api" => "API",
        "testing" => "TEST",
        "governance" => "GOV",
        "other" => "OTHER",
        _ => "OTHER",
    }
}

/// Find the next available friction ID for a given category
fn find_next_friction_id(category: &str) -> Result<String> {
    let entries = load_all_friction_entries()?;
    let prefix = get_category_prefix(category);
    let pattern = format!("FRICTION-{}-", prefix);

    // Find the highest number for this category
    let max_number = entries
        .iter()
        .filter_map(|e| {
            if e.id.starts_with(&pattern) {
                // Extract the number part
                e.id.strip_prefix(&pattern)?.parse::<u32>().ok()
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    // Next ID is max + 1
    let next_number = max_number + 1;
    Ok(format!("FRICTION-{}-{:03}", prefix, next_number))
}

/// Create a new friction entry
#[allow(clippy::too_many_arguments)]
pub fn create_friction_entry(
    category: &str,
    severity: &str,
    summary: &str,
    description: Option<&str>,
    flow: Option<&str>,
    phase: Option<&str>,
    discovered_by: Option<&str>,
    refs: &[String],
) -> Result<()> {
    // Validate category
    let valid_categories = [
        "tooling",
        "process",
        "documentation",
        "devex",
        "ci_cd",
        "platform",
        "api",
        "testing",
        "governance",
        "other",
    ];
    if !valid_categories.contains(&category) {
        anyhow::bail!(
            "Invalid category '{}'. Must be one of: {}",
            category,
            valid_categories.join(", ")
        );
    }

    // Validate severity
    let valid_severities = ["low", "medium", "high", "critical"];
    if !valid_severities.contains(&severity) {
        anyhow::bail!(
            "Invalid severity '{}'. Must be one of: {}",
            severity,
            valid_severities.join(", ")
        );
    }

    // Generate ID
    let id = find_next_friction_id(category)?;

    // Get today's date
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Build context if any fields provided
    let context = if flow.is_some() || phase.is_some() || discovered_by.is_some() {
        Some(FrictionContext {
            discovered_by: Some(discovered_by.unwrap_or("human").to_string()),
            flow: flow.map(String::from),
            phase: phase.map(String::from),
            files_involved: Vec::new(),
            commands_involved: Vec::new(),
        })
    } else {
        Some(FrictionContext {
            discovered_by: Some("human".to_string()),
            flow: None,
            phase: None,
            files_involved: Vec::new(),
            commands_involved: Vec::new(),
        })
    };

    // Create friction entry
    let entry = FrictionEntry {
        id: id.clone(),
        date: date.clone(),
        category: category.to_string(),
        severity: severity.to_string(),
        summary: summary.to_string(),
        description: description.unwrap_or(summary).to_string(),
        expected_behavior: None,
        workaround: None,
        impact: None,
        context,
        status: "open".to_string(),
        resolution: None,
        refs: refs.to_vec(),
        related_items: None,
    };

    // Save to file
    let filepath = entry.save()?;

    println!("✅ Created friction entry: {}", id);
    println!("   File: {}", filepath.display());
    println!("   Date: {}", date);
    println!("   Category: {}", category);
    println!("   Severity: {}", severity);
    println!("   Status: open");

    Ok(())
}

/// Create a GitHub issue from a friction entry
pub fn gh_create_issue(
    friction_id: &str,
    extra_labels: Option<&str>,
    dry_run: bool,
    open_in_browser: bool,
) -> Result<()> {
    use crate::commands::github::{GhClient, friction_issue_body, friction_labels};

    // Load the friction entry
    let friction_dir = Path::new("friction");
    let file_path = friction_dir.join(format!("{}.yaml", friction_id));

    if !file_path.exists() {
        anyhow::bail!("Friction entry '{}' not found at {}", friction_id, file_path.display());
    }

    let entry = FrictionEntry::load(&file_path)?;

    // Check if already linked to a GitHub issue
    if let Some(ref related) = entry.related_items
        && !related.issues.is_empty()
    {
        eprintln!(
            "⚠️  Warning: Friction entry '{}' is already linked to GitHub issues: {}",
            friction_id,
            related.issues.join(", ")
        );
    }

    // Generate labels
    let mut labels = friction_labels(&entry.category, &entry.severity);
    if let Some(extra) = extra_labels {
        labels.extend(extra.split(',').map(|s| s.trim().to_string()));
    }

    // Generate issue body
    let flow = entry.context.as_ref().and_then(|c| c.flow.as_deref());
    let phase = entry.context.as_ref().and_then(|c| c.phase.as_deref());
    let body = friction_issue_body(
        &entry.id,
        &entry.summary,
        &entry.description,
        &entry.category,
        &entry.severity,
        &entry.date,
        flow,
        phase,
        &entry.refs,
    );

    let title = format!("[Friction] {}", entry.summary);

    if dry_run {
        println!("🔍 Dry run - would create GitHub issue:\n");
        println!("Title: {}", title);
        println!("Labels: {}", labels.join(", "));
        println!("\nBody:\n{}", body);
        return Ok(());
    }

    // Create the issue
    println!("Creating GitHub issue from friction entry {}...", friction_id);
    let issue_ref = GhClient::create_issue(&title, &body, &labels)?;

    println!("✅ Created GitHub issue: {}", issue_ref.url);
    println!("   Issue number: #{}", issue_ref.number);

    // Update friction entry with issue reference
    let mut updated_entry = entry.clone();
    let related = updated_entry.related_items.get_or_insert(RelatedItems {
        issues: Vec::new(),
        adrs: Vec::new(),
        tasks: Vec::new(),
    });
    related.issues.push(issue_ref.as_short());
    updated_entry.save()?;

    println!("   Updated friction entry with issue reference");

    if open_in_browser {
        GhClient::open_in_browser(issue_ref.number)?;
    }

    Ok(())
}

/// Link an existing GitHub issue to a friction entry
pub fn gh_link_issue(friction_id: &str, issue_number: &str) -> Result<()> {
    use crate::commands::github::IssueRef;

    // Parse issue number
    let issue_ref = IssueRef::parse(issue_number)
        .ok_or_else(|| anyhow::anyhow!("Invalid issue number: {}", issue_number))?;

    // Load the friction entry
    let friction_dir = Path::new("friction");
    let file_path = friction_dir.join(format!("{}.yaml", friction_id));

    if !file_path.exists() {
        anyhow::bail!("Friction entry '{}' not found at {}", friction_id, file_path.display());
    }

    let mut entry = FrictionEntry::load(&file_path)?;

    // Check if already linked
    let issue_short = issue_ref.as_short();
    if let Some(ref related) = entry.related_items
        && related.issues.contains(&issue_short)
    {
        println!("ℹ️  Friction entry '{}' is already linked to {}", friction_id, issue_short);
        return Ok(());
    }

    // Add issue reference
    let related = entry.related_items.get_or_insert(RelatedItems {
        issues: Vec::new(),
        adrs: Vec::new(),
        tasks: Vec::new(),
    });
    related.issues.push(issue_short.clone());

    // Save the updated entry
    entry.save()?;

    println!("✅ Linked friction entry {} to GitHub issue {}", friction_id, issue_short);

    Ok(())
}

/// Resolve a friction entry (mark as resolved/wont_fix with resolution details)
#[allow(clippy::too_many_arguments)]
pub fn resolve_friction_entry(
    id: &str,
    resolved_by: &str,
    fix_description: Option<&str>,
    pr_links: &[String],
    verification: Option<&str>,
    status: &str,
) -> Result<()> {
    // Validate status
    let valid_statuses = ["resolved", "wont_fix"];
    if !valid_statuses.contains(&status) {
        anyhow::bail!("Invalid status '{}'. Must be one of: {}", status, valid_statuses.join(", "));
    }

    // Find and load the friction entry
    let friction_dir = Path::new("friction");
    let file_path = friction_dir.join(format!("{}.yaml", id));

    if !file_path.exists() {
        anyhow::bail!("Friction entry '{}' not found at {}", id, file_path.display());
    }

    let mut entry = FrictionEntry::load(&file_path)?;

    // Warn if already resolved
    if entry.status == "resolved" || entry.status == "wont_fix" {
        eprintln!(
            "⚠️  Warning: Friction entry '{}' is already {} (re-resolving)",
            id, entry.status
        );
    }

    // Update status and add resolution
    entry.status = status.to_string();
    entry.resolution = Some(Resolution {
        resolved_by: resolved_by.to_string(),
        resolved_at: chrono::Utc::now().to_rfc3339(),
        fix_description: fix_description.map(String::from),
        pr_links: pr_links.to_vec(),
        verification: verification.map(String::from),
    });

    // Save the updated entry
    entry.save()?;

    println!("✅ Resolved friction entry: {}", id);
    println!("   Status: {}", status);
    println!("   Resolved by: {}", resolved_by);
    if let Some(desc) = fix_description {
        println!("   Fix: {}", desc);
    }
    if !pr_links.is_empty() {
        println!("   PRs: {}", pr_links.join(", "));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friction_entry_serialization() {
        let entry = FrictionEntry {
            id: "FRICTION-TEST-001".to_string(),
            date: "2025-11-26".to_string(),
            category: "testing".to_string(),
            severity: "low".to_string(),
            summary: "Test friction entry".to_string(),
            description: "Test description".to_string(),
            expected_behavior: None,
            workaround: None,
            impact: None,
            context: None,
            status: "open".to_string(),
            resolution: None,
            refs: Vec::new(),
            related_items: None,
        };

        let yaml = serde_yaml::to_string(&entry).unwrap();
        assert!(yaml.contains("FRICTION-TEST-001"));
        assert!(yaml.contains("testing"));
    }

    #[test]
    fn test_friction_entry_deserialization() {
        let yaml = r#"
id: FRICTION-TEST-002
date: "2025-11-26"
category: devex
severity: medium
summary: "Test friction"
description: "Test description"
status: open
"#;

        let entry: FrictionEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.id, "FRICTION-TEST-002");
        assert_eq!(entry.category, "devex");
        assert_eq!(entry.status, "open");
    }
}
