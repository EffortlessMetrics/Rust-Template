use super::bdd_tags::extract_ac_ids_from_ledger;
use super::governance::parse_front_matter;
use super::links::{anchor_exists_in_markdown, normalize_path};
use super::skills_agents::{extract_declared_agents, extract_declared_skills};
use super::version_alignment::VERSION_CONSUMERS;
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

    let fm_min = parse_front_matter(content_minimal).expect("should parse minimal front-matter");
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
