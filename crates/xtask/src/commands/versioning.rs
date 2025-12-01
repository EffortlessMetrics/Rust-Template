//! Versioning module for semantic version parsing, validation, and management.
//!
//! This module provides utilities for:
//! - Parsing and validating semantic version strings (X.Y.Z format)
//! - Managing version information including tags and dates
//! - Loading version manifests for release automation
//! - Planning and applying version updates atomically

use anyhow::{Context, Result};
use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::path::Path;

/// Semantic version with major, minor, and patch components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Create a new Version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Parse a version string in X.Y.Z format.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        // Strip optional 'v' prefix
        let s = s.strip_prefix('v').unwrap_or(s);

        let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)$").context("Failed to compile version regex")?;

        let caps = re
            .captures(s)
            .with_context(|| format!("Invalid version format '{}', expected X.Y.Z", s))?;

        let major = caps[1]
            .parse::<u32>()
            .with_context(|| format!("Invalid major version: {}", &caps[1]))?;
        let minor = caps[2]
            .parse::<u32>()
            .with_context(|| format!("Invalid minor version: {}", &caps[2]))?;
        let patch = caps[3]
            .parse::<u32>()
            .with_context(|| format!("Invalid patch version: {}", &caps[3]))?;

        Ok(Version { major, minor, patch })
    }

    /// Convert to git tag format (vX.Y.Z).
    pub fn to_tag(&self) -> String {
        format!("v{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Convert to kernel tag format (vX.Y.Z-kernel).
    pub fn to_kernel_tag(&self) -> String {
        format!("v{}.{}.{}-kernel", self.major, self.minor, self.patch)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

/// Validate a version string format and return the parsed Version.
pub fn validate_version_format(version: &str) -> Result<Version> {
    Version::parse(version)
}

/// Complete version information including tags and date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub tag: String,
    pub kernel_tag: String,
    pub date: String,
}

impl VersionInfo {
    /// Create VersionInfo with the current date.
    pub fn new(version_str: &str) -> Result<Self> {
        let version = Version::parse(version_str)?;
        let date = Local::now().format("%Y-%m-%d").to_string();
        Ok(Self {
            version: version.to_string(),
            tag: version.to_tag(),
            kernel_tag: version.to_kernel_tag(),
            date,
        })
    }

    /// Create VersionInfo with a specific date.
    pub fn with_date(version_str: &str, date: &str) -> Result<Self> {
        let version = Version::parse(version_str)?;
        Ok(Self {
            version: version.to_string(),
            tag: version.to_tag(),
            kernel_tag: version.to_kernel_tag(),
            date: date.to_string(),
        })
    }
}

/// A planned file edit.
#[derive(Debug, Clone)]
pub struct FileEdit {
    pub path: String,
    pub line_number: usize,
    pub old_text: String,
    pub new_text: String,
}

impl FileEdit {
    /// Format a preview of this edit for dry-run display.
    pub fn format_preview(&self) -> String {
        format!("{}:{}\n  - {}\n  + {}", self.path, self.line_number, self.old_text, self.new_text)
    }
}

/// Pattern for version replacement in a file.
/// Matches the structure in specs/version_manifest.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct FilePattern {
    /// Human-readable marker to locate the line
    pub marker: String,
    /// Pattern type: yaml_value, heading_version, inline_version, etc.
    #[serde(rename = "type")]
    pub pattern_type: String,
    /// Format style: quoted, prefixed, prefixed_paren, etc.
    pub format: String,
    /// Regex pattern for the entire line
    #[serde(default)]
    pub line_pattern: Option<String>,
    /// Example of the expected format
    #[serde(default)]
    pub example: Option<String>,
    /// Additional notes
    #[serde(default)]
    pub notes: Option<String>,
}

/// Target file with its patterns.
/// Matches the structure in specs/version_manifest.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionTarget {
    /// Path to the file
    pub path: String,
    /// Description of the file's purpose
    #[serde(default)]
    pub description: Option<String>,
    /// Patterns to match and update
    pub patterns: Vec<FilePattern>,
    /// Whether this file is required to exist
    #[serde(default = "default_true")]
    pub required: bool,
    /// Update priority (1=highest)
    #[serde(default = "default_priority")]
    pub priority: u32,
    /// Additional notes
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_priority() -> u32 {
    5
}

/// Version format specification.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionFormat {
    pub pattern: String,
    #[serde(default)]
    pub examples: Vec<String>,
}

/// Version manifest declaring all version-bearing files.
/// Matches the structure in specs/version_manifest.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifest {
    pub schema_version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version_format: Option<VersionFormat>,
    pub files: Vec<VersionTarget>,
}

/// Source of truth for version information.
pub const VERSION_SOURCE_OF_TRUTH: &str = "specs/spec_ledger.yaml";
pub const VERSION_PATH: &str = "metadata.template_version";
pub const DATE_PATH: &str = "metadata.last_updated";

impl VersionManifest {
    /// Load the version manifest from specs/version_manifest.yaml.
    pub fn load() -> Result<Self> {
        let manifest_path = "specs/version_manifest.yaml";
        if !Path::new(manifest_path).exists() {
            anyhow::bail!(
                "Version manifest not found at {}. Create it to enable manifest-driven versioning.",
                manifest_path
            );
        }

        let content = fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path))?;

        serde_yaml::from_str(&content).with_context(|| format!("Failed to parse {}", manifest_path))
    }

    /// Extract the current version from the source of truth file (specs/spec_ledger.yaml).
    pub fn extract_current_version(&self) -> Result<VersionInfo> {
        let content = fs::read_to_string(VERSION_SOURCE_OF_TRUTH).with_context(|| {
            format!("Failed to read source of truth: {}", VERSION_SOURCE_OF_TRUTH)
        })?;

        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", VERSION_SOURCE_OF_TRUTH))?;

        // Extract version using dot-notation path
        let version = extract_yaml_value(&yaml, VERSION_PATH)
            .with_context(|| format!("Failed to extract version at {}", VERSION_PATH))?;

        // Extract date using dot-notation path
        let date = extract_yaml_value(&yaml, DATE_PATH)
            .with_context(|| format!("Failed to extract date at {}", DATE_PATH))?;

        VersionInfo::with_date(&version, &date)
    }

    /// Get files sorted by priority (lowest number = highest priority).
    pub fn files_by_priority(&self) -> Vec<&VersionTarget> {
        let mut files: Vec<_> = self.files.iter().collect();
        files.sort_by_key(|f| f.priority);
        files
    }
}

/// Extract a value from YAML using dot-notation path (e.g., "metadata.template_version").
fn extract_yaml_value(yaml: &serde_yaml::Value, path: &str) -> Result<String> {
    // Handle optional JSONPath-style prefix
    let path = path.strip_prefix("$.").unwrap_or(path);

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = yaml;

    for part in parts {
        current =
            current.get(part).with_context(|| format!("Key '{}' not found in YAML path", part))?;
    }

    current
        .as_str()
        .map(|s| s.to_string())
        .with_context(|| format!("Value at '{}' is not a string", path))
}

/// Generate the new line by substituting version in the example pattern.
fn generate_new_line(pattern: &FilePattern, version: &VersionInfo) -> Result<String> {
    // Use the example field if available, otherwise construct from format
    let example = pattern
        .example
        .as_ref()
        .with_context(|| format!("Pattern '{}' missing example field", pattern.marker))?;

    // The example contains a version like "3.3.5" - replace it with the new version
    // We use regex to find and replace version patterns
    let version_re =
        Regex::new(r"\d+\.\d+\.\d+(-kernel)?").context("Failed to compile version regex")?;

    // Determine what to substitute based on format
    let replacement = match pattern.format.as_str() {
        "kernel_suffixed" => format!("{}-kernel", version.version),
        _ => version.version.clone(),
    };

    // Replace all version occurrences in the example
    let new_line = version_re.replace_all(example, replacement.as_str()).to_string();

    // Also handle date substitution if present
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").context("Failed to compile date regex")?;
    let new_line = date_re.replace_all(&new_line, version.date.as_str()).to_string();

    Ok(new_line)
}

/// Plan version changes for all target files.
pub fn plan_changes(version: &VersionInfo, manifest: &VersionManifest) -> Result<Vec<FileEdit>> {
    let mut edits = Vec::new();

    for target in &manifest.files {
        // Skip glob patterns (release_evidence files)
        if target.path.contains('*') {
            continue;
        }

        if !Path::new(&target.path).exists() {
            if !target.required {
                continue; // Skip optional files that don't exist
            }
            anyhow::bail!("Target file not found: {}", target.path);
        }

        let content = fs::read_to_string(&target.path)
            .with_context(|| format!("Failed to read {}", target.path))?;

        let lines: Vec<&str> = content.lines().collect();

        for pattern in &target.patterns {
            // Skip patterns without examples (like [Unreleased] section which needs special handling)
            if pattern.example.is_none() {
                continue;
            }

            let mut found = false;
            for (i, line) in lines.iter().enumerate() {
                if line.contains(&pattern.marker) {
                    found = true;

                    let new_line = generate_new_line(pattern, version)?;

                    if *line != new_line {
                        edits.push(FileEdit {
                            path: target.path.clone(),
                            line_number: i + 1,
                            old_text: line.to_string(),
                            new_text: new_line,
                        });
                    }
                    break;
                }
            }

            if !found && target.required {
                // Only warn for required files with missing patterns
                eprintln!("Warning: Pattern '{}' not found in {}", pattern.marker, target.path);
            }
        }
    }

    Ok(edits)
}

/// Apply planned changes to files.
///
/// If `dry_run` is true, only prints what would be changed.
/// Otherwise, applies changes atomically using temp files.
pub fn apply_changes(edits: &[FileEdit], dry_run: bool) -> Result<()> {
    use colored::Colorize;
    use std::collections::HashMap;

    if dry_run {
        println!("{}", "Dry run - changes that would be made:".blue().bold());
        println!();
        for edit in edits {
            println!("{}", edit.path.cyan());
            println!("  Line {}: ", edit.line_number);
            println!("    {} {}", "-".red(), edit.old_text.dimmed());
            println!("    {} {}", "+".green(), edit.new_text.green());
            println!();
        }
        println!(
            "{} {} edits across {} files",
            "Would make".blue(),
            edits.len(),
            edits.iter().map(|e| &e.path).collect::<std::collections::HashSet<_>>().len()
        );
        return Ok(());
    }

    // Group edits by file
    let mut by_file: HashMap<&str, Vec<&FileEdit>> = HashMap::new();
    for edit in edits {
        by_file.entry(&edit.path).or_default().push(edit);
    }

    // Apply edits file by file
    for (path, file_edits) in by_file {
        apply_file_edits(path, &file_edits)?;
    }

    println!("{} Applied {} edits", "✓".green(), edits.len());

    Ok(())
}

/// Apply multiple edits to a single file atomically.
fn apply_file_edits(path: &str, edits: &[&FileEdit]) -> Result<()> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path))?;

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Sort edits by line number descending to avoid index shifting
    let mut sorted_edits: Vec<&&FileEdit> = edits.iter().collect();
    sorted_edits.sort_by(|a, b| b.line_number.cmp(&a.line_number));

    for edit in sorted_edits {
        let idx = edit.line_number - 1;
        if idx >= lines.len() {
            anyhow::bail!(
                "Line {} out of range in {} (file has {} lines)",
                edit.line_number,
                path,
                lines.len()
            );
        }
        if lines[idx] != edit.old_text {
            anyhow::bail!(
                "Text mismatch at {}:{}\n  Expected: {}\n  Found: {}",
                path,
                edit.line_number,
                edit.old_text,
                lines[idx]
            );
        }
        lines[idx] = edit.new_text.clone();
    }

    // Write atomically via temp file
    let temp_path = format!("{}.tmp", path);
    let new_content = lines.join("\n") + "\n";
    fs::write(&temp_path, &new_content)
        .with_context(|| format!("Failed to write temp file {}", temp_path))?;

    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename {} to {}", temp_path, path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse_valid() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_parse_with_prefix() {
        let v = Version::parse("v3.3.5").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 3);
        assert_eq!(v.patch, 5);
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("abc").is_err());
        assert!(Version::parse("").is_err());
    }

    #[test]
    fn test_version_to_string() {
        let v = Version::new(3, 3, 5);
        assert_eq!(v.to_string(), "3.3.5");
    }

    #[test]
    fn test_version_to_tag() {
        let v = Version::new(3, 3, 5);
        assert_eq!(v.to_tag(), "v3.3.5");
    }

    #[test]
    fn test_version_to_kernel_tag() {
        let v = Version::new(3, 3, 5);
        assert_eq!(v.to_kernel_tag(), "v3.3.5-kernel");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        let v3 = Version::new(1, 1, 0);
        let v4 = Version::new(1, 0, 1);

        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v1 < v4);
        assert!(v3 > v4);
    }

    #[test]
    fn test_version_info_new() {
        let info = VersionInfo::new("3.3.6").unwrap();
        assert_eq!(info.version, "3.3.6");
        assert_eq!(info.tag, "v3.3.6");
        assert_eq!(info.kernel_tag, "v3.3.6-kernel");
        // date is current date, just check it's not empty
        assert!(!info.date.is_empty());
    }

    #[test]
    fn test_version_info_with_date() {
        let info = VersionInfo::with_date("3.3.6", "2025-12-01").unwrap();
        assert_eq!(info.version, "3.3.6");
        assert_eq!(info.date, "2025-12-01");
    }

    #[test]
    fn test_file_edit_preview() {
        let edit = FileEdit {
            path: "README.md".to_string(),
            line_number: 5,
            old_text: "version: 3.3.5".to_string(),
            new_text: "version: 3.3.6".to_string(),
        };
        let preview = edit.format_preview();
        assert!(preview.contains("README.md:5"));
        assert!(preview.contains("3.3.5"));
        assert!(preview.contains("3.3.6"));
    }
}
