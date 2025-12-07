use anyhow::{Context, Result};
use colored::Colorize;
use spec_runtime::{SpecLedger, load_spec_ledger};

use crate::kernel::layout_for_repo;

pub fn run(ac_id: &str) -> Result<()> {
    let layout = layout_for_repo();
    let ledger: SpecLedger =
        load_spec_ledger(&layout.ledger).context("Failed to load spec_ledger.yaml")?;

    // Find the AC in the ledger
    let mut found_ac = None;
    let mut req_id = String::new();
    let mut story_id = String::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                if ac.id == ac_id {
                    found_ac = Some(ac.clone());
                    req_id = req.id.clone();
                    story_id = story.id.clone();
                    break;
                }
            }
            if found_ac.is_some() {
                break;
            }
        }
        if found_ac.is_some() {
            break;
        }
    }

    let ac = found_ac.context(format!("AC {} not found in spec_ledger.yaml", ac_id))?;

    // Display AC information
    println!("{}", "=".repeat(80).bright_blue());
    println!("{} {}", "Acceptance Criterion:".bright_blue().bold(), ac_id.bright_white().bold());
    println!("{}", "=".repeat(80).bright_blue());
    println!();
    println!("{} {}", "Story:".bright_cyan(), story_id);
    println!("{} {}", "Requirement:".bright_cyan(), req_id);
    println!("{} {}", "Text:".bright_cyan(), ac.text);
    println!();

    if ac.tests.is_empty() {
        println!("{}", "⚠ No tests mapped to this AC".yellow().bold());
        return Ok(());
    }

    println!("{}", "Mapped Tests:".bright_green().bold());
    println!("{}", "-".repeat(80).bright_blue());
    println!();

    for (idx, test_mapping) in ac.tests.iter().enumerate() {
        let test_num = format!("[{}]", idx + 1);
        println!(
            "{} {} {}",
            test_num.bright_yellow().bold(),
            "Type:".bright_cyan(),
            test_mapping.test_type
        );
        println!("    {} {}", "Tag:".bright_cyan(), test_mapping.tag);

        if let Some(file) = &test_mapping.file {
            println!("    {} {}", "File:".bright_cyan(), file.bright_white());
        }

        if let Some(module) = &test_mapping.module {
            println!("    {} {}", "Module:".bright_cyan(), module.bright_white());
        }

        println!();
    }

    // Provide actionable commands
    println!("{}", "Run Tests:".bright_green().bold());
    println!("{}", "-".repeat(80).bright_blue());
    println!();

    for test_mapping in &ac.tests {
        match test_mapping.test_type.as_str() {
            "bdd" | "integration" => {
                let tag = &test_mapping.tag;
                let clean_tag = tag.trim_start_matches('@');
                println!(
                    "  {} {}",
                    "BDD/Integration:".bright_cyan(),
                    format!("cargo xtask test-ac {}", ac_id).bright_white()
                );
                println!(
                    "  {} {}",
                    "Direct:".bright_cyan(),
                    format!("CUCUMBER_TAG_EXPRESSION='@{}' cargo test -p acceptance", clean_tag)
                        .bright_white()
                );
            }
            "unit" => {
                if let Some(module) = &test_mapping.module {
                    println!(
                        "  {} {}",
                        "Unit test:".bright_cyan(),
                        format!("cargo test -p spec-runtime {}", module).bright_white()
                    );
                }
            }
            _ => {
                println!("  {} {}", "Unknown type:".yellow(), test_mapping.test_type);
            }
        }
        println!();
    }

    Ok(())
}
