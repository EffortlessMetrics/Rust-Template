use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub struct DesignNewArgs {
    pub id: String,
    pub title: String,
    pub requirements: Vec<String>,
    pub adrs: Vec<String>,
    pub owner: Option<String>,
}

pub fn run(args: DesignNewArgs) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    let design_dir = root.join("docs/design");
    fs::create_dir_all(&design_dir)
        .with_context(|| format!("Failed to create design dir: {}", design_dir.display()))?;

    let slug = slugify(&args.title);
    let file_name = format!("{}-{}.md", args.id.to_lowercase(), slug);
    let path = design_dir.join(&file_name);

    if path.exists() {
        anyhow::bail!("Design doc already exists at {}", path.display());
    }

    let today = Utc::now().date_naive().to_string();
    let requirements_str = if args.requirements.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            args.requirements.iter().map(|r| format!("\"{}\"", r)).collect::<Vec<_>>().join(", ")
        )
    };
    let adrs_str = if args.adrs.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            args.adrs.iter().map(|a| format!("\"{}\"", a)).collect::<Vec<_>>().join(", ")
        )
    };
    let owner = args.owner.as_deref().unwrap_or("platform");

    let content = format!(
        r#"---
doc_type: design_doc
id: {id}
title: "{title}"
stories: []
requirements: {requirements}
acs: []
adrs: {adrs}
status: draft
last_reviewed: {date}
owner: "{owner}"
---

# {title}

## 1. Context

- Brief description of the problem being solved
- Link to user stories and requirements
- Why this design is needed

## 2. High-Level Design

- Key components and their responsibilities
- Interactions and data flows
- Architecture diagrams (optional)

## 3. Edge Cases & Failure Modes

- Known tricky scenarios
- How errors are surfaced and handled
- Degradation and fallback strategies

## 4. Tests & Invariants

- How BDD/ACs validate this design
- Critical invariants that must hold
- Testing strategy

## 5. Open Questions / Future Work

- Items to be determined
- Explicit non-goals
- Follow-up work or extensions
"#,
        id = args.id,
        title = args.title,
        requirements = requirements_str,
        adrs = adrs_str,
        date = today,
        owner = owner
    );

    fs::write(&path, content).with_context(|| format!("Writing {}", path.display()))?;

    let rel_path = path.strip_prefix(root).unwrap_or(&path);
    println!("{} Created design doc at {}", "✓".green(), rel_path.display());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  • Add this entry to specs/doc_index.yaml:");
    println!("    - id: {}", args.id);
    println!("      file: {}", rel_path.display());
    println!("      doc_type: design_doc");
    println!("      stories: []");
    println!("      requirements: {}", requirements_str);
    println!("      acs: []");
    println!("      adrs: {}", adrs_str);
    println!("  • Fill in the design sections in the new file.");
    println!("  • Run: {}", "cargo xtask docs-check".cyan());

    Ok(())
}

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
