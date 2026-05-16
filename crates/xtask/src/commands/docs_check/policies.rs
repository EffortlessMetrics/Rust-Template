use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::version_alignment::extract_version_from_ledger;

pub(crate) fn validate_doc_policies() -> Result<()> {
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

pub(crate) fn validate_service_policies() -> Result<()> {
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
pub(crate) fn validate_kernel_req_doc_coverage() -> Result<()> {
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
pub(crate) fn validate_doc_types(index: &crate::docs_index::DocIndex) -> Result<()> {
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
pub(crate) fn check_orphaned_versions() -> Result<()> {
    use std::collections::HashSet;

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
