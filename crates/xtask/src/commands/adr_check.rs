use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Configuration for ADR check
#[derive(Debug, Clone)]
pub struct AdrCheckArgs {
    pub ledger: PathBuf,
    pub adr_dir: PathBuf,
    pub verbosity: crate::Verbosity,
}

impl Default for AdrCheckArgs {
    fn default() -> Self {
        Self {
            ledger: PathBuf::from("specs/spec_ledger.yaml"),
            adr_dir: PathBuf::from("docs/adr"),
            verbosity: crate::Verbosity::Normal,
        }
    }
}

/// Represents a reference to an ADR in the spec ledger
#[derive(Debug)]
struct AdrReference {
    adr_id: String,
    context: String, // e.g., "Story US-TPL-001", "AC AC-TPL-001"
}

/// Spec ledger structures for parsing ADR references
#[derive(Debug, Deserialize)]
struct SpecLedger {
    #[serde(default)]
    metadata: Option<Metadata>,
    stories: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(default)]
    adrs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Story {
    id: String,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    adr: Option<Vec<String>>,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
struct Requirement {
    id: String,
    /// Tags for categorizing requirements (e.g., @tier1, @security).
    /// Future: Used for filtering ADR checks by tag.
    /// See TASK-DX-ADR-FILTERING for planned tag-based ADR validation.
    #[serde(default)]
    #[allow(dead_code)]
    tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    adr: Option<Vec<String>>,
    acceptance_criteria: Vec<AcceptanceCriterion>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriterion {
    id: String,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    adr: Option<Vec<String>>,
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    Option::<StringOrVec>::deserialize(deserializer).map(|opt| match opt {
        Some(StringOrVec::String(s)) => Some(vec![s]),
        Some(StringOrVec::Vec(v)) => Some(v),
        None => None,
    })
}

/// Main ADR check entry point
pub fn run(args: AdrCheckArgs) -> Result<()> {
    let verbose = args.verbosity.is_verbose();
    let quiet = args.verbosity.is_quiet();

    if !quiet {
        println!("{}", "Checking ADR references...".cyan().bold());
    }

    // 1. Collect all ADR references from the ledger
    let references = collect_adr_references(&args.ledger, verbose)?;

    if verbose {
        println!("\nFound {} ADR references in ledger:", references.len());
        for ref_item in &references {
            println!("  {} → {}", ref_item.adr_id.yellow(), ref_item.context.dimmed());
        }
    }

    // 2. Find all ADR files in docs/adr/
    let adr_files = find_adr_files(&args.adr_dir, verbose)?;

    if verbose {
        println!("\nFound {} ADR files:", adr_files.len());
        for adr in &adr_files {
            println!("  {}", adr.green());
        }
    }

    // 3. Validate references
    let mut missing_adrs = Vec::new();
    let referenced_adrs: HashSet<String> = references.iter().map(|r| r.adr_id.clone()).collect();

    for ref_item in &references {
        if !adr_files.contains(&ref_item.adr_id) {
            missing_adrs.push(ref_item);
        }
    }

    // 4. Find unreferenced ADRs
    let unreferenced_adrs: Vec<&String> =
        adr_files.iter().filter(|adr| !referenced_adrs.contains(*adr)).collect();

    // 5. Report results
    let mut has_errors = false;

    if !missing_adrs.is_empty() {
        has_errors = true;
        println!("\n{}", "❌ Missing ADR files:".red().bold());
        for ref_item in &missing_adrs {
            println!("  {} (referenced by {})", ref_item.adr_id.yellow(), ref_item.context);
        }
    }

    if !unreferenced_adrs.is_empty() && !quiet {
        println!("\n{}", "⚠  Unreferenced ADRs (may be orphaned):".yellow().bold());
        for adr in &unreferenced_adrs {
            println!("  {}", adr.dimmed());
        }
        println!(
            "\n  {}",
            "Note: These ADRs exist but aren't referenced in spec_ledger.yaml".dimmed()
        );
        println!("  {}", "They may be deprecated, superseded, or not yet wired in.".dimmed());
    }

    if !has_errors {
        if !quiet {
            println!("\n{}", "✓ All ADR references are valid".green().bold());
            if !unreferenced_adrs.is_empty() {
                println!("  {} unreferenced ADRs (warnings only)", unreferenced_adrs.len());
            }
        }
        Ok(())
    } else {
        anyhow::bail!("ADR validation failed: {} missing ADR files", missing_adrs.len());
    }
}

/// Collect all ADR references from the spec ledger
fn collect_adr_references(ledger_path: &Path, _verbose: bool) -> Result<Vec<AdrReference>> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    let ledger: SpecLedger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger: {}", ledger_path.display()))?;

    let mut references = Vec::new();

    // Collect ADRs from metadata
    if let Some(metadata) = &ledger.metadata
        && let Some(adrs) = &metadata.adrs
    {
        for adr in adrs {
            references.push(AdrReference {
                adr_id: adr.clone(),
                context: "Metadata (template-wide)".to_string(),
            });
        }
    }

    for story in &ledger.stories {
        if let Some(adrs) = &story.adr {
            for adr in adrs {
                references.push(AdrReference {
                    adr_id: adr.clone(),
                    context: format!("Story {}", story.id),
                });
            }
        }

        for req in &story.requirements {
            if let Some(adrs) = &req.adr {
                for adr in adrs {
                    references.push(AdrReference {
                        adr_id: adr.clone(),
                        context: format!("Requirement {} (Story {})", req.id, story.id),
                    });
                }
            }

            for ac in &req.acceptance_criteria {
                if let Some(adrs) = &ac.adr {
                    for adr in adrs {
                        references.push(AdrReference {
                            adr_id: adr.clone(),
                            context: format!("AC {} (Requirement {})", ac.id, req.id),
                        });
                    }
                }
            }
        }
    }

    Ok(references)
}

/// Find all ADR files in docs/adr/ and extract their IDs
fn find_adr_files(adr_dir: &Path, verbose: bool) -> Result<HashSet<String>> {
    if !adr_dir.exists() {
        if verbose {
            println!("  {} ADR directory not found: {}", "⚠".yellow(), adr_dir.display());
        }
        return Ok(HashSet::new());
    }

    let mut adr_ids = HashSet::new();

    for entry in WalkDir::new(adr_dir).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md")
            && let Some(adr_id) = extract_adr_id(path)
        {
            adr_ids.insert(adr_id);
        }
    }

    Ok(adr_ids)
}

/// Extract ADR ID from filename (e.g., "0001-hexagonal-architecture.md" → "ADR-0001")
fn extract_adr_id(path: &Path) -> Option<String> {
    path.file_stem().and_then(|s| s.to_str()).and_then(|name| {
        // Match pattern like "0001-title" or "ADR-0001-title"
        if name.starts_with("ADR-") {
            // Already has ADR- prefix, take "ADR-NNNN" part
            name.splitn(3, '-').take(2).collect::<Vec<_>>().join("-").into()
        } else {
            // Assumes format "NNNN-title"
            name.split('-').next().map(|num| format!("ADR-{}", num))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_adr_id() {
        assert_eq!(extract_adr_id(Path::new("0001-hexagonal.md")), Some("ADR-0001".to_string()));
        assert_eq!(
            extract_adr_id(Path::new("ADR-0002-nix-first.md")),
            Some("ADR-0002".to_string())
        );
        assert_eq!(
            extract_adr_id(Path::new("0042-meaning-of-life.md")),
            Some("ADR-0042".to_string())
        );
    }
}
