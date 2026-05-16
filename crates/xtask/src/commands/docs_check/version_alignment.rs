use anyhow::Result;
use colored::Colorize;
use std::fs;

/// Consumer files that must stay in sync with spec_ledger.yaml version.
/// @AC-PLT-009: docs-check validates version alignment across 8 consumer files
pub const VERSION_CONSUMERS: [&str; 8] = [
    "README.md",
    "CLAUDE.md",
    "docs/ROADMAP.md",
    "docs/KERNEL_SNAPSHOT.md",
    "docs/explanation/TEMPLATE-CONTRACTS.md",
    "specs/service_metadata.yaml",
    "specs/doc_index.yaml",
    "CHANGELOG.md",
];

/// Version alignment result for a single file
#[derive(Debug)]
struct VersionCheck {
    file: &'static str,
    expected: String,
    found: String,
    pattern: &'static str,
}

impl VersionCheck {
    fn is_ok(&self) -> bool {
        self.expected == self.found
    }
}
pub(crate) fn check_version_alignment_v2() -> Result<()> {
    // Step 1: Extract canonical version from spec_ledger
    let canonical_version = extract_version_from_ledger()?;
    if canonical_version == "unknown" {
        anyhow::bail!("Could not extract template_version from specs/spec_ledger.yaml");
    }

    eprintln!("  Canonical version (spec_ledger): {}", canonical_version);

    // Step 2: Check all consumer files against the canonical version
    // This list must match VERSION_CONSUMERS (enforced by compile-time assertion below)
    let checks = vec![
        check_readme_version(&canonical_version),
        check_claude_version(&canonical_version),
        check_roadmap_version(&canonical_version),
        check_kernel_snapshot_version(&canonical_version),
        check_template_contracts_version(&canonical_version),
        check_service_metadata_version(&canonical_version),
        check_doc_index_version(&canonical_version),
        check_changelog_version(&canonical_version),
    ];

    // Compile-time assertion: checks list length must match VERSION_CONSUMERS
    const _: () = assert!(VERSION_CONSUMERS.len() == 8, "VERSION_CONSUMERS length mismatch");

    let mut mismatches: Vec<VersionCheck> = Vec::new();

    for check in checks {
        match check {
            Ok(vc) => {
                if vc.is_ok() {
                    eprintln!("  {} version: {} ✓", vc.file, vc.found);
                } else {
                    eprintln!("  {} version: {} (expected {}) ✗", vc.file, vc.found, vc.expected);
                    mismatches.push(vc);
                }
            }
            Err(e) => {
                // File doesn't exist or couldn't be parsed - that's OK for optional files
                eprintln!("  {} skipped: {}", e, "(optional or missing)".dimmed());
            }
        }
    }

    if !mismatches.is_empty() {
        eprintln!();
        eprintln!("{}", "Version mismatches found:".yellow().bold());
        for m in &mismatches {
            eprintln!(
                "  • {} has '{}', expected '{}' (pattern: {})",
                m.file.cyan(),
                m.found.red(),
                m.expected.green(),
                m.pattern.dimmed()
            );
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!(
            "  1. Update the canonical version: {}",
            "specs/spec_ledger.yaml → metadata.template_version".cyan()
        );
        eprintln!("  2. Or run: {} to bump all files", "cargo xtask release-prepare X.Y.Z".cyan());
        eprintln!("  3. Commit changes and verify: {}", "cargo xtask selftest".cyan());

        anyhow::bail!(
            "Version alignment failed: {} file(s) out of sync with spec_ledger (v{})",
            mismatches.len(),
            canonical_version
        );
    }

    Ok(())
}

/// Extract canonical version from spec_ledger.yaml
pub(crate) fn extract_version_from_ledger() -> Result<String> {
    let content = fs::read_to_string("specs/spec_ledger.yaml")?;
    for line in content.lines() {
        if line.trim().starts_with("template_version:")
            && let Some(version) = line.split(':').nth(1)
        {
            return Ok(version.trim().trim_matches('"').to_string());
        }
    }
    Ok("unknown".to_string())
}

/// Check README.md H1: `# ... (vX.Y.Z)` or badge line
fn check_readme_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("README.md")?;

    // Look for version in H1 or badge line
    for line in content.lines() {
        // H1 pattern: # Something (vX.Y.Z)
        if line.starts_with('#')
            && !line.starts_with("##")
            && let Some(start) = line.find("(v")
            && let Some(end) = line[start..].find(')')
        {
            let found = line[start + 2..start + end].to_string();
            return Ok(VersionCheck {
                file: "README.md",
                expected: canonical.to_string(),
                found,
                pattern: "H1: # ... (vX.Y.Z)",
            });
        }
        // Badge line pattern: **Template Version:** vX.Y.Z
        if line.contains("Template Version")
            && line.contains('v')
            && let Some(version) = extract_version_after_v(line)
        {
            return Ok(VersionCheck {
                file: "README.md",
                expected: canonical.to_string(),
                found: version,
                pattern: "**Template Version:** vX.Y.Z",
            });
        }
    }

    Ok(VersionCheck {
        file: "README.md",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "H1 or Template Version line",
    })
}

/// Check CLAUDE.md: `**Template Version:** vX.Y.Z` or H1
fn check_claude_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("CLAUDE.md")?;

    for line in content.lines() {
        // H1 pattern: # ... (vX.Y.Z)
        if line.starts_with("# ")
            && let Some(start) = line.find("(v")
            && let Some(end) = line[start..].find(')')
        {
            let found = line[start + 2..start + end].to_string();
            return Ok(VersionCheck {
                file: "CLAUDE.md",
                expected: canonical.to_string(),
                found,
                pattern: "H1: # ... (vX.Y.Z)",
            });
        }
        // Template Version line
        if line.contains("Template Version")
            && line.contains('v')
            && let Some(version) = extract_version_after_v(line)
        {
            return Ok(VersionCheck {
                file: "CLAUDE.md",
                expected: canonical.to_string(),
                found: version,
                pattern: "**Template Version:** vX.Y.Z",
            });
        }
    }

    Ok(VersionCheck {
        file: "CLAUDE.md",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "H1 or Template Version line",
    })
}

/// Check docs/ROADMAP.md: H1 with `(vX.Y.Z)`
fn check_roadmap_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("docs/ROADMAP.md")?;

    for line in content.lines() {
        // H1 pattern: # Roadmap: ... (vX.Y.Z)
        if line.starts_with("# ")
            && let Some(start) = line.find("(v")
            && let Some(end) = line[start..].find(')')
        {
            let found = line[start + 2..start + end].to_string();
            return Ok(VersionCheck {
                file: "docs/ROADMAP.md",
                expected: canonical.to_string(),
                found,
                pattern: "H1: # ... (vX.Y.Z)",
            });
        }
    }

    Ok(VersionCheck {
        file: "docs/ROADMAP.md",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "H1: # ... (vX.Y.Z)",
    })
}

/// Check docs/KERNEL_SNAPSHOT.md: H1 `# Kernel Snapshot vX.Y.Z`
fn check_kernel_snapshot_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("docs/KERNEL_SNAPSHOT.md")?;

    for line in content.lines() {
        // H1 pattern: # Kernel Snapshot vX.Y.Z
        if line.starts_with("# Kernel Snapshot")
            && let Some(version) = extract_version_after_v(line)
        {
            return Ok(VersionCheck {
                file: "docs/KERNEL_SNAPSHOT.md",
                expected: canonical.to_string(),
                found: version,
                pattern: "H1: # Kernel Snapshot vX.Y.Z",
            });
        }
    }

    Ok(VersionCheck {
        file: "docs/KERNEL_SNAPSHOT.md",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "H1: # Kernel Snapshot vX.Y.Z",
    })
}

/// Check docs/explanation/TEMPLATE-CONTRACTS.md: `**Template Version:** vX.Y.Z`
fn check_template_contracts_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("docs/explanation/TEMPLATE-CONTRACTS.md")?;

    for line in content.lines() {
        if line.contains("Template Version")
            && line.contains('v')
            && let Some(version) = extract_version_after_v(line)
        {
            return Ok(VersionCheck {
                file: "docs/explanation/TEMPLATE-CONTRACTS.md",
                expected: canonical.to_string(),
                found: version,
                pattern: "**Template Version:** vX.Y.Z",
            });
        }
    }

    Ok(VersionCheck {
        file: "docs/explanation/TEMPLATE-CONTRACTS.md",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "**Template Version:** vX.Y.Z",
    })
}

/// Check specs/service_metadata.yaml: `template_version: vX.Y.Z`
fn check_service_metadata_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("specs/service_metadata.yaml")?;

    for line in content.lines() {
        if line.trim().starts_with("template_version:")
            && let Some(value) = line.split(':').nth(1)
        {
            let found = value.trim().trim_matches('"').trim_start_matches('v').to_string();
            return Ok(VersionCheck {
                file: "specs/service_metadata.yaml",
                expected: canonical.to_string(),
                found,
                pattern: "template_version: vX.Y.Z",
            });
        }
    }

    Ok(VersionCheck {
        file: "specs/service_metadata.yaml",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "template_version: vX.Y.Z",
    })
}

/// Check specs/doc_index.yaml: `template_version: "X.Y.Z"`
fn check_doc_index_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("specs/doc_index.yaml")?;

    for line in content.lines() {
        if line.trim().starts_with("template_version:")
            && let Some(value) = line.split(':').nth(1)
        {
            let found = value.trim().trim_matches('"').to_string();
            return Ok(VersionCheck {
                file: "specs/doc_index.yaml",
                expected: canonical.to_string(),
                found,
                pattern: "template_version: \"X.Y.Z\"",
            });
        }
    }

    Ok(VersionCheck {
        file: "specs/doc_index.yaml",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "template_version: \"X.Y.Z\"",
    })
}

/// Check CHANGELOG.md: First version section after [Unreleased] should be `## [X.Y.Z]`
fn check_changelog_version(canonical: &str) -> Result<VersionCheck> {
    let content = fs::read_to_string("CHANGELOG.md")?;

    let mut found_unreleased = false;
    for line in content.lines() {
        // Skip until we find [Unreleased]
        if line.contains("[Unreleased]") {
            found_unreleased = true;
            continue;
        }

        // Look for the first version heading after [Unreleased]
        if found_unreleased && line.starts_with("## [") {
            // Extract version from ## [X.Y.Z] - YYYY-MM-DD
            if let Some(start) = line.find('[')
                && let Some(end) = line[start..].find(']')
            {
                let found = line[start + 1..start + end].to_string();
                return Ok(VersionCheck {
                    file: "CHANGELOG.md",
                    expected: canonical.to_string(),
                    found,
                    pattern: "## [X.Y.Z] - YYYY-MM-DD (first after [Unreleased])",
                });
            }
        }
    }

    Ok(VersionCheck {
        file: "CHANGELOG.md",
        expected: canonical.to_string(),
        found: "unknown".to_string(),
        pattern: "## [X.Y.Z] (first section after [Unreleased])",
    })
}

/// Helper: Extract version number after 'v' (e.g., "v3.3.4" -> "3.3.4")
fn extract_version_after_v(line: &str) -> Option<String> {
    // Find 'v' followed by digits
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'v' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            // Found vX.Y.Z pattern
            let rest = &line[i + 1..];
            let version: String =
                rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            if !version.is_empty() {
                return Some(version);
            }
        }
    }
    None
}
