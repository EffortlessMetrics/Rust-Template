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

    /// Save friction entry to YAML file
    #[allow(dead_code)]
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

/// List friction entries filtered by status or severity
pub fn list_friction_entries(
    status_filter: Option<&str>,
    severity_filter: Option<&str>,
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
        println!("No friction entries found.");
        return Ok(());
    }

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

    let stats = calculate_stats(&entries);
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
