use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

mod bdd_tags;
mod governance;
mod links;
mod skills_agents;
#[cfg(test)]
mod tests;
mod version_alignment;

use bdd_tags::validate_bdd_tags;
use governance::{
    check_ac_status_clean, validate_doc_policies, validate_doc_types,
    validate_kernel_req_doc_coverage, validate_service_policies,
};
pub(crate) use governance::{validate_doc_index, validate_feature_status_invariants};
use links::{check_orphaned_versions, validate_markdown_links};
use skills_agents::validate_skills_agents_alignment;
pub(crate) use version_alignment::check_version_alignment_v2;

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
