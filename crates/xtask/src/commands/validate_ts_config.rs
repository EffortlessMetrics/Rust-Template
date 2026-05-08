use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSummary {
    pub files_checked: usize,
    pub violations: usize,
    pub advisories: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding {
    path: PathBuf,
    severity: Severity,
    message: &'static str,
    fix: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Violation,
    Advisory,
}

pub fn run() -> Result<()> {
    println!("Validating TypeScript configuration standards...");
    println!();

    let summary = validate_workspace(Path::new("."), true)?;

    println!();
    if summary.violations > 0 {
        println!(
            "{}",
            format!("Found {} TypeScript config violation(s)", summary.violations).red()
        );
        println!();
        println!("TypeScript configuration standards for this repo:");
        println!("  - module: \"NodeNext\"");
        println!("  - moduleResolution: \"NodeNext\"");
        println!("  - No ignoreDeprecations flags");
        println!();
        println!("See docs/how-to/implement-backstage-plugin.md for details.");
        anyhow::bail!("TypeScript configuration validation failed");
    }

    println!(
        "{}",
        format!(
            "✓ All TypeScript configurations pass validation ({} file(s) checked, {} advisory warning(s))",
            summary.files_checked, summary.advisories
        )
        .green()
    );
    Ok(())
}

pub fn validate_workspace(root: &Path, emit_findings: bool) -> Result<ValidationSummary> {
    let mut summary = ValidationSummary { files_checked: 0, violations: 0, advisories: 0 };

    for tsconfig in find_tsconfig_files(root) {
        summary.files_checked += 1;
        let content = std::fs::read_to_string(&tsconfig)
            .with_context(|| format!("failed to read {}", tsconfig.display()))?;
        let findings = validate_content(&tsconfig, &content)?;

        for finding in findings {
            match finding.severity {
                Severity::Violation => {
                    summary.violations += 1;
                    if emit_findings {
                        println!("{} {}", "✗".red(), display_path(root, &finding.path).red());
                    }
                }
                Severity::Advisory => {
                    summary.advisories += 1;
                    if emit_findings {
                        println!("{} {}", "⚠".yellow(), display_path(root, &finding.path).yellow());
                    }
                }
            }
            if emit_findings {
                println!("  - {}", finding.message);
                if let Some(fix) = finding.fix {
                    println!("  - Fix: {}", fix);
                }
            }
        }
    }

    Ok(summary)
}

fn find_tsconfig_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !matches!(name.as_ref(), ".git" | "node_modules" | "target")
        })
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "tsconfig.json")
        .map(|entry| entry.path().to_path_buf())
        .collect();
    files.sort();
    files
}

fn validate_content(path: &Path, content: &str) -> Result<Vec<Finding>> {
    let deprecated_module_resolution = Regex::new(r#""moduleResolution"\s*:\s*"(node10|node)""#)?;
    let any_module_resolution = Regex::new(r#""moduleResolution"\s*:"#)?;
    let node_next_module_resolution = Regex::new(r#""moduleResolution"\s*:\s*"NodeNext""#)?;
    let ignore_deprecations = Regex::new(r#""ignoreDeprecations""#)?;

    let mut findings = Vec::new();

    if deprecated_module_resolution.is_match(content) {
        findings.push(Finding {
            path: path.to_path_buf(),
            severity: Severity::Violation,
            message: "Uses deprecated moduleResolution (node10 or node)",
            fix: Some("Use \"moduleResolution\": \"NodeNext\""),
        });
    }

    if ignore_deprecations.is_match(content) {
        findings.push(Finding {
            path: path.to_path_buf(),
            severity: Severity::Violation,
            message: "Contains ignoreDeprecations flag",
            fix: Some("Remove ignoreDeprecations and address warnings"),
        });
    }

    if any_module_resolution.is_match(content) && !node_next_module_resolution.is_match(content) {
        findings.push(Finding {
            path: path.to_path_buf(),
            severity: Severity::Advisory,
            message: "moduleResolution is not NodeNext (advisory)",
            fix: None,
        });
    }

    Ok(findings)
}

fn display_path<'a>(root: &Path, path: &'a Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_node_next_config() {
        let findings = validate_content(
            Path::new("tsconfig.json"),
            r#"{ "compilerOptions": { "moduleResolution": "NodeNext" } }"#,
        )
        .unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn rejects_deprecated_module_resolution_and_ignore_deprecations() {
        let findings = validate_content(
            Path::new("tsconfig.json"),
            r#"{ "compilerOptions": { "moduleResolution": "node", "ignoreDeprecations": "5.0" } }"#,
        )
        .unwrap();
        assert_eq!(2, findings.iter().filter(|f| f.severity == Severity::Violation).count());
    }

    #[test]
    fn warns_on_non_node_next_module_resolution() {
        let findings = validate_content(
            Path::new("tsconfig.json"),
            r#"{ "compilerOptions": { "moduleResolution": "bundler" } }"#,
        )
        .unwrap();
        assert_eq!(
            vec![Severity::Advisory],
            findings.iter().map(|f| f.severity).collect::<Vec<_>>()
        );
    }
}
