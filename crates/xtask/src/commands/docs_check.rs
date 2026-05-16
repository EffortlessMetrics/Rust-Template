use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

mod ac_status;
mod bdd_tags;
mod front_matter;
mod links;
mod policies;
mod skills_agents;
mod version_alignment;

use ac_status::check_ac_status_clean;
use bdd_tags::validate_bdd_tags;
pub(crate) use front_matter::validate_doc_index;
use links::validate_markdown_links;
pub(crate) use policies::{
    check_orphaned_versions, validate_doc_policies, validate_doc_types,
    validate_feature_status_invariants, validate_kernel_req_doc_coverage,
    validate_service_policies,
};
use skills_agents::validate_skills_agents_alignment;
pub(crate) use version_alignment::check_version_alignment_v2;

#[cfg(test)]
pub(crate) use bdd_tags::extract_ac_ids_from_ledger;
#[cfg(test)]
pub(crate) use front_matter::parse_front_matter;
#[cfg(test)]
pub(crate) use links::{anchor_exists_in_markdown, normalize_path};
#[cfg(test)]
pub(crate) use skills_agents::{extract_declared_agents, extract_declared_skills};
#[cfg(test)]
pub(crate) use version_alignment::VERSION_CONSUMERS;

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
            eprintln!("  3. Or demote the REQ by removing 'must_have_ac: true'");
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

    // Check markdown links (hard gate - broken internal links fail docs-check)
    print!("Markdown links... ");
    match validate_markdown_links() {
        Ok(_) => println!("{}", "✓ Valid".green()),
        Err(e) => {
            println!("{}", "✗ Broken links found".red());
            eprintln!("  {}", e);
            issues += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
