//! Version consistency validation command.
//!
//! Validates that all version references across the codebase match.
//! This prevents version drift between Cargo.toml, spec_ledger.yaml, README.md, etc.

use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Files and patterns to check for version consistency.
struct VersionSource {
    path: &'static str,
    pattern: &'static str,
    description: &'static str,
}

const VERSION_SOURCES: &[VersionSource] = &[
    VersionSource {
        path: "Cargo.toml",
        pattern: r#"^version\s*=\s*"(\d+\.\d+\.\d+)""#,
        description: "workspace Cargo.toml version",
    },
    VersionSource {
        path: "specs/spec_ledger.yaml",
        pattern: r#"template_version:\s*"(\d+\.\d+\.\d+)""#,
        description: "spec_ledger template_version",
    },
    VersionSource {
        path: "README.md",
        pattern: r#"^#\s+Test Service\s+\(v(\d+\.\d+\.\d+)\)"#,
        description: "README.md title version",
    },
    VersionSource {
        path: "CLAUDE.md",
        pattern: r#"^\*\*Template Version:\*\*\s+v?(\d+\.\d+\.\d+)"#,
        description: "CLAUDE.md template version",
    },
];

#[derive(Default)]
pub struct VersionCheckArgs {
    /// Output in JSON format
    pub json: bool,
}

/// Result of checking a single source
#[derive(Debug)]
struct SourceResult {
    path: String,
    description: String,
    version: Option<String>,
    error: Option<String>,
}

pub fn run(args: VersionCheckArgs) -> Result<()> {
    let mut results: Vec<SourceResult> = Vec::new();
    let mut canonical_version: Option<String> = None;

    // Extract versions from all sources
    for source in VERSION_SOURCES {
        let result = extract_version(source);

        // First source (Cargo.toml) is canonical
        if canonical_version.is_none()
            && let Some(ref v) = result.version
        {
            canonical_version = Some(v.clone());
        }

        results.push(result);
    }

    if args.json {
        print_json(&results, &canonical_version)?;
    } else {
        print_human(&results, &canonical_version)?;
    }

    // Check for mismatches
    let canonical = canonical_version.as_deref().unwrap_or("unknown");
    let mismatches: Vec<&SourceResult> = results
        .iter()
        .filter(|r| r.version.as_deref() != Some(canonical) && r.error.is_none())
        .collect();

    let errors: Vec<&SourceResult> = results.iter().filter(|r| r.error.is_some()).collect();

    if !mismatches.is_empty() || !errors.is_empty() {
        if !args.json {
            println!();
            if !mismatches.is_empty() {
                eprintln!(
                    "{} Version mismatch detected! {} file(s) have different versions.",
                    "✗".red().bold(),
                    mismatches.len()
                );
                eprintln!("  Canonical version (from Cargo.toml): {}", canonical.green());
                for m in &mismatches {
                    eprintln!(
                        "  {} {} has version {}",
                        "→".yellow(),
                        m.path,
                        m.version.as_deref().unwrap_or("unknown").red()
                    );
                }
                eprintln!();
                eprintln!("Fix with: cargo xtask release-prepare {}", canonical);
            }
            if !errors.is_empty() {
                eprintln!("{} {} file(s) could not be checked:", "⚠".yellow().bold(), errors.len());
                for e in &errors {
                    eprintln!(
                        "  {} {}: {}",
                        "→".yellow(),
                        e.path,
                        e.error.as_deref().unwrap_or("unknown error")
                    );
                }
            }
        }
        anyhow::bail!("Version consistency check failed");
    }

    if !args.json {
        println!();
        println!("{} All version references match: {}", "✓".green().bold(), canonical.green());
    }

    Ok(())
}

fn extract_version(source: &VersionSource) -> SourceResult {
    let path = Path::new(source.path);

    if !path.exists() {
        return SourceResult {
            path: source.path.to_string(),
            description: source.description.to_string(),
            version: None,
            error: Some("File not found".to_string()),
        };
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult {
                path: source.path.to_string(),
                description: source.description.to_string(),
                version: None,
                error: Some(format!("Failed to read: {}", e)),
            };
        }
    };

    let re = match Regex::new(source.pattern) {
        Ok(r) => r,
        Err(e) => {
            return SourceResult {
                path: source.path.to_string(),
                description: source.description.to_string(),
                version: None,
                error: Some(format!("Invalid regex: {}", e)),
            };
        }
    };

    // Search line by line for multiline patterns
    for line in content.lines() {
        if let Some(caps) = re.captures(line)
            && let Some(version) = caps.get(1)
        {
            return SourceResult {
                path: source.path.to_string(),
                description: source.description.to_string(),
                version: Some(version.as_str().to_string()),
                error: None,
            };
        }
    }

    SourceResult {
        path: source.path.to_string(),
        description: source.description.to_string(),
        version: None,
        error: Some("Version pattern not found".to_string()),
    }
}

fn print_human(results: &[SourceResult], canonical: &Option<String>) -> Result<()> {
    println!();
    println!("{}", "Version Consistency Check".bold());
    println!("{}", "=========================".blue());
    println!();

    let canonical_ver = canonical.as_deref().unwrap_or("unknown");
    println!("  {} {} (canonical)", "Cargo.toml".bold(), canonical_ver.green());

    for result in results.iter().skip(1) {
        let status = match (&result.version, &result.error) {
            (Some(v), _) if v == canonical_ver => "✓".green(),
            (Some(_), _) => "✗".red(),
            (None, Some(_)) => "⚠".yellow(),
            (None, None) => "?".dimmed(),
        };

        let version_str = match (&result.version, &result.error) {
            (Some(v), _) => v.clone(),
            (None, Some(e)) => format!("({}) {}", "error".red(), e),
            (None, None) => "not found".to_string(),
        };

        println!("  {} {} {}", status, result.path, version_str.dimmed());
    }

    Ok(())
}

fn print_json(results: &[SourceResult], canonical: &Option<String>) -> Result<()> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct JsonOutput {
        canonical_version: Option<String>,
        consistent: bool,
        sources: Vec<JsonSource>,
    }

    #[derive(Serialize)]
    struct JsonSource {
        path: String,
        description: String,
        version: Option<String>,
        matches_canonical: bool,
        error: Option<String>,
    }

    let canonical_ver = canonical.as_deref();
    let sources: Vec<JsonSource> = results
        .iter()
        .map(|r| JsonSource {
            path: r.path.clone(),
            description: r.description.clone(),
            version: r.version.clone(),
            matches_canonical: r.version.as_deref() == canonical_ver,
            error: r.error.clone(),
        })
        .collect();

    let consistent = sources.iter().all(|s| s.matches_canonical && s.error.is_none());

    let output = JsonOutput { canonical_version: canonical.clone(), consistent, sources };

    let json = serde_json::to_string_pretty(&output)
        .context("Failed to serialize version check output to JSON")?;
    println!("{}", json);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_extraction_patterns() {
        // Test Cargo.toml pattern
        let cargo_pattern = r#"^version\s*=\s*"(\d+\.\d+\.\d+)""#;
        let re = Regex::new(cargo_pattern).unwrap();
        assert!(re.captures(r#"version = "3.3.12""#).is_some());

        // Test spec_ledger pattern
        let spec_pattern = r#"template_version:\s*"(\d+\.\d+\.\d+)""#;
        let re = Regex::new(spec_pattern).unwrap();
        assert!(re.captures(r#"  template_version: "3.3.12""#).is_some());

        // Test README pattern
        let readme_pattern = r#"^#\s+Test Service\s+\(v(\d+\.\d+\.\d+)\)"#;
        let re = Regex::new(readme_pattern).unwrap();
        assert!(re.captures("# Test Service (v3.3.12)").is_some());

        // Test CLAUDE.md pattern
        let claude_pattern = r#"^\*\*Template Version:\*\*\s+v?(\d+\.\d+\.\d+)"#;
        let re = Regex::new(claude_pattern).unwrap();
        assert!(re.captures("**Template Version:** v3.3.12").is_some());
    }

    #[test]
    fn test_version_check_runs_from_repo() {
        // This test validates that version checking works from the actual repo
        // Skip if not running from repo root
        if !Path::new("Cargo.toml").exists() {
            return;
        }

        let args = VersionCheckArgs { json: false };
        // We just check it doesn't panic; actual version consistency depends on repo state
        let _ = run(args);
    }
}
