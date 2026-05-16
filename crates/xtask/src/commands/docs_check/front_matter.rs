use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub(crate) struct DocFrontMatter {
    pub(crate) id: String,
    pub(crate) doc_type: String,
    #[serde(default)]
    pub(crate) stories: Vec<String>,
    #[serde(default)]
    pub(crate) requirements: Vec<String>,
    #[serde(default)]
    pub(crate) acs: Vec<String>,
    #[serde(default)]
    pub(crate) adrs: Vec<String>,
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

pub(crate) fn parse_front_matter(content: &str) -> Result<DocFrontMatter> {
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
