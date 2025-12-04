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
    pub fn to_tag(self) -> String {
        format!("v{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Convert to kernel tag format (vX.Y.Z-kernel).
    pub fn to_kernel_tag(self) -> String {
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
    /// Future: Used when implementing retroactive version updates for historical releases.
    /// Currently only used in tests. See TASK-DX-VERSION-HISTORY for version history features.
    #[allow(dead_code)]
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

/// Pattern for version replacement in a file.
/// Matches the structure in specs/version_manifest.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct FilePattern {
    /// Human-readable marker to locate the line
    pub marker: String,
    /// Pattern type: yaml_value, heading_version, inline_version, etc.
    /// Future: Used when implementing pattern-based version replacement strategies.
    /// See AC-KERN-VERSION-UPDATE for version update automation requirements.
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub pattern_type: String,
    /// Format style: quoted, prefixed, prefixed_paren, etc.
    pub format: String,
    /// Regex pattern for the entire line
    #[serde(default)]
    pub line_pattern: Option<String>,
    /// Example of the expected format
    #[serde(default)]
    pub example: Option<String>,
    /// Additional notes about the pattern.
    /// Future: Displayed in version manifest validation errors for better UX.
    #[serde(default)]
    #[allow(dead_code)]
    pub notes: Option<String>,
}

/// Target file with its patterns.
/// Matches the structure in specs/version_manifest.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionTarget {
    /// Path to the file
    pub path: String,
    /// Description of the file's purpose.
    /// Future: Displayed in version update UI and validation reports.
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,
    /// Patterns to match and update
    pub patterns: Vec<FilePattern>,
    /// Whether this file is required to exist
    #[serde(default = "default_true")]
    pub required: bool,
    /// Update priority (1=highest).
    /// Future: Used when implementing ordered version updates across files.
    /// See TASK-DX-VERSION-ORDERING for version update sequencing.
    #[serde(default = "default_priority")]
    #[allow(dead_code)]
    pub priority: u32,
    /// Additional notes about this file.
    /// Future: Displayed in version update reports and validation errors.
    #[serde(default)]
    #[allow(dead_code)]
    pub notes: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_priority() -> u32 {
    5
}

/// Version format specification.
/// Future: Used when implementing custom version formats and validation.
/// See TASK-DX-VERSION-FORMATS for planned version format features.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionFormat {
    /// Regex pattern for valid version strings.
    /// Future: Used in version validation and custom format support.
    #[allow(dead_code)]
    pub pattern: String,
    /// Example version strings matching this format.
    /// Future: Displayed in version validation error messages.
    #[serde(default)]
    #[allow(dead_code)]
    pub examples: Vec<String>,
}

/// Version manifest declaring all version-bearing files.
/// Matches the structure in specs/version_manifest.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifest {
    /// Schema version for version manifest format.
    /// Future: Used for version manifest migration and compatibility checks.
    #[allow(dead_code)]
    pub schema_version: String,
    /// Human-readable description of the version manifest.
    /// Future: Displayed in version manifest validation reports.
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,
    /// Custom version format specification.
    /// Future: Enables custom version formats beyond semantic versioning.
    /// See TASK-DX-VERSION-FORMATS for custom format support.
    #[serde(default)]
    #[allow(dead_code)]
    pub version_format: Option<VersionFormat>,
    pub files: Vec<VersionTarget>,
}

impl VersionManifest {
    /// Load the version manifest from specs/version_manifest.yaml.
    /// Searches from repo root (determined via CARGO_MANIFEST_DIR for xtask).
    pub fn load() -> Result<Self> {
        Self::load_from_path(Self::default_manifest_path()?)
    }

    /// Get the default manifest path, resolving from repo root.
    fn default_manifest_path() -> Result<std::path::PathBuf> {
        // When running as part of xtask, CARGO_MANIFEST_DIR is crates/xtask
        // We need to go up two levels to get to repo root
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .context("Could not determine repo root from CARGO_MANIFEST_DIR")?;
        Ok(repo_root.join("specs/version_manifest.yaml"))
    }

    /// Load a version manifest from a specific path.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let manifest_path = path.as_ref();
        if !manifest_path.exists() {
            anyhow::bail!(
                "Version manifest not found at {}. Create it to enable manifest-driven versioning.",
                manifest_path.display()
            );
        }

        let content = fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", manifest_path.display()))
    }
}

/// Generate the new line by substituting version in the example pattern.
fn generate_new_line(pattern: &FilePattern, version: &VersionInfo) -> Result<String> {
    // Use the example field if available
    let example = pattern
        .example
        .as_ref()
        .with_context(|| format!("Pattern '{}' missing example field", pattern.marker))?;

    // Find the marker position in the example
    let marker_pos = example.find(&pattern.marker);

    // The example contains a version like "3.3.5" - replace the first one after the marker
    let version_re =
        Regex::new(r"\d+\.\d+\.\d+(-kernel)?").context("Failed to compile version regex")?;

    // Determine what to substitute based on format
    let replacement = match pattern.format.as_str() {
        "kernel_suffixed" => format!("{}-kernel", version.version),
        _ => version.version.clone(),
    };

    // Replace only the first version occurrence after the marker
    let new_line = if let Some(pos) = marker_pos {
        // Split at marker, replace first version in the portion after marker
        let (before_marker, at_and_after) = example.split_at(pos);
        let replaced_after = version_re.replacen(at_and_after, 1, replacement.as_str());
        format!("{}{}", before_marker, replaced_after)
    } else {
        // Fallback: replace first occurrence in the entire example
        version_re.replacen(example, 1, replacement.as_str()).to_string()
    };

    // Handle date substitution - replace first date if present
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").context("Failed to compile date regex")?;
    let new_line = date_re.replacen(&new_line, 1, version.date.as_str()).to_string();

    Ok(new_line)
}

/// Plan version changes for all target files.
pub fn plan_changes(version: &VersionInfo, manifest: &VersionManifest) -> Result<Vec<FileEdit>> {
    use std::collections::HashSet;
    let mut edits = Vec::new();
    let mut edited_lines: HashSet<(String, usize)> = HashSet::new();

    for target in &manifest.files {
        // Skip glob patterns (release_evidence files)
        if target.path.contains('*') {
            continue;
        }

        // Skip CHANGELOG.md - it's handled specially in release_prepare
        // (we insert a new section rather than replacing)
        if target.path.contains("CHANGELOG.md") {
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
            // Skip patterns without examples
            if pattern.example.is_none() {
                continue;
            }

            for (i, line) in lines.iter().enumerate() {
                if line.contains(&pattern.marker) {
                    // Skip if this line was already edited by another pattern
                    let key = (target.path.clone(), i + 1);
                    if edited_lines.contains(&key) {
                        continue;
                    }

                    let new_line = generate_new_line(pattern, version)?;

                    if *line != new_line {
                        edits.push(FileEdit {
                            path: target.path.clone(),
                            line_number: i + 1,
                            old_text: line.to_string(),
                            new_text: new_line,
                        });
                        edited_lines.insert(key);
                    }
                    break;
                }
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
        let v = Version::parse("3.3.5").unwrap();
        assert_eq!(v.to_string(), "3.3.5");
    }

    #[test]
    fn test_version_to_tag() {
        let v = Version::parse("3.3.5").unwrap();
        assert_eq!(v.to_tag(), "v3.3.5");
    }

    #[test]
    fn test_version_to_kernel_tag() {
        let v = Version::parse("3.3.5").unwrap();
        assert_eq!(v.to_kernel_tag(), "v3.3.5-kernel");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        let v3 = Version::parse("1.1.0").unwrap();
        let v4 = Version::parse("1.0.1").unwrap();

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

    // === A2 Tests: Manifest Loading, Plan Generation, Apply Changes ===

    #[test]
    fn test_manifest_load_from_repo() {
        // Test loading actual manifest from specs/version_manifest.yaml
        // This test should run from repo root during `cargo test -p xtask`
        let manifest = VersionManifest::load();
        assert!(manifest.is_ok(), "Should load manifest: {:?}", manifest.err());

        let manifest = manifest.unwrap();
        assert_eq!(manifest.schema_version, "1.0");
        assert!(!manifest.files.is_empty(), "Manifest should have files");

        // Check that spec_ledger.yaml is in the files list
        let has_spec_ledger = manifest.files.iter().any(|f| f.path.contains("spec_ledger.yaml"));
        assert!(has_spec_ledger, "Manifest should include spec_ledger.yaml");
    }

    #[test]
    fn test_generate_new_line_yaml_value() {
        let pattern = FilePattern {
            marker: "template_version:".to_string(),
            pattern_type: "yaml_value".to_string(),
            format: "quoted".to_string(),
            line_pattern: None,
            example: Some("  template_version: \"3.3.5\"".to_string()),
            notes: None,
        };

        let version = VersionInfo::with_date("3.3.6", "2025-12-01").unwrap();
        let new_line = generate_new_line(&pattern, &version).unwrap();

        assert_eq!(new_line, "  template_version: \"3.3.6\"");
    }

    #[test]
    fn test_generate_new_line_heading_version() {
        let pattern = FilePattern {
            marker: "# Test Service (v".to_string(),
            pattern_type: "heading_version".to_string(),
            format: "prefixed_paren".to_string(),
            line_pattern: None,
            example: Some("# Test Service (v3.3.5)".to_string()),
            notes: None,
        };

        let version = VersionInfo::with_date("3.3.6", "2025-12-01").unwrap();
        let new_line = generate_new_line(&pattern, &version).unwrap();

        assert_eq!(new_line, "# Test Service (v3.3.6)");
    }

    #[test]
    fn test_generate_new_line_kernel_suffixed() {
        let pattern = FilePattern {
            marker: "**Version:**".to_string(),
            pattern_type: "inline_version".to_string(),
            format: "kernel_suffixed".to_string(),
            line_pattern: None,
            example: Some("**Date:** 2025-11-30 | **Version:** v3.3.5-kernel".to_string()),
            notes: None,
        };

        let version = VersionInfo::with_date("3.3.6", "2025-12-01").unwrap();
        let new_line = generate_new_line(&pattern, &version).unwrap();

        assert_eq!(new_line, "**Date:** 2025-12-01 | **Version:** v3.3.6-kernel");
    }

    #[test]
    fn test_generate_new_line_with_date_substitution() {
        let pattern = FilePattern {
            marker: "**Date:**".to_string(),
            pattern_type: "inline_version".to_string(),
            format: "prefixed".to_string(),
            line_pattern: None,
            example: Some("**Date:** 2025-11-30 | **Version:** v3.3.5".to_string()),
            notes: None,
        };

        let version = VersionInfo::with_date("3.3.6", "2025-12-01").unwrap();
        let new_line = generate_new_line(&pattern, &version).unwrap();

        // Both version and date should be updated
        assert!(new_line.contains("2025-12-01"), "Date should be updated");
        assert!(new_line.contains("3.3.6"), "Version should be updated");
    }

    #[test]
    fn test_plan_changes_with_temp_files() {
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temp directory with test files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let readme_path = temp_path.join("README.md");
        let mut readme = fs::File::create(&readme_path).expect("Failed to create README");
        writeln!(readme, "# Test Service (v3.3.5)").expect("Failed to write");
        writeln!(readme, "Some content").expect("Failed to write");
        writeln!(readme, "**Template Version:** v3.3.5").expect("Failed to write");

        // Create a minimal manifest for testing
        let manifest = VersionManifest {
            schema_version: "1.0".to_string(),
            description: Some("Test manifest".to_string()),
            version_format: None,
            files: vec![VersionTarget {
                path: readme_path.to_string_lossy().to_string(),
                description: Some("Test README".to_string()),
                patterns: vec![
                    FilePattern {
                        marker: "# Test Service (v".to_string(),
                        pattern_type: "heading_version".to_string(),
                        format: "prefixed_paren".to_string(),
                        line_pattern: None,
                        example: Some("# Test Service (v3.3.5)".to_string()),
                        notes: None,
                    },
                    FilePattern {
                        marker: "**Template Version:**".to_string(),
                        pattern_type: "inline_version".to_string(),
                        format: "prefixed".to_string(),
                        line_pattern: None,
                        example: Some("**Template Version:** v3.3.5".to_string()),
                        notes: None,
                    },
                ],
                required: true,
                priority: 1,
                notes: None,
            }],
        };

        let version = VersionInfo::with_date("3.3.6", "2025-12-01").unwrap();
        let edits = plan_changes(&version, &manifest).expect("Should plan changes");

        assert_eq!(edits.len(), 2, "Should plan 2 edits");
        assert!(edits[0].new_text.contains("3.3.6"), "First edit should have new version");
        assert!(edits[1].new_text.contains("3.3.6"), "Second edit should have new version");
    }

    #[test]
    fn test_apply_changes_dry_run() {
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temp directory with test files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let readme_path = temp_path.join("README.md");
        let mut readme = fs::File::create(&readme_path).expect("Failed to create README");
        writeln!(readme, "version: 3.3.5").expect("Failed to write");

        let original_content =
            fs::read_to_string(&readme_path).expect("Failed to read original content");

        // Create edit
        let edits = vec![FileEdit {
            path: readme_path.to_string_lossy().to_string(),
            line_number: 1,
            old_text: "version: 3.3.5".to_string(),
            new_text: "version: 3.3.6".to_string(),
        }];

        // Apply with dry_run = true
        apply_changes(&edits, true).expect("Dry run should succeed");

        // Verify file was NOT modified
        let after_content = fs::read_to_string(&readme_path).expect("Failed to read after dry run");
        assert_eq!(original_content, after_content, "File should not be modified in dry run");
    }

    #[test]
    fn test_apply_changes_actual() {
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temp directory with test files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let readme_path = temp_path.join("README.md");
        let mut readme = fs::File::create(&readme_path).expect("Failed to create README");
        writeln!(readme, "version: 3.3.5").expect("Failed to write");

        // Create edit
        let edits = vec![FileEdit {
            path: readme_path.to_string_lossy().to_string(),
            line_number: 1,
            old_text: "version: 3.3.5".to_string(),
            new_text: "version: 3.3.6".to_string(),
        }];

        // Apply with dry_run = false
        apply_changes(&edits, false).expect("Apply should succeed");

        // Verify file was modified
        let after_content = fs::read_to_string(&readme_path).expect("Failed to read after apply");
        assert!(after_content.contains("3.3.6"), "File should contain new version");
        assert!(!after_content.contains("3.3.5"), "File should not contain old version");
    }

    #[test]
    fn test_apply_changes_atomicity_on_mismatch() {
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temp directory with test files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let readme_path = temp_path.join("README.md");
        let mut readme = fs::File::create(&readme_path).expect("Failed to create README");
        writeln!(readme, "version: 3.3.4").expect("Failed to write"); // Note: different version

        let original_content =
            fs::read_to_string(&readme_path).expect("Failed to read original content");

        // Create edit with mismatched old_text (expects 3.3.5 but file has 3.3.4)
        let edits = vec![FileEdit {
            path: readme_path.to_string_lossy().to_string(),
            line_number: 1,
            old_text: "version: 3.3.5".to_string(), // This doesn't match!
            new_text: "version: 3.3.6".to_string(),
        }];

        // Apply should fail due to mismatch
        let result = apply_changes(&edits, false);
        assert!(result.is_err(), "Should fail on text mismatch");

        // Verify file was NOT modified (atomicity)
        let after_content =
            fs::read_to_string(&readme_path).expect("Failed to read after failed apply");
        assert_eq!(original_content, after_content, "File should not be modified on failure");
    }
}
