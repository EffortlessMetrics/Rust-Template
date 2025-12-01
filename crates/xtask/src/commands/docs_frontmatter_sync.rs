use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Document front-matter extracted from markdown files
/// Must align with specs/doc_index.yaml entries (Docs-as-Code v3)
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct DocFrontMatter {
    id: String,
    doc_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    stories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    requirements: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    acs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    adrs: Vec<String>,
}

/// Result of syncing a single file
#[derive(Debug)]
struct SyncResult {
    file: String,
    status: SyncStatus,
}

#[derive(Debug)]
enum SyncStatus {
    InSync,
    Updated,
    Error(String),
}

pub fn run(fix: bool) -> Result<()> {
    let mode = if fix { "SYNC" } else { "CHECK" };
    println!("{}", format!("📝 Docs Front-matter {} Mode", mode).blue().bold());
    println!();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");
    let index_path = root.join("specs/doc_index.yaml");

    if !index_path.exists() {
        anyhow::bail!("specs/doc_index.yaml not found. Cannot sync front-matter without index.");
    }

    // Load doc index
    let index = load_doc_index(&index_path)?;
    println!("Loaded doc index: {} entries", index.docs.len().to_string().cyan());
    println!();

    let mut results: Vec<SyncResult> = Vec::new();
    let mut updated_count = 0;
    let mut error_count = 0;
    let mut in_sync_count = 0;

    // Sync each design doc entry
    for entry in &index.docs {
        // Skip non-markdown files (CI workflows are YAML)
        if entry.doc_type == "ci_workflow" {
            continue;
        }

        let result = sync_file(root, entry, fix)?;

        match &result.status {
            SyncStatus::InSync => in_sync_count += 1,
            SyncStatus::Updated => updated_count += 1,
            SyncStatus::Error(_) => error_count += 1,
        }

        results.push(result);
    }

    // Print summary
    println!();
    println!("{}", "Summary:".bold());
    println!("  {} in sync", in_sync_count.to_string().green());
    if updated_count > 0 {
        println!("  {} updated", updated_count.to_string().yellow());
    }
    if error_count > 0 {
        println!("  {} errors", error_count.to_string().red());
    }
    println!();

    // Print detailed results
    for result in &results {
        match &result.status {
            SyncStatus::InSync => {
                if !fix {
                    println!("  {} {}", "✓".green(), result.file.dimmed());
                }
            }
            SyncStatus::Updated => {
                println!("  {} {}", "↻".yellow(), result.file);
            }
            SyncStatus::Error(msg) => {
                println!("  {} {} - {}", "✗".red(), result.file, msg.red());
            }
        }
    }

    if error_count > 0 {
        anyhow::bail!("{} file(s) had errors during sync", error_count);
    }

    if !fix && updated_count > 0 {
        println!();
        println!("{}", "Run with --fix to update files:".bold());
        println!("  {}", "cargo xtask docs-frontmatter-sync --fix".cyan());
        anyhow::bail!("{} file(s) need front-matter sync", updated_count);
    }

    if fix && updated_count > 0 {
        println!();
        println!("{}", format!("✓ Updated {} file(s)", updated_count).green().bold());
    } else if !fix && updated_count == 0 && error_count == 0 {
        println!("{}", "✓ All front-matter in sync".green().bold());
    }

    Ok(())
}

/// Load doc_index.yaml using spec_runtime
fn load_doc_index(path: &PathBuf) -> Result<spec_runtime::DocIndex> {
    spec_runtime::load_doc_index(path)
}

/// Sync a single file: check or update front-matter from index
fn sync_file(root: &Path, entry: &spec_runtime::DocEntry, fix: bool) -> Result<SyncResult> {
    let doc_path = root.join(&entry.file);

    if !doc_path.exists() {
        return Ok(SyncResult {
            file: entry.file.clone(),
            status: SyncStatus::Error(format!("File not found: {}", entry.file)),
        });
    }

    let content =
        fs::read_to_string(&doc_path).with_context(|| format!("Failed to read {}", entry.file))?;

    // Parse existing front-matter
    let (existing_fm, body_start) = match parse_frontmatter(&content) {
        Ok((fm, start)) => (Some(fm), start),
        Err(_) => (None, 0),
    };

    // Generate expected front-matter from index
    let expected_fm = DocFrontMatter {
        id: entry.id.clone(),
        doc_type: entry.doc_type.clone(),
        stories: entry.stories.clone(),
        requirements: entry.requirements.clone(),
        acs: entry.acs.clone(),
        adrs: entry.adrs.clone(),
    };

    // Compare
    let needs_update = match &existing_fm {
        Some(fm) => fm != &expected_fm,
        None => true, // No front-matter, needs to be added
    };

    if !needs_update {
        return Ok(SyncResult { file: entry.file.clone(), status: SyncStatus::InSync });
    }

    if !fix {
        // Check mode: just report the difference
        return Ok(SyncResult {
            file: entry.file.clone(),
            status: SyncStatus::Updated, // Would be updated if --fix was used
        });
    }

    // Fix mode: write the corrected front-matter
    let new_frontmatter = generate_frontmatter(&expected_fm);
    let body =
        if body_start > 0 { content[body_start..].trim_start() } else { content.trim_start() };

    let new_content = format!("{}\n{}", new_frontmatter, body);

    fs::write(&doc_path, new_content).with_context(|| format!("Failed to write {}", entry.file))?;

    Ok(SyncResult { file: entry.file.clone(), status: SyncStatus::Updated })
}

/// Parse front-matter from markdown content
/// Returns (DocFrontMatter, body_start_offset)
fn parse_frontmatter(content: &str) -> Result<(DocFrontMatter, usize)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        anyhow::bail!("Missing YAML front-matter");
    }

    let rest = &trimmed[3..]; // Skip first "---"
    if let Some(end_pos) = rest.find("\n---") {
        let yaml_str = &rest[..end_pos];
        let fm: DocFrontMatter =
            serde_yaml::from_str(yaml_str).context("Failed to parse front-matter YAML")?;

        // Calculate body start: 3 (first ---) + end_pos + 5 (\n--- + \n)
        let body_start = content.find("\n---").unwrap() + 5;
        Ok((fm, body_start))
    } else {
        anyhow::bail!("Malformed front-matter: missing closing ---");
    }
}

/// Generate YAML front-matter string from DocFrontMatter
fn generate_frontmatter(fm: &DocFrontMatter) -> String {
    // Manually format to ensure clean YAML with proper array formatting
    let mut lines =
        vec!["---".to_string(), format!("id: {}", fm.id), format!("doc_type: {}", fm.doc_type)];

    // Format arrays in compact YAML syntax
    if !fm.stories.is_empty() {
        lines.push(format!("stories: [{}]", fm.stories.join(", ")));
    }

    if !fm.requirements.is_empty() {
        lines.push(format!("requirements: [{}]", fm.requirements.join(", ")));
    }

    if !fm.acs.is_empty() {
        lines.push(format!("acs: [{}]", fm.acs.join(", ")));
    }

    if !fm.adrs.is_empty() {
        lines.push(format!("adrs: [{}]", fm.adrs.join(", ")));
    }

    lines.push("---".to_string());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_full() {
        let content = r#"---
id: TEST-DOC-001
doc_type: design_doc
stories: [US-TEST-001]
requirements: [REQ-TEST-001]
acs: [AC-TEST-001]
adrs: [ADR-0001]
---

# Test Document
"#;

        let (fm, body_start) = parse_frontmatter(content).expect("should parse");
        assert_eq!(fm.id, "TEST-DOC-001");
        assert_eq!(fm.doc_type, "design_doc");
        assert_eq!(fm.stories, vec!["US-TEST-001"]);
        assert_eq!(fm.requirements, vec!["REQ-TEST-001"]);
        assert_eq!(fm.acs, vec!["AC-TEST-001"]);
        assert_eq!(fm.adrs, vec!["ADR-0001"]);
        assert!(body_start > 0);
    }

    #[test]
    fn test_parse_frontmatter_minimal() {
        let content = r#"---
id: TEST-DOC-002
doc_type: guide
---

# Minimal Document
"#;

        let (fm, _) = parse_frontmatter(content).expect("should parse");
        assert_eq!(fm.id, "TEST-DOC-002");
        assert_eq!(fm.doc_type, "guide");
        assert!(fm.stories.is_empty());
        assert!(fm.requirements.is_empty());
        assert!(fm.acs.is_empty());
        assert!(fm.adrs.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_missing() {
        let content = "# No Front Matter\n\nJust content.";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_malformed() {
        let content = "---\nid: TEST\ndoc_type: guide\n# Missing closing ---\n";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_frontmatter_full() {
        let fm = DocFrontMatter {
            id: "TEST-001".to_string(),
            doc_type: "design_doc".to_string(),
            stories: vec!["US-001".to_string()],
            requirements: vec!["REQ-001".to_string()],
            acs: vec!["AC-001".to_string()],
            adrs: vec!["ADR-0001".to_string()],
        };

        let yaml = generate_frontmatter(&fm);
        assert!(yaml.contains("id: TEST-001"));
        assert!(yaml.contains("doc_type: design_doc"));
        assert!(yaml.contains("stories: [US-001]"));
        assert!(yaml.contains("requirements: [REQ-001]"));
        assert!(yaml.contains("acs: [AC-001]"));
        assert!(yaml.contains("adrs: [ADR-0001]"));
        assert!(yaml.starts_with("---"));
        assert!(yaml.ends_with("---"));
    }

    #[test]
    fn test_generate_frontmatter_minimal() {
        let fm = DocFrontMatter {
            id: "TEST-002".to_string(),
            doc_type: "guide".to_string(),
            stories: vec![],
            requirements: vec![],
            acs: vec![],
            adrs: vec![],
        };

        let yaml = generate_frontmatter(&fm);
        assert!(yaml.contains("id: TEST-002"));
        assert!(yaml.contains("doc_type: guide"));
        // Empty arrays should not appear
        assert!(!yaml.contains("stories:"));
        assert!(!yaml.contains("requirements:"));
        assert!(!yaml.contains("acs:"));
        assert!(!yaml.contains("adrs:"));
    }

    #[test]
    fn test_frontmatter_roundtrip() {
        let original = DocFrontMatter {
            id: "ROUNDTRIP-001".to_string(),
            doc_type: "design_doc".to_string(),
            stories: vec!["US-001".to_string(), "US-002".to_string()],
            requirements: vec!["REQ-A".to_string()],
            acs: vec!["AC-X".to_string(), "AC-Y".to_string()],
            adrs: vec!["ADR-0001".to_string()],
        };

        let yaml = generate_frontmatter(&original);
        let content = format!("{}\n\n# Test Doc", yaml);

        let (parsed, _) = parse_frontmatter(&content).expect("should parse generated YAML");
        assert_eq!(parsed, original);
    }
}
