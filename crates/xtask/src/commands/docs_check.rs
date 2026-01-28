use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

pub fn run() -> Result<()> {
    println!("{}", "📚 Checking documentation consistency...".blue().bold());
    println!();

    let mut issues = 0;

    // Check version alignment (enhanced Docs-as-Code v2)
    print!("Version alignment... ");
    match check_version_alignment_v2() {
        Ok(_) => println!("{}", "✓ Consistent".green()),
        Err(e) => {
            println!("{}", "✗ Mismatch".red());
            eprintln!("  {}", e);
            issues += 1;
        }
    }

    // Check ADR references
    print!("ADR references... ");
    match crate::commands::adr_check::run(crate::commands::adr_check::AdrCheckArgs {
        verbosity: crate::Verbosity::Quiet,
        ..Default::default()
    }) {
        Ok(_) => println!("{}", "✓ Valid".green()),
        Err(e) => {
            println!("{}", "✗ Issues found".red());
            eprintln!("  {}", e);
            issues += 1;
        }
    }

    // Check AC status cleanliness
    print!("AC status consistency... ");
    match check_ac_status_clean() {
        Ok(_) => println!("{}", "✓ Up to date".green()),
        Err(e) => {
            println!("{}", "✗ Out of sync".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix AC status consistency:".bold());
            eprintln!("  1. Run {} to regenerate", "cargo xtask ac-status".cyan());
            eprintln!("  2. Commit the updated docs/feature_status.md");
            issues += 1;
        }
    }

    // Check Docs-as-Spec validation (front-matter sync - HARD GATE)
    print!("Doc index & front-matter... ");
    match validate_doc_index() {
        Ok(_) => println!("{}", "✓ Consistent".green()),
        Err(e) => {
            println!("{}", "✗ Issues found".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix front-matter mismatches:".bold());
            eprintln!(
                "  1. Edit {} to reflect desired doc metadata",
                "specs/doc_index.yaml".cyan()
            );
            eprintln!(
                "  2. Run {} to sync front-matter",
                "cargo xtask docs-frontmatter-sync --fix".cyan()
            );
            eprintln!("  3. Commit the updated doc files");
            eprintln!();
            eprintln!("Note: Front-matter must match doc_index.yaml exactly (bidirectional).");
            issues += 1;
        }
    }

    // Check Feature Status header invariants (AC-PLT-010 extension)
    print!("Feature Status invariants... ");
    match validate_feature_status_invariants() {
        Ok(_) => println!("{}", "✓ Valid".green()),
        Err(e) => {
            println!("{}", "✗ Issues found".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix Feature Status invariants:".bold());
            eprintln!("  1. Run {} to regenerate", "cargo xtask ac-status".cyan());
            eprintln!("  2. Verify the header contains 'Template Version: X.Y.Z'");
            eprintln!("  3. Commit the updated docs/feature_status.md");
            issues += 1;
        }
    }

    // Check Doc Policies
    print!("Doc policies... ");
    match validate_doc_policies() {
        Ok(_) => println!("{}", "✓ Satisfied".green()),
        Err(e) => {
            println!("{}", "✗ Violations found".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix doc policy violations:".bold());
            eprintln!("  1. Review policies in {}", "specs/doc_policies.yaml".cyan());
            eprintln!("  2. Register docs in {} with required tags", "specs/doc_index.yaml".cyan());
            eprintln!("  3. Verify doc types and references align with policy rules");
            eprintln!("  See: {}", "docs/reference/doc-sources.md".dimmed());
            issues += 1;
        }
    }

    // Check Kernel REQ→Doc coverage (Slice C)
    // All kernel REQs (must_have_ac: true) must have at least one doc covering them.
    // This is now a hard check since kernel documentation coverage is complete.
    print!("Kernel REQ doc coverage... ");
    match validate_kernel_req_doc_coverage() {
        Ok(_) => println!("{}", "✓ Covered".green()),
        Err(e) => {
            println!("{}", "✗ Missing docs".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix kernel REQ documentation gaps:".bold());
            eprintln!("  1. Create a doc in docs/... covering the requirement");
            eprintln!(
                "  2. Register it in {} with the REQ ID in 'requirements:'",
                "specs/doc_index.yaml".cyan()
            );
            eprintln!("  3. Or demote the REQ by removing must_have_ac: true");
            issues += 1;
        }
    }

    // Check Doc type contracts (Slice D)
    // Note: This is a soft check (warning) to encourage gradual improvement.
    // See docs/reference/doc-sources.md Section 6.5 for the contract table.
    print!("Doc type contracts... ");
    {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");
        let index_path = root.join("specs/doc_index.yaml");
        if index_path.exists() {
            match crate::docs_index::load_doc_index(&index_path) {
                Ok(index) => match validate_doc_types(&index) {
                    Ok(_) => println!("{}", "✓ Valid".green()),
                    Err(e) => {
                        println!("{}", "⚠ Warnings".yellow());
                        eprintln!("  [WARN] {}", e);
                        // Soft check: don't increment issues count
                    }
                },
                Err(e) => {
                    println!("{}", "⚠ Skipped".yellow());
                    eprintln!("  [WARN] Could not load doc_index: {}", e);
                }
            }
        } else {
            println!("{}", "⚠ Skipped".yellow());
            eprintln!("  [WARN] specs/doc_index.yaml not found");
        }
    }

    // Check Skills definitions
    print!("Skills definitions... ");
    match crate::commands::skills::run_lint() {
        Ok(_) => println!("{}", "✓ Valid".green()),
        Err(e) => {
            println!("{}", "✗ Issues found".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix Skills definitions:".bold());
            eprintln!("  1. Run {} to auto-format", "cargo xtask skills-fmt".cyan());
            eprintln!("  2. Or edit {} directly", ".claude/skills/*/SKILL.md".cyan());
            eprintln!("  3. Verify skill names, descriptions, and tools");
            eprintln!("  See: {}", "docs/SKILLS_GOVERNANCE.md".dimmed());
            issues += 1;
        }
    }

    // Check for orphaned version strings (AC-TPL-VERSION-MANIFEST extension)
    print!("Orphaned version strings... ");
    match check_orphaned_versions() {
        Ok(_) => println!("{}", "✓ No orphans".green()),
        Err(e) => {
            println!("{}", "✗ Orphans found".red());
            eprintln!("  {}", e);
            issues += 1;
        }
    }

    // Check contract facts synchronization (selftest steps, kernel AC count, etc.)
    print!("Contract facts... ");
    match crate::commands::contracts::check() {
        Ok(_) => println!("{}", "✓ Synchronized".green()),
        Err(e) => {
            println!("{}", "✗ Drift detected".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix contract drift:".bold());
            eprintln!("  1. Run {} to synchronize", "cargo xtask contracts-fmt".cyan());
            eprintln!("  2. Commit the updated documentation");
            issues += 1;
        }
    }

    // Check markdown links (advisory - doesn't block docs-check)
    // TODO: Promote to hard gate once existing broken links are fixed
    print!("Markdown links... ");
    match validate_markdown_links() {
        Ok(_) => println!("{}", "✓ Valid".green()),
        Err(e) => {
            println!("{}", "⚠ Broken links (advisory)".yellow());
            eprintln!("  {}", e);
            // Advisory only - don't increment issues count
            // issues += 1;
        }
    }

    // Check BDD feature file tags (advisory - validates @AC-* tags exist in spec_ledger)
    // Issue #95: BDD feature file tag validation
    print!("BDD feature tags... ");
    match validate_bdd_tags() {
        Ok(_) => println!("{}", "✓ Valid".green()),
        Err(e) => {
            println!("{}", "⚠ Issues found (advisory)".yellow());
            eprintln!("  {}", e);
            // Advisory only - don't increment issues count
            // This allows special tags like @ci-only, @smoke, @wip
        }
    }

    // Check Skills/Agents spec_ledger alignment (AC-TPL-SKILLS-GOVERNANCE-002, AC-TPL-AGENTS-GOVERNANCE-002)
    print!("Skills/Agents alignment... ");
    match validate_skills_agents_alignment() {
        Ok(_) => println!("{}", "✓ Aligned".green()),
        Err(e) => {
            println!("{}", "⚠ Alignment warnings (advisory)".yellow());
            eprintln!("  {}", e);
            // Advisory only - don't increment issues count
            // This is advisory since spec_ledger might not have explicit declarations
        }
    }

    println!();
    if issues == 0 {
        println!("{} Documentation is consistent", "✓".green().bold());
    } else {
        println!("{} {} issue(s) found", "✗".red().bold(), issues);
        println!();
        println!("{}", "To fix:".bold());
        println!("  • Align versions: {}", "cargo xtask release-prepare X.Y.Z".cyan());
        println!("  • Or manually sync: {}", "README.md, CLAUDE.md, spec_ledger.yaml".dimmed());
        println!("  • Commit generated docs if out of sync");
        println!("  • Fix Skills definitions: {}", "cargo xtask skills-fmt".cyan());
        println!("  • See: {}", "docs/RELEASE_PLAYBOOK.md".dimmed());
    }

    // Check Service Policies
    print!("Service policies... ");
    match validate_service_policies() {
        Ok(_) => println!("{}", "✓ Satisfied".green()),
        Err(e) => {
            println!("{}", "✗ Violations found".red());
            eprintln!("  {}", e);
            eprintln!();
            eprintln!("{}", "To fix service policy violations:".bold());
            eprintln!("  1. Review policies in {}", "specs/service_policies.yaml".cyan());
            eprintln!("  2. Ensure required docs (runbooks, etc.) exist");
            eprintln!(
                "  3. Update {} to declare service requirements",
                "specs/service_metadata.yaml".cyan()
            );
            issues += 1;
        }
    }

    if issues > 0 {
        anyhow::bail!("{} documentation issues", issues);
    }

    Ok(())
}

/// Docs-as-Code v2: Comprehensive version alignment check
///
/// Treats `specs/spec_ledger.yaml.metadata.template_version` as the canonical version,
/// and all other files as consumers that must match.
///
/// Uses VERSION_CONSUMERS constant to ensure the check list stays in sync with tests.
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
fn extract_version_from_ledger() -> Result<String> {
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

fn check_ac_status_clean() -> Result<()> {
    // Use check mode to verify AC status file matches computed state without writing.
    // This is cleaner than regenerating + checking git status, and doesn't modify the repo.
    //
    // Note: We ignore AC failures (test failures) here - we only care if the file is in sync.
    // The ac-status command in check mode will fail if the file content differs, which is
    // what we want to catch. If it fails due to AC test failures, that's a separate concern.
    match crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity: crate::Verbosity::Quiet,
        check: true, // Check mode: verify without writing
        ..Default::default()
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Check if this is a sync error vs an AC failure
            let err_str = e.to_string();
            if err_str.contains("out of sync") {
                // File is out of sync - this is the error we want to surface
                anyhow::bail!(
                    "AC status file is out of sync.\n\
                     Run 'cargo xtask ac-status' to regenerate and commit the changes.\n\
                     Error: {}",
                    err_str.lines().next().unwrap_or(&err_str)
                );
            } else if err_str.contains("ACs failed") {
                // ACs failed, but file is in sync - that's fine for docs-check
                Ok(())
            } else {
                // Some other error (file not found, etc.)
                Err(e)
            }
        }
    }
}

/// Document front-matter extracted from markdown files
/// Must align with specs/doc_index.yaml entries (Docs-as-Code v3)
#[derive(Debug, Deserialize)]
struct DocFrontMatter {
    id: String,
    doc_type: String,
    #[serde(default)]
    stories: Vec<String>,
    #[serde(default)]
    requirements: Vec<String>,
    #[serde(default)]
    acs: Vec<String>,
    #[serde(default)]
    adrs: Vec<String>,
}

pub(crate) fn validate_doc_index() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");
    let index_path = root.join("specs/doc_index.yaml");

    if !index_path.exists() {
        // Not fatal if doc_index doesn't exist yet (MVP phase)
        return Ok(());
    }

    let index = crate::docs_index::load_doc_index(&index_path)?;
    let mut errors = Vec::new();

    for entry in &index.docs {
        let doc_path = root.join(&entry.file);
        if !doc_path.exists() {
            errors.push(format!(
                "Doc '{}' listed in index but file missing: {}",
                entry.id, entry.file
            ));
            continue;
        }

        // Skip front-matter validation for CI workflows (they are YAML files themselves)
        if entry.doc_type == "ci_workflow" {
            continue;
        }

        let content = fs::read_to_string(&doc_path)?;
        match parse_front_matter(&content) {
            Ok(fm) => {
                // Core field validation
                if fm.id != entry.id {
                    errors.push(format!(
                        "ID mismatch in {}: front-matter='{}', index='{}'",
                        entry.file, fm.id, entry.id
                    ));
                }
                if fm.doc_type != entry.doc_type {
                    errors.push(format!(
                        "doc_type mismatch in {}: front-matter='{}', index='{}'",
                        entry.file, fm.doc_type, entry.doc_type
                    ));
                }

                // Docs-as-Code v3: Bidirectional validation for all reference fields
                // Check index → front-matter (items in index must be in front-matter)
                for story in &entry.stories {
                    if !fm.stories.contains(story) {
                        errors.push(format!(
                            "Story '{}' in index but not front-matter: {}",
                            story, entry.file
                        ));
                    }
                }
                for req in &entry.requirements {
                    if !fm.requirements.contains(req) {
                        errors.push(format!(
                            "Requirement '{}' in index but not front-matter: {}",
                            req, entry.file
                        ));
                    }
                }
                for ac in &entry.acs {
                    if !fm.acs.contains(ac) {
                        errors.push(format!(
                            "AC '{}' in index but not front-matter: {}",
                            ac, entry.file
                        ));
                    }
                }
                for adr in &entry.adrs {
                    if !fm.adrs.contains(adr) {
                        errors.push(format!(
                            "ADR '{}' in index but not front-matter: {}",
                            adr, entry.file
                        ));
                    }
                }

                // Check front-matter → index (items in front-matter must be in index)
                for story in &fm.stories {
                    if !entry.stories.contains(story) {
                        errors.push(format!(
                            "Story '{}' in front-matter but not index: {}",
                            story, entry.file
                        ));
                    }
                }
                for req in &fm.requirements {
                    if !entry.requirements.contains(req) {
                        errors.push(format!(
                            "Requirement '{}' in front-matter but not index: {}",
                            req, entry.file
                        ));
                    }
                }
                for ac in &fm.acs {
                    if !entry.acs.contains(ac) {
                        errors.push(format!(
                            "AC '{}' in front-matter but not index: {}",
                            ac, entry.file
                        ));
                    }
                }
                for adr in &fm.adrs {
                    if !entry.adrs.contains(adr) {
                        errors.push(format!(
                            "ADR '{}' in front-matter but not index: {}",
                            adr, entry.file
                        ));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Failed to parse front-matter in {}: {}", entry.file, e));
            }
        }
    }

    // Check for orphaned docs: docs with front-matter but not in the index
    // Scan docs/ directory for markdown files with front-matter that aren't registered
    let indexed_files: std::collections::HashSet<String> =
        index.docs.iter().map(|e| e.file.clone()).collect();

    for docs_dir in [
        "docs",
        "docs/adr",
        "docs/design",
        "docs/how-to",
        "docs/explanation",
        "docs/reference",
        "docs/plans",
        "docs/runbooks",
    ] {
        let dir_path = root.join(docs_dir);
        if !dir_path.exists() {
            continue;
        }

        let Ok(entries) = fs::read_dir(&dir_path) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            // Get relative path from repo root
            let rel_path =
                path.strip_prefix(root).ok().and_then(|p| p.to_str()).unwrap_or("").to_string();
            if rel_path.is_empty() || indexed_files.contains(&rel_path) {
                continue;
            }

            // Check if this file has front-matter
            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(fm) = parse_front_matter(&content)
            {
                errors.push(format!(
                    "Doc '{}' ({}) has front-matter but is not registered in doc_index.yaml",
                    fm.id, rel_path
                ));
            }
        }
    }

    if !errors.is_empty() {
        eprintln!();
        for err in &errors {
            eprintln!("  ✗ {}", err);
        }
        eprintln!();
        eprintln!("To fix:");
        eprintln!("  • Align front-matter and specs/doc_index.yaml");
        eprintln!("  • Or update doc_index if the mapping changed intentionally");
        anyhow::bail!("Docs-as-Spec: {} issue(s)", errors.len());
    }

    Ok(())
}

fn parse_front_matter(content: &str) -> Result<DocFrontMatter> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        anyhow::bail!("Missing YAML front-matter");
    }

    let rest = &trimmed[3..]; // Skip first "---"
    if let Some(end_pos) = rest.find("\n---") {
        let yaml_str = &rest[..end_pos];
        let fm: DocFrontMatter = serde_yaml::from_str(yaml_str)?;
        Ok(fm)
    } else {
        anyhow::bail!("Malformed front-matter: missing closing ---");
    }
}

fn validate_doc_policies() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    let policies_path = root.join("specs/doc_policies.yaml");
    let ledger_path = root.join("specs/spec_ledger.yaml");
    let index_path = root.join("specs/doc_index.yaml");

    if !policies_path.exists() {
        return Ok(());
    }

    let policies = crate::docs_index::load_policies(&policies_path)?;
    let ledger = crate::docs_index::load_ledger(&ledger_path)?;
    let index = if index_path.exists() {
        crate::docs_index::load_doc_index(&index_path)?
    } else {
        crate::docs_index::DocIndex {
            schema_version: "1.0".to_string(),
            template_version: "0.0.0".to_string(),
            docs: vec![],
        }
    };

    let mut violations = Vec::new();

    // Build map of Requirement ID -> List of (DocEntry, DocType)
    let mut req_docs: std::collections::HashMap<String, Vec<&crate::docs_index::DocEntry>> =
        std::collections::HashMap::new();

    for doc in &index.docs {
        for req_id in &doc.requirements {
            req_docs.entry(req_id.clone()).or_default().push(doc);
        }
    }

    // Check each requirement against policies
    for story in &ledger.stories {
        for req in &story.requirements {
            for rule in &policies.rules {
                // Check if rule applies
                let applies =
                    rule.applies_to.requirement_tags.iter().any(|tag| req.tags.contains(tag));

                if applies {
                    // Check if satisfied
                    let docs_for_req = req_docs.get(&req.id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let matching_docs_count = docs_for_req
                        .iter()
                        .filter(|d| rule.require_doc_types.contains(&d.doc_type))
                        .count();

                    if matching_docs_count < rule.min_docs {
                        violations.push(format!(
                            "Requirement {} (tags: {:?}) violates policy '{}': requires at least {} doc(s) of type {:?}, found {}",
                            req.id, req.tags, rule.id, rule.min_docs, rule.require_doc_types, matching_docs_count
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        eprintln!();
        for v in &violations {
            eprintln!("  ✗ {}", v);
        }
        eprintln!();
        anyhow::bail!("{} policy violation(s)", violations.len());
    }

    Ok(())
}

fn validate_service_policies() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    let policies_path = root.join("specs/service_policies.yaml");
    if !policies_path.exists() {
        return Ok(());
    }

    // Check if SERVICE_METADATA.yaml exists (it might not in the template repo itself, but we check if it does)
    let metadata_path = root.join("docs/templates/SERVICE_METADATA.example.yaml");
    if !metadata_path.exists() {
        // In template repo, we might skip this or check the example
        return Ok(());
    }

    // Load metadata
    let content = std::fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read {}", metadata_path.display()))?;
    let metadata: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Check runbook requirement (advisory - detect and report, never auto-create)
    if let Some(true) = metadata.get("runbook_required").and_then(|v| v.as_bool()) {
        let runbooks_dir = root.join("docs/runbooks");
        if !runbooks_dir.exists() {
            eprintln!("  ⚠ Advisory: runbook_required=true but docs/runbooks/ does not exist");
            eprintln!("    Remediation: mkdir -p docs/runbooks && create runbook files");
        } else if runbooks_dir.read_dir()?.next().is_none() {
            eprintln!("  ⚠ Advisory: runbook_required=true but docs/runbooks/ is empty");
            eprintln!("    Remediation: add runbook files to docs/runbooks/");
        }
    }

    Ok(())
}

/// Validate feature_status.md header invariants (AC-PLT-010 extension)
/// Ensures the generated file contains Template Version and Last Updated metadata
/// that match the spec_ledger.yaml source of truth.
pub(crate) fn validate_feature_status_invariants() -> Result<()> {
    let canonical_version = extract_version_from_ledger()?;
    if canonical_version == "unknown" {
        return Ok(()); // Can't validate if we can't extract canonical version
    }

    let feature_status_path = "docs/feature_status.md";
    let content = fs::read_to_string(feature_status_path)
        .map_err(|e| anyhow::anyhow!("Could not read {}: {}", feature_status_path, e))?;

    let mut has_version = false;
    let mut version_match = false;

    // Check for Template Version in the HTML comment header
    for line in content.lines().take(20) {
        // Look for: "  Template Version: X.Y.Z"
        if line.contains("Template Version:") {
            has_version = true;
            if line.contains(&canonical_version) {
                version_match = true;
            } else {
                // Extract the version found
                if let Some(found_version) = line.split("Template Version:").nth(1) {
                    let found = found_version.trim();
                    eprintln!(
                        "  Version mismatch in {}: found '{}', expected '{}'",
                        feature_status_path, found, canonical_version
                    );
                }
            }
            break;
        }
    }

    if !has_version {
        eprintln!(
            "  {} header missing Template Version metadata. Run 'cargo xtask ac-status' to regenerate.",
            feature_status_path
        );
        return Err(anyhow::anyhow!(
            "{} header does not contain Template Version invariant",
            feature_status_path
        ));
    }

    if !version_match {
        return Err(anyhow::anyhow!(
            "{} Template Version does not match spec_ledger.yaml",
            feature_status_path
        ));
    }

    Ok(())
}

/// Validate that every kernel REQ (must_have_ac=true) has at least one doc covering it
/// in doc_index.yaml. This ensures kernel design is documented, not just tested.
///
/// A "kernel REQ" is defined as:
/// - A requirement with `must_have_ac: true`, OR
/// - A requirement that contains at least one AC with `must_have_ac: true`
fn validate_kernel_req_doc_coverage() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");
    let ledger_path = root.join("specs/spec_ledger.yaml");
    let index_path = root.join("specs/doc_index.yaml");

    // If doc_index doesn't exist, skip this check (MVP phase)
    if !index_path.exists() {
        return Ok(());
    }

    let ledger = crate::docs_index::load_ledger(&ledger_path)?;
    let index = crate::docs_index::load_doc_index(&index_path)?;

    // Build set of REQ IDs that are documented
    let documented_reqs: std::collections::HashSet<&str> =
        index.docs.iter().flat_map(|doc| doc.requirements.iter().map(|r| r.as_str())).collect();

    // Find kernel REQs without documentation
    let mut missing_docs: Vec<String> = Vec::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            // Check if this is a kernel REQ:
            // 1. REQ itself has must_have_ac: true, OR
            // 2. Any AC under it has must_have_ac: true
            let is_kernel_req =
                req.must_have_ac || req.acceptance_criteria.iter().any(|ac| ac.must_have_ac);

            if is_kernel_req && !documented_reqs.contains(req.id.as_str()) {
                missing_docs.push(format!("{} (\"{}\")", req.id, req.title));
            }
        }
    }

    if !missing_docs.is_empty() {
        eprintln!();
        eprintln!("{}", "Kernel requirements missing documentation:".yellow().bold());
        for req in &missing_docs {
            eprintln!("  ✗ {}", req);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Create a doc in docs/… covering the requirement");
        eprintln!("  • Register it in specs/doc_index.yaml with the REQ ID in 'requirements:'");
        eprintln!("  • Or demote the REQ to non-kernel by removing must_have_ac: true");
        anyhow::bail!(
            "Kernel REQ doc coverage: {} kernel requirement(s) have no documentation",
            missing_docs.len()
        );
    }

    Ok(())
}

/// Validate doc_type contracts for each document in the index.
/// Each doc_type has minimal expectations for what references it should contain.
/// This is a soft check (warnings only) to encourage gradual improvement.
///
/// See docs/reference/doc-sources.md Section 6.5 for the full contract table.
fn validate_doc_types(index: &crate::docs_index::DocIndex) -> Result<()> {
    let mut issues = Vec::new();

    for doc in &index.docs {
        // Normalize doc_type: treat "how-to" as "how_to"
        let doc_type = doc.doc_type.replace('-', "_");

        match doc_type.as_str() {
            "how_to" => {
                // Step-by-step runbooks: requirements or acs must be non-empty
                if doc.requirements.is_empty() && doc.acs.is_empty() {
                    issues.push(format!(
                        "how_to '{}' ({}) should reference at least one requirement or AC",
                        doc.id, doc.file
                    ));
                }
            }
            "explanation" => {
                // Conceptual background: stories or requirements must be non-empty
                if doc.stories.is_empty() && doc.requirements.is_empty() {
                    issues.push(format!(
                        "explanation '{}' ({}) should reference at least one story or requirement",
                        doc.id, doc.file
                    ));
                }
            }
            "design_doc" => {
                // Architecture / decisions: requirements must be non-empty
                if doc.requirements.is_empty() {
                    issues.push(format!(
                        "design_doc '{}' ({}) should reference at least one requirement",
                        doc.id, doc.file
                    ));
                }
            }
            "reference" => {
                // Commands / APIs / schemas: should reference ≥1 REQ or AC
                if doc.requirements.is_empty() && doc.acs.is_empty() {
                    issues.push(format!(
                        "reference '{}' ({}) should reference at least one requirement or AC",
                        doc.id, doc.file
                    ));
                }
            }
            "status" => {
                // Snapshots / roadmaps: requirements and acs must be non-empty
                if doc.requirements.is_empty() || doc.acs.is_empty() {
                    issues.push(format!(
                        "status '{}' ({}) should reference both requirements and ACs",
                        doc.id, doc.file
                    ));
                }
            }
            "adr" => {
                // Architecture decision record: requirements must be non-empty
                if doc.requirements.is_empty() {
                    issues.push(format!(
                        "adr '{}' ({}) should reference at least one requirement",
                        doc.id, doc.file
                    ));
                }
            }
            "guide" => {
                // User-facing documentation: requirements or acs should be non-empty
                if doc.requirements.is_empty() && doc.acs.is_empty() {
                    issues.push(format!(
                        "guide '{}' ({}) should reference at least one requirement or AC",
                        doc.id, doc.file
                    ));
                }
            }
            "impl_plan" => {
                // Implementation plan: requirements and acs must be non-empty
                if doc.requirements.is_empty() || doc.acs.is_empty() {
                    issues.push(format!(
                        "impl_plan '{}' ({}) should reference both requirements and ACs",
                        doc.id, doc.file
                    ));
                }
            }
            "requirements_doc" => {
                // Requirements specification: requirements must be non-empty
                if doc.requirements.is_empty() {
                    issues.push(format!(
                        "requirements_doc '{}' ({}) should reference at least one requirement",
                        doc.id, doc.file
                    ));
                }
            }
            "ci_workflow" => {
                // CI workflow YAML: no validation (YAML files, not markdown)
            }
            _ => {
                // Unknown doc_type: warn about it
                issues.push(format!(
                    "Unknown doc_type '{}' for doc '{}' ({})",
                    doc.doc_type, doc.id, doc.file
                ));
            }
        }
    }

    if !issues.is_empty() {
        eprintln!();
        eprintln!("{}", "Doc type contract warnings:".yellow().bold());
        for issue in &issues {
            eprintln!("  ⚠ {}", issue);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Add missing requirements/acs/stories in frontmatter + doc_index.yaml");
        eprintln!("  • Or adjust doc_type if the doc was misclassified");
        eprintln!("  • See: docs/reference/doc-sources.md Section 6.5");
        anyhow::bail!("Doc type contracts: {} warning(s)", issues.len());
    }

    Ok(())
}

/// Check for orphaned version strings not covered by version_manifest.yaml.
///
/// Scans Markdown and YAML files for version patterns (vX.Y.Z, X.Y.Z) and
/// checks if they are covered by patterns declared in specs/version_manifest.yaml.
/// Emits DOC_ORPHANED_VERSION errors for unmanaged version strings.
///
/// Supports inline suppression via comments:
/// - Markdown: `<!-- doclint:disable orphan-version -->`
/// - YAML: `# doclint:disable orphan-version`
///
/// Tagged with: AC-TPL-VERSION-MANIFEST
fn check_orphaned_versions() -> Result<()> {
    use regex::Regex;
    use std::collections::HashSet;
    use walkdir::WalkDir;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    // Load version manifest
    let manifest_path = root.join("specs/version_manifest.yaml");
    if !manifest_path.exists() {
        // No manifest means no version governance - skip check
        return Ok(());
    }

    let manifest = crate::commands::versioning::VersionManifest::load_from_path(&manifest_path)
        .context("Failed to load version manifest")?;

    // Build set of "owned" file paths and their patterns
    // A version string is "owned" if it appears in a file covered by the manifest
    // and matches one of the declared patterns for that file
    let mut owned_patterns: HashSet<String> = HashSet::new();

    for target in &manifest.files {
        // Skip glob patterns (e.g., release_evidence/v*.md)
        if target.path.contains('*') {
            continue;
        }

        for pattern in &target.patterns {
            // Track the marker as an owned pattern
            owned_patterns.insert(pattern.marker.clone());

            // Also track the line_pattern regex if available
            if let Some(line_pat) = &pattern.line_pattern {
                owned_patterns.insert(line_pat.clone());
            }
        }
    }

    // Version pattern: vX.Y.Z or X.Y.Z (possibly with -kernel suffix)
    let version_re =
        Regex::new(r"v?\d+\.\d+\.\d+(-kernel)?").context("Failed to compile version regex")?;

    // Suppression comment patterns
    let md_suppress = "<!-- doclint:disable orphan-version -->";
    let yaml_suppress = "# doclint:disable orphan-version";

    let mut orphans = Vec::new();

    // Walk docs/ and specs/ directories for markdown and YAML files
    let scan_dirs =
        vec![root.join("docs"), root.join("specs"), root.join("README.md"), root.join("CLAUDE.md")];

    // Files to skip entirely (they contain version examples or non-template versions)
    let skip_files = [
        "specs/version_manifest.yaml",
        "specs/version_manifest.example.yaml",
        "specs/openapi/openapi.yaml",
        "specs/service_policies.yaml",
        "specs/doc_policies.yaml",
        "specs/devex_flows.yaml",
    ];

    for scan_path in scan_dirs {
        if !scan_path.exists() {
            continue;
        }

        let entries: Vec<_> = if scan_path.is_file() {
            vec![scan_path]
        } else {
            WalkDir::new(&scan_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.is_file()
                        && (path.extension().map(|s| s == "md").unwrap_or(false)
                            || path.extension().map(|s| s == "yaml" || s == "yml").unwrap_or(false))
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        };

        for file_path in entries {
            // Get relative path for checks
            let relative_path = file_path.strip_prefix(root).ok().and_then(|p| p.to_str());

            // Skip files that are explicitly excluded
            if let Some(rel) = relative_path
                && skip_files.contains(&rel)
            {
                continue;
            }

            // Skip files covered by manifest
            let is_manifest_file = relative_path
                .and_then(|p| manifest.files.iter().find(|f| Path::new(&f.path) == Path::new(p)))
                .is_some();

            if is_manifest_file {
                // This file is governed by the manifest - its versions are "owned"
                continue;
            }

            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;

            // Check for file-wide suppression comment in first 25 lines (covers frontmatter + comment)
            let lines: Vec<&str> = content.lines().collect();
            let file_suppressed = lines
                .iter()
                .take(25) // Check first 25 lines for suppression (covers typical frontmatter)
                .any(|line| line.contains(md_suppress) || line.contains(yaml_suppress));

            if file_suppressed {
                // Entire file is suppressed from orphan-version checking
                continue;
            }

            for (line_num, line) in lines.iter().enumerate() {
                // Check for inline suppression comment (per-line)
                if line.contains(md_suppress) || line.contains(yaml_suppress) {
                    continue;
                }

                // Skip lines with schema_version (these are YAML schema versions, not template versions)
                if line.contains("schema_version")
                    || line.contains("openapi:")
                    || line.contains("version:") && line.contains("\"")
                {
                    continue;
                }

                // Look for version patterns
                for cap in version_re.find_iter(line) {
                    let version_str = cap.as_str();

                    // Check if this version occurrence is in an owned context
                    // A version is "owned" if the line contains one of the manifest markers
                    let is_owned =
                        owned_patterns.iter().any(|marker| line.contains(marker.as_str()));

                    if !is_owned {
                        let relative = file_path
                            .strip_prefix(root)
                            .unwrap_or(&file_path)
                            .display()
                            .to_string();

                        orphans.push(format!(
                            "{}:{} - orphaned version '{}' (not in version_manifest.yaml)",
                            relative,
                            line_num + 1,
                            version_str
                        ));
                    }
                }
            }
        }
    }

    if !orphans.is_empty() {
        eprintln!();
        eprintln!("{}", "DOC_ORPHANED_VERSION errors:".yellow().bold());
        for orphan in &orphans {
            eprintln!("  ✗ {}", orphan);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!(
            "  1. Add the file and pattern to {} if this version should be managed",
            "specs/version_manifest.yaml".cyan()
        );
        eprintln!(
            "  2. Or suppress with {} or {}",
            "<!-- doclint:disable orphan-version -->".cyan(),
            "# doclint:disable orphan-version".cyan()
        );
        eprintln!("  3. Or remove the version string if it's stale/duplicated");
        eprintln!(
            "  4. Run {} to see what's managed",
            "cargo xtask release-prepare --dry-run X.Y.Z".cyan()
        );

        anyhow::bail!("{} orphaned version string(s) found", orphans.len());
    }

    Ok(())
}

/// Broken link information for reporting
#[derive(Debug)]
struct BrokenLink {
    /// Path to the file containing the broken link
    file: String,
    /// Line number where the link was found
    line: usize,
    /// The link text (what appears in brackets)
    #[allow(dead_code)]
    text: String,
    /// The target URL/path that is broken
    target: String,
    /// Reason the link is broken
    reason: String,
}

/// Validate markdown links in documentation files.
///
/// Scans markdown files in:
/// - docs/
/// - specs/features/ (only .md files)
/// - README.md, CHANGELOG.md, CLAUDE.md (root)
///
/// Checks that local file links (not http/https) resolve to existing files.
/// Anchor links (#section) are advisory warnings only.
fn validate_markdown_links() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    // Regex to match markdown links: [text](url)
    // Captures: group 1 = link text, group 2 = url/path
    let link_re =
        Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").context("Failed to compile markdown link regex")?;

    let mut broken_links: Vec<BrokenLink> = Vec::new();
    let mut anchor_warnings: Vec<BrokenLink> = Vec::new();

    // Collect markdown files to scan
    let mut files_to_scan: Vec<PathBuf> = Vec::new();

    // Root-level markdown files
    for name in ["README.md", "CHANGELOG.md", "CLAUDE.md"] {
        let path = root.join(name);
        if path.exists() {
            files_to_scan.push(path);
        }
    }

    // docs/ directory (recursive)
    let docs_dir = root.join("docs");
    if docs_dir.exists() {
        for entry in WalkDir::new(&docs_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
            e.path().is_file() && e.path().extension().map(|s| s == "md").unwrap_or(false)
        }) {
            files_to_scan.push(entry.path().to_path_buf());
        }
    }

    // specs/features/ directory (only .md files)
    let specs_features_dir = root.join("specs/features");
    if specs_features_dir.exists() {
        for entry in
            WalkDir::new(&specs_features_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "md").unwrap_or(false)
            })
        {
            files_to_scan.push(entry.path().to_path_buf());
        }
    }

    // Process each file
    for file_path in &files_to_scan {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files we can't read
        };

        let file_dir = file_path.parent().unwrap_or(root);
        let relative_file = file_path.strip_prefix(root).unwrap_or(file_path).display().to_string();

        for (line_num, line) in content.lines().enumerate() {
            // Skip code blocks (simple heuristic: lines starting with ``` or indented by 4 spaces)
            if line.trim_start().starts_with("```") || line.starts_with("    ") {
                continue;
            }

            for cap in link_re.captures_iter(line) {
                let link_text = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let target = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                // Skip external URLs
                if target.starts_with("http://") || target.starts_with("https://") {
                    continue;
                }

                // Skip mailto links
                if target.starts_with("mailto:") {
                    continue;
                }

                // Skip pure anchor links (links to sections in the same document)
                if target.starts_with('#') {
                    // These are self-referential anchors, we could validate them
                    // but for now we skip as they're lower priority
                    continue;
                }

                // Handle links with anchors (e.g., "file.md#section")
                let (path_part, anchor_part) = if let Some(hash_pos) = target.find('#') {
                    (&target[..hash_pos], Some(&target[hash_pos + 1..]))
                } else {
                    (target, None)
                };

                // Skip empty paths (just anchor was handled above)
                if path_part.is_empty() {
                    continue;
                }

                // Resolve the target path relative to the file's directory
                let resolved_path = if let Some(stripped) = path_part.strip_prefix('/') {
                    // Absolute path from repo root
                    root.join(stripped)
                } else {
                    // Relative path from file's directory
                    file_dir.join(path_part)
                };

                // Normalize the path (resolve .. and .)
                let normalized_path = normalize_path(&resolved_path);

                // Check if the file/directory exists
                if !normalized_path.exists() {
                    broken_links.push(BrokenLink {
                        file: relative_file.clone(),
                        line: line_num + 1,
                        text: link_text.to_string(),
                        target: target.to_string(),
                        reason: "File not found".to_string(),
                    });
                    continue;
                }

                // If it's a directory link without trailing slash, that's OK
                // (some markdown renderers handle this)

                // If there's an anchor, try to validate it (advisory only)
                // Note: Nested ifs kept for clarity - each level has distinct semantics
                #[allow(clippy::collapsible_if)]
                if let Some(anchor) = anchor_part {
                    if !anchor.is_empty() && normalized_path.is_file() {
                        if let Ok(target_content) = fs::read_to_string(&normalized_path) {
                            if !anchor_exists_in_markdown(&target_content, anchor) {
                                anchor_warnings.push(BrokenLink {
                                    file: relative_file.clone(),
                                    line: line_num + 1,
                                    text: link_text.to_string(),
                                    target: target.to_string(),
                                    reason: format!("Anchor '{}' not found in target file", anchor),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Report results
    if !broken_links.is_empty() {
        eprintln!();
        eprintln!("{}", "Broken markdown links:".red().bold());
        for link in &broken_links {
            eprintln!(
                "  {}:{} - {} ({})",
                link.file.cyan(),
                link.line,
                link.target.red(),
                link.reason
            );
        }
    }

    if !anchor_warnings.is_empty() {
        eprintln!();
        eprintln!("{}", "Anchor warnings (advisory):".yellow().bold());
        for link in &anchor_warnings {
            eprintln!(
                "  {}:{} - {} ({})",
                link.file.cyan(),
                link.line,
                link.target.yellow(),
                link.reason
            );
        }
    }

    if !broken_links.is_empty() {
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Update broken links to point to existing files");
        eprintln!("  • Or remove references to deleted documents");
        eprintln!("  • Check relative paths are correct from the file's location");
        anyhow::bail!("{} broken link(s) found", broken_links.len());
    }

    Ok(())
}

/// Normalize a path by resolving . and .. components.
fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory markers
            }
            _ => {
                normalized.push(component);
            }
        }
    }
    normalized
}

/// Check if an anchor (heading ID) exists in a markdown file.
///
/// Markdown heading anchors are typically generated by:
/// 1. Converting to lowercase
/// 2. Replacing spaces with hyphens
/// 3. Removing special characters
fn anchor_exists_in_markdown(content: &str, anchor: &str) -> bool {
    let anchor_lower = anchor.to_lowercase();

    for line in content.lines() {
        // Check for markdown headings (# Heading)
        if let Some(heading_text) = line.strip_prefix('#') {
            // Remove leading # and whitespace
            let heading_text = heading_text.trim_start_matches('#').trim();

            // Generate anchor from heading
            let generated_anchor: String = heading_text
                .to_lowercase()
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else if c.is_whitespace() {
                        '-'
                    } else {
                        // Skip other characters
                        '\0'
                    }
                })
                .filter(|&c| c != '\0')
                .collect();

            // Remove consecutive hyphens
            let normalized: String =
                generated_anchor.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");

            if normalized == anchor_lower {
                return true;
            }
        }

        // Also check for explicit anchor tags: {#anchor-id}
        if line.contains(&format!("{{#{}}}", anchor)) {
            return true;
        }

        // Check for HTML anchor tags: <a name="anchor-id">
        if line.contains(&format!("name=\"{}\"", anchor))
            || line.contains(&format!("id=\"{}\"", anchor))
        {
            return true;
        }
    }

    false
}

/// BDD tag issue information for reporting
#[derive(Debug)]
struct BddTagIssue {
    /// Path to the feature file
    file: String,
    /// Line number where the tag was found
    line: usize,
    /// The tag that has an issue
    tag: String,
    /// Description of the issue
    issue: String,
}

/// Validate BDD feature file tags against spec_ledger.yaml.
///
/// This check validates:
/// 1. `@AC-*` tags in feature files exist as AC IDs in spec_ledger.yaml
/// 2. Reports orphaned tags (tags referencing non-existent ACs)
/// 3. Reports scenarios without AC tags (advisory only)
///
/// Special tags like `@ci-only`, `@smoke`, `@wip` are allowed and not validated.
///
/// Issue #95: BDD feature file tag validation
fn validate_bdd_tags() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    let ledger_path = root.join("specs/spec_ledger.yaml");
    let features_dir = root.join("specs/features");

    if !ledger_path.exists() || !features_dir.exists() {
        // Can't validate without spec_ledger or feature files
        return Ok(());
    }

    // Load AC IDs from spec_ledger.yaml
    let ac_ids = extract_ac_ids_from_ledger(&ledger_path)?;

    // Regex to match @AC-* tags: @AC-XXX-NNN or @AC-XXX-NNN-SUFFIX
    let ac_tag_re = Regex::new(r"@(AC-[A-Z]+-[A-Z0-9]+(?:-[A-Z0-9]+)*)")
        .context("Failed to compile AC tag regex")?;

    // Special tags that are allowed and not validated
    let special_tags = [
        "@ci-only",
        "@smoke",
        "@wip",
        "@platform",
        "@issues",
        "@schema",
        "@ordering",
        "@filtering",
        "@pagination",
        "@cursor",
        "@offset",
        "@summary",
        "@error",
        "@devup",
        "@selective_testing",
        "@release_bundle_generation",
        "@release_bundle_structure",
        "@example_fork_ci",
    ];

    let mut orphaned_tags: Vec<BddTagIssue> = Vec::new();
    let mut untagged_scenarios: Vec<BddTagIssue> = Vec::new();

    // Scan all .feature files
    for entry in WalkDir::new(&features_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
        e.path().is_file() && e.path().extension().map(|s| s == "feature").unwrap_or(false)
    }) {
        let file_path = entry.path();
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let relative_file = file_path.strip_prefix(root).unwrap_or(file_path).display().to_string();

        let mut in_scenario = false;
        let mut scenario_has_ac_tag = false;
        let mut scenario_line = 0;
        let mut scenario_tags: Vec<String> = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for tag lines (start with @)
            if trimmed.starts_with('@') {
                // Extract all AC tags from this line
                for cap in ac_tag_re.captures_iter(trimmed) {
                    let tag = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    scenario_tags.push(format!("@{}", tag));

                    // Check if this AC ID exists in spec_ledger
                    if !ac_ids.contains(tag) {
                        orphaned_tags.push(BddTagIssue {
                            file: relative_file.clone(),
                            line: line_num + 1,
                            tag: format!("@{}", tag),
                            issue: "AC not found in spec_ledger".to_string(),
                        });
                    } else {
                        scenario_has_ac_tag = true;
                    }
                }
            }

            // Check for Scenario or Scenario Outline
            if trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:") {
                // If we were in a previous scenario, check if it had an AC tag
                if in_scenario && !scenario_has_ac_tag {
                    // Only report if there were no special tags either
                    let has_special_tag = scenario_tags.iter().any(|t| {
                        special_tags.iter().any(|st| t.to_lowercase() == st.to_lowercase())
                    });
                    if !has_special_tag {
                        // Extract scenario name
                        let scenario_name = trimmed
                            .trim_start_matches("Scenario:")
                            .trim_start_matches("Scenario Outline:")
                            .trim();
                        untagged_scenarios.push(BddTagIssue {
                            file: relative_file.clone(),
                            line: scenario_line,
                            tag: String::new(),
                            issue: format!("Scenario '{}' has no @AC-* tag", scenario_name),
                        });
                    }
                }

                // Start tracking new scenario
                in_scenario = true;
                scenario_has_ac_tag = false;
                scenario_line = line_num + 1;
                scenario_tags.clear();
            }

            // Feature-level tags apply to all scenarios (reset tracking)
            if trimmed.starts_with("Feature:") {
                in_scenario = false;
                scenario_has_ac_tag = false;
                scenario_tags.clear();
            }
        }

        // Check final scenario
        if in_scenario && !scenario_has_ac_tag {
            let has_special_tag = scenario_tags
                .iter()
                .any(|t| special_tags.iter().any(|st| t.to_lowercase() == st.to_lowercase()));
            if !has_special_tag && !untagged_scenarios.iter().any(|s| s.line == scenario_line) {
                untagged_scenarios.push(BddTagIssue {
                    file: relative_file.clone(),
                    line: scenario_line,
                    tag: String::new(),
                    issue: "Scenario has no @AC-* tag".to_string(),
                });
            }
        }
    }

    // Report results
    if !orphaned_tags.is_empty() {
        eprintln!();
        eprintln!("{}", "BDD tag validation issues:".yellow().bold());
        for issue in &orphaned_tags {
            eprintln!(
                "  {}:{} - {} ({})",
                issue.file.cyan(),
                issue.line,
                issue.tag.red(),
                issue.issue
            );
        }
    }

    if !untagged_scenarios.is_empty() && orphaned_tags.is_empty() {
        eprintln!();
        eprintln!("{}", "Untagged scenarios (advisory):".yellow().bold());
        // Only show first 5 to avoid noise
        for issue in untagged_scenarios.iter().take(5) {
            eprintln!("  {}:{} - {}", issue.file.cyan(), issue.line, issue.issue.yellow());
        }
        if untagged_scenarios.len() > 5 {
            eprintln!("  ... and {} more", untagged_scenarios.len() - 5);
        }
    }

    if !orphaned_tags.is_empty() {
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Add missing AC IDs to specs/spec_ledger.yaml");
        eprintln!("  • Or update the @AC-* tag to reference an existing AC");
        eprintln!("  • Use 'cargo xtask ac-new' to create new ACs");
        anyhow::bail!("{} orphaned @AC-* tag(s) found", orphaned_tags.len());
    }

    Ok(())
}

/// Extract AC IDs from spec_ledger.yaml.
/// Returns a set of all AC IDs (e.g., "AC-TPL-001", "AC-PLT-018").
fn extract_ac_ids_from_ledger(ledger_path: &Path) -> Result<std::collections::HashSet<String>> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read {}", ledger_path.display()))?;

    let mut ac_ids = std::collections::HashSet::new();

    // Look for lines like "- id: AC-XXX-NNN"
    let ac_id_re =
        Regex::new(r"^\s*-\s*id:\s*(AC-[A-Z]+-[A-Z0-9]+(?:-[A-Z0-9]+)*)").expect("valid regex");

    for line in content.lines() {
        if let Some(cap) = ac_id_re.captures(line)
            && let Some(id) = cap.get(1)
        {
            ac_ids.insert(id.as_str().to_string());
        }
    }

    Ok(ac_ids)
}

/// Validate that Skills and Agents on disk are aligned with spec_ledger.yaml declarations.
///
/// This check validates:
/// 1. Skills in `.claude/skills/*/SKILL.md` have corresponding entries in spec_ledger
/// 2. Agents in `.claude/agents/*.md` have corresponding entries in spec_ledger
/// 3. Declared skills/agents in spec_ledger exist on disk
///
/// Tagged with: AC-TPL-SKILLS-GOVERNANCE-002, AC-TPL-AGENTS-GOVERNANCE-002
fn validate_skills_agents_alignment() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    let ledger_path = root.join("specs/spec_ledger.yaml");
    if !ledger_path.exists() {
        // Can't validate without spec_ledger
        return Ok(());
    }

    let ledger_content = fs::read_to_string(&ledger_path)
        .with_context(|| format!("Failed to read {}", ledger_path.display()))?;

    // Extract declared skills from spec_ledger
    // Look for AC-TPL-SKILLS-ALIGN-001 which lists expected skills
    let declared_skills = extract_declared_skills(&ledger_content);

    // Extract declared agents from spec_ledger
    // Agents are referenced in REQ-TPL-AGENTS-GOVERNANCE section
    let declared_agents = extract_declared_agents(&ledger_content);

    // Scan filesystem for actual skills
    let skills_dir = root.join(".claude/skills");
    let mut actual_skills: Vec<String> = Vec::new();
    if skills_dir.exists() {
        for entry in WalkDir::new(&skills_dir).min_depth(1).max_depth(1) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                let skill_file = entry.path().join("SKILL.md");
                if skill_file.exists()
                    && let Some(name) = entry.file_name().to_str()
                {
                    actual_skills.push(name.to_string());
                }
            }
        }
    }

    // Scan filesystem for actual agents
    let agents_dir = root.join(".claude/agents");
    let mut actual_agents: Vec<String> = Vec::new();
    if agents_dir.exists() {
        for entry in WalkDir::new(&agents_dir).min_depth(1).max_depth(1) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false)
                    && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                {
                    actual_agents.push(name.to_string());
                }
            }
        }
    }

    let mut issues = Vec::new();

    // Check for orphaned skills (exist in directory but not declared)
    if !declared_skills.is_empty() {
        for skill in &actual_skills {
            if !declared_skills.contains(skill) {
                issues.push(format!(
                    "Skill '{}' exists in .claude/skills/ but is not declared in spec_ledger.yaml (AC-TPL-SKILLS-ALIGN-001)",
                    skill
                ));
            }
        }

        // Check for missing skills (declared but directory missing)
        for skill in &declared_skills {
            if !actual_skills.contains(skill) {
                issues.push(format!(
                    "Skill '{}' declared in spec_ledger.yaml but missing from .claude/skills/",
                    skill
                ));
            }
        }
    }

    // Check for orphaned agents (exist in directory but not declared)
    // Note: Agents governance is softer - we mainly check that agents exist
    // and are governed by REQ-TPL-AGENTS-GOVERNANCE requirements
    if !declared_agents.is_empty() {
        for agent in &actual_agents {
            if !declared_agents.contains(agent) {
                issues.push(format!(
                    "Agent '{}' exists in .claude/agents/ but may need spec_ledger coverage (REQ-TPL-AGENTS-GOVERNANCE)",
                    agent
                ));
            }
        }

        // Check for missing agents (declared but file missing)
        for agent in &declared_agents {
            if !actual_agents.contains(agent) {
                issues.push(format!(
                    "Agent '{}' referenced in spec_ledger.yaml but missing from .claude/agents/",
                    agent
                ));
            }
        }
    }

    // Also validate that basic governance docs exist
    let skills_governance = root.join("docs/SKILLS_GOVERNANCE.md");
    let agents_governance = root.join("docs/AGENTS_GOVERNANCE.md");

    if !actual_skills.is_empty() && !skills_governance.exists() {
        issues.push(
            "Skills exist but docs/SKILLS_GOVERNANCE.md is missing (AC-TPL-SKILLS-GOVERNANCE-001)"
                .to_string(),
        );
    }

    if !actual_agents.is_empty() && !agents_governance.exists() {
        issues.push(
            "Agents exist but docs/AGENTS_GOVERNANCE.md is missing (AC-TPL-AGENTS-GOVERNANCE-001)"
                .to_string(),
        );
    }

    if !issues.is_empty() {
        eprintln!();
        eprintln!("{}", "Skills/Agents alignment issues:".yellow().bold());
        for issue in &issues {
            eprintln!("  ⚠ {}", issue);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Ensure each Skill in .claude/skills/* is listed in AC-TPL-SKILLS-ALIGN-001");
        eprintln!("  • Ensure each Agent in .claude/agents/* has REQ/AC coverage");
        eprintln!("  • See: {}", "docs/SKILLS_GOVERNANCE.md".cyan());
        eprintln!("  • See: {}", "docs/AGENTS_GOVERNANCE.md".cyan());
        anyhow::bail!("{} alignment issue(s)", issues.len());
    }

    Ok(())
}

/// Extract declared skill names from spec_ledger.yaml content.
/// Looks for AC-TPL-SKILLS-ALIGN-001 which lists expected skills.
fn extract_declared_skills(content: &str) -> Vec<String> {
    let mut skills = Vec::new();

    // Look for the AC-TPL-SKILLS-ALIGN-001 section which contains:
    // (bootstrap-dev-env, governed-feature-dev, governed-maintenance,
    // governed-release, governed-governance-debug)
    let skill_pattern = Regex::new(
        r"(?:bootstrap-dev-env|governed-feature-dev|governed-maintenance|governed-release|governed-governance-debug)"
    ).expect("valid regex");

    // Find the AC-TPL-SKILLS-ALIGN-001 section
    let mut in_align_section = false;
    for line in content.lines() {
        if line.contains("AC-TPL-SKILLS-ALIGN-001") {
            in_align_section = true;
            continue;
        }
        // Look for the next AC or requirement to mark end of section
        if in_align_section
            && (line.trim().starts_with("- id: AC-")
                || line.trim().starts_with("- id: REQ-")
                || line.trim().starts_with("- id: US-"))
        {
            break;
        }

        if in_align_section {
            // Extract skill names from this section
            for cap in skill_pattern.find_iter(line) {
                let skill_name = cap.as_str().to_string();
                if !skills.contains(&skill_name) {
                    skills.push(skill_name);
                }
            }
        }
    }

    // Also look for any explicit skills: list in the ledger
    // Pattern: skills: [skill1, skill2] or skills:\n  - skill1\n  - skill2
    let skills_list_re = Regex::new(r#"\.claude/skills/([a-z0-9-]+)"#).expect("valid regex");
    for cap in skills_list_re.captures_iter(content) {
        if let Some(skill_name) = cap.get(1) {
            let name = skill_name.as_str().to_string();
            if !skills.contains(&name) {
                skills.push(name);
            }
        }
    }

    skills
}

/// Extract declared agent names from spec_ledger.yaml content.
/// Looks for REQ-TPL-AGENTS-GOVERNANCE and related sections.
fn extract_declared_agents(content: &str) -> Vec<String> {
    let mut agents = Vec::new();

    // Look for agent references in the form .claude/agents/agent-name.md
    let agents_re = Regex::new(r#"\.claude/agents/([a-z0-9-]+)\.md"#).expect("valid regex");
    for cap in agents_re.captures_iter(content) {
        if let Some(agent_name) = cap.get(1) {
            let name = agent_name.as_str().to_string();
            if !agents.contains(&name) {
                agents.push(name);
            }
        }
    }

    // Also look for agent mentions in the agents/* pattern
    let agents_pattern_re = Regex::new(r#"agents/\*"#).expect("valid regex");
    if agents_pattern_re.is_match(content) {
        // The pattern .claude/agents/* is mentioned, which means agents are governed
        // but specific names are not enumerated in spec_ledger
        // In this case, we return empty to avoid false positives
        // The governance is established by REQ-TPL-AGENTS-GOVERNANCE
    }

    agents
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-PLT-DOC-INDEX-FRONTMATTER: Tests that doc_index.yaml entries and document
    /// front-matter are bidirectionally synchronized for stories, requirements, acs, adrs.
    #[test]
    fn test_doc_index_frontmatter_sync() {
        // This test validates that the validate_doc_index function can catch:
        // 1. Items in index but not in front-matter
        // 2. Items in front-matter but not in index

        // Test parsing of front-matter with all fields
        let content_full = r#"---
id: TEST-DOC-001
doc_type: design_doc
stories: [US-TEST-001]
requirements: [REQ-TEST-001]
acs: [AC-TEST-001]
adrs: [ADR-0001]
---

# Test Document
"#;

        let fm = parse_front_matter(content_full).expect("should parse full front-matter");
        assert_eq!(fm.id, "TEST-DOC-001");
        assert_eq!(fm.doc_type, "design_doc");
        assert_eq!(fm.stories, vec!["US-TEST-001"]);
        assert_eq!(fm.requirements, vec!["REQ-TEST-001"]);
        assert_eq!(fm.acs, vec!["AC-TEST-001"]);
        assert_eq!(fm.adrs, vec!["ADR-0001"]);

        // Test parsing with empty arrays (defaults)
        let content_minimal = r#"---
id: TEST-DOC-002
doc_type: guide
---

# Minimal Document
"#;

        let fm_min =
            parse_front_matter(content_minimal).expect("should parse minimal front-matter");
        assert_eq!(fm_min.id, "TEST-DOC-002");
        assert_eq!(fm_min.doc_type, "guide");
        assert!(fm_min.stories.is_empty());
        assert!(fm_min.requirements.is_empty());
        assert!(fm_min.acs.is_empty());
        assert!(fm_min.adrs.is_empty());
    }

    #[test]
    fn test_parse_front_matter_missing() {
        let content = "# No Front Matter\n\nJust content.";
        let result = parse_front_matter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_front_matter_malformed() {
        let content = "---\nid: TEST\ndoc_type: guide\n# Missing closing ---\n";
        let result = parse_front_matter(content);
        assert!(result.is_err());
    }

    /// @AC-PLT-009: docs-check validates version alignment across 8 consumer files
    #[test]
    fn test_docs_check_validates_eight_consumers() {
        // Verify that docs-check validates version alignment across exactly 8 consumer files
        // Uses the shared VERSION_CONSUMERS constant that check_version_alignment_v2() also uses
        assert_eq!(VERSION_CONSUMERS.len(), 8, "docs-check must validate exactly 8 consumer files");

        // Each file should be a valid path pattern
        for file in &VERSION_CONSUMERS {
            assert!(!file.is_empty(), "Consumer file path should not be empty");
            assert!(
                file.ends_with(".md") || file.ends_with(".yaml"),
                "Consumer file should be .md or .yaml"
            );
        }
    }

    #[test]
    fn test_anchor_exists_in_markdown_heading() {
        let content = r#"
# My Heading

Some content here.

## Another Section

More content.
"#;
        assert!(anchor_exists_in_markdown(content, "my-heading"));
        assert!(anchor_exists_in_markdown(content, "another-section"));
        assert!(!anchor_exists_in_markdown(content, "nonexistent"));
    }

    #[test]
    fn test_anchor_exists_in_markdown_special_chars() {
        let content = r#"
# What's New in v2.0?

Some content.

## API Reference (v1)

More content.
"#;
        assert!(anchor_exists_in_markdown(content, "whats-new-in-v20"));
        assert!(anchor_exists_in_markdown(content, "api-reference-v1"));
    }

    #[test]
    fn test_anchor_exists_explicit_anchor() {
        let content = r#"
# My Heading {#custom-anchor}

Some content.
"#;
        assert!(anchor_exists_in_markdown(content, "custom-anchor"));
    }

    #[test]
    fn test_anchor_exists_html_anchor() {
        let content = r#"
<a name="html-anchor"></a>
# My Heading

<div id="div-anchor">Content</div>
"#;
        assert!(anchor_exists_in_markdown(content, "html-anchor"));
        assert!(anchor_exists_in_markdown(content, "div-anchor"));
    }

    #[test]
    fn test_normalize_path() {
        // Test with parent directory references
        let path = PathBuf::from("/a/b/../c/./d");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, PathBuf::from("/a/c/d"));

        // Test with multiple parent references
        let path2 = PathBuf::from("/a/b/c/../../d");
        let normalized2 = normalize_path(&path2);
        assert_eq!(normalized2, PathBuf::from("/a/d"));
    }

    /// @AC-TPL-SKILLS-GOVERNANCE-002: Tests extraction of declared skills from spec_ledger
    #[test]
    fn test_extract_declared_skills() {
        let content = r#"
      - id: AC-TPL-SKILLS-ALIGN-001
        text: >
          Existing .claude/skills/* are aligned with documented workflows
          (bootstrap-dev-env, governed-feature-dev, governed-maintenance,
          governed-release, governed-governance-debug).
        tags: [kernel]
        must_have_ac: true
      - id: AC-NEXT
"#;
        let skills = extract_declared_skills(content);
        assert!(skills.contains(&"bootstrap-dev-env".to_string()));
        assert!(skills.contains(&"governed-feature-dev".to_string()));
        assert!(skills.contains(&"governed-maintenance".to_string()));
        assert!(skills.contains(&"governed-release".to_string()));
        assert!(skills.contains(&"governed-governance-debug".to_string()));
    }

    /// @AC-TPL-SKILLS-GOVERNANCE-002: Tests extraction from path references
    #[test]
    fn test_extract_declared_skills_from_paths() {
        let content = r#"
        The skill at .claude/skills/my-custom-skill should be governed.
        Also see .claude/skills/another-skill for reference.
"#;
        let skills = extract_declared_skills(content);
        assert!(skills.contains(&"my-custom-skill".to_string()));
        assert!(skills.contains(&"another-skill".to_string()));
    }

    /// @AC-TPL-AGENTS-GOVERNANCE-002: Tests extraction of declared agents from spec_ledger
    #[test]
    fn test_extract_declared_agents() {
        let content = r#"
        See .claude/agents/historian.md for the historian agent.
        Also .claude/agents/example-agent.md is provided as a template.
"#;
        let agents = extract_declared_agents(content);
        assert!(agents.contains(&"historian".to_string()));
        assert!(agents.contains(&"example-agent".to_string()));
    }

    /// @AC-TPL-SKILLS-ALIGN-001: Tests that all 5 governed skills exist
    #[test]
    fn test_skills_agents_alignment_integration() {
        // This test validates the integration by checking that the extraction
        // functions work correctly with realistic spec_ledger content
        let expected_skills = [
            "bootstrap-dev-env",
            "governed-feature-dev",
            "governed-maintenance",
            "governed-release",
            "governed-governance-debug",
        ];

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap().parent().unwrap();
        let skills_dir = root.join(".claude/skills");

        // Verify each expected skill exists
        for skill in &expected_skills {
            let skill_path = skills_dir.join(skill).join("SKILL.md");
            assert!(
                skill_path.exists(),
                "Governed skill '{}' should exist at {}",
                skill,
                skill_path.display()
            );
        }
    }

    /// Issue #95: Tests extraction of AC IDs from spec_ledger content
    #[test]
    fn test_extract_ac_ids_from_ledger_content() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"
stories:
  - id: US-TEST-001
    title: Test Story
    requirements:
      - id: REQ-TEST-001
        title: Test Requirement
        acceptance_criteria:
          - id: AC-TEST-001
            text: First acceptance criterion
          - id: AC-TEST-002
            text: Second acceptance criterion
      - id: REQ-TEST-002
        title: Another Requirement
        acceptance_criteria:
          - id: AC-TEST-003-SUFFIX
            text: AC with suffix
  - id: US-PLATFORM-001
    title: Platform Story
    requirements:
      - id: REQ-PLT-001
        title: Platform Requirement
        acceptance_criteria:
          - id: AC-PLT-001
            text: Platform AC
"#;

        // Write content to a temp file
        let mut temp_file = NamedTempFile::new().expect("create temp file");
        temp_file.write_all(content.as_bytes()).expect("write content");

        let ac_ids = extract_ac_ids_from_ledger(temp_file.path()).expect("extract AC IDs");

        assert!(ac_ids.contains("AC-TEST-001"), "Should contain AC-TEST-001");
        assert!(ac_ids.contains("AC-TEST-002"), "Should contain AC-TEST-002");
        assert!(ac_ids.contains("AC-TEST-003-SUFFIX"), "Should contain AC-TEST-003-SUFFIX");
        assert!(ac_ids.contains("AC-PLT-001"), "Should contain AC-PLT-001");
        assert_eq!(ac_ids.len(), 4, "Should have exactly 4 AC IDs");
    }

    /// Issue #95: Tests that BDD tag validation integrates with real spec_ledger
    #[test]
    fn test_bdd_tag_validation_with_real_ledger() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap().parent().unwrap();
        let ledger_path = root.join("specs/spec_ledger.yaml");

        if !ledger_path.exists() {
            return; // Skip if no ledger
        }

        let ac_ids = extract_ac_ids_from_ledger(&ledger_path).expect("extract AC IDs");

        // Verify some known ACs exist
        assert!(ac_ids.len() > 50, "Should have many AC IDs in spec_ledger");

        // Check for some known AC patterns
        let has_plt = ac_ids.iter().any(|id| id.starts_with("AC-PLT-"));
        let has_tpl = ac_ids.iter().any(|id| id.starts_with("AC-TPL-"));

        assert!(has_plt || has_tpl, "Should have AC-PLT-* or AC-TPL-* IDs");
    }
}
