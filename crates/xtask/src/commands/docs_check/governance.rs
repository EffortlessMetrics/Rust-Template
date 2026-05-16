use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use super::version_alignment::extract_version_from_ledger;

pub(super) fn check_ac_status_clean() -> Result<()> {
    // Use check mode to verify AC status file matches computed state without writing.
    // This is cleaner than regenerating + checking git status, and doesn't modify the repo.
    //
    // Note: We only enforce strict check mode in CI because:
    // - BDD runs with tag filtering locally (excludes @ci-only scenarios)
    // - Integration tests (AC-MYSERV-*, AC-GOV-025) need app-http running
    // - This produces different coverage than what was used to generate the committed file
    // - The committed file reflects full BDD coverage (CI mode with app-http)
    // - Comparing against partial coverage would always fail locally
    //
    // In CI: check mode verifies file is in sync
    // Locally: skip sync check (selftest step 6 will regenerate anyway)
    let in_ci = crate::env::is_ci();
    if !in_ci {
        // Locally, just verify ac-status runs without errors (don't check file sync)
        return Ok(());
    }

    // Note: We ignore AC failures (test failures) here - we only care if the file is in sync.
    // The ac-status command in check mode will fail if the file content differs, which is
    // what we want to catch. If it fails due to AC test failures, that's a separate concern.
    match crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity: crate::Verbosity::Quiet,
        check: true, // Check mode: verify without writing (CI only)
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
pub(super) struct DocFrontMatter {
    pub(super) id: String,
    pub(super) doc_type: String,
    #[serde(default)]
    pub(super) stories: Vec<String>,
    #[serde(default)]
    pub(super) requirements: Vec<String>,
    #[serde(default)]
    pub(super) acs: Vec<String>,
    #[serde(default)]
    pub(super) adrs: Vec<String>,
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

pub(super) fn parse_front_matter(content: &str) -> Result<DocFrontMatter> {
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

pub(super) fn validate_doc_policies() -> Result<()> {
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

pub(super) fn validate_service_policies() -> Result<()> {
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
pub(super) fn validate_kernel_req_doc_coverage() -> Result<()> {
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
pub(super) fn validate_doc_types(index: &crate::docs_index::DocIndex) -> Result<()> {
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
