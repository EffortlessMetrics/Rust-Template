use anyhow::Result;
use colored::Colorize;

use crate::cli::{Commands, get_command_name};
use crate::devex;

pub(crate) fn show_command_docs(command: &Commands) -> Result<()> {
    let name = get_command_name(command);
    println!();
    println!("{} {}", "Documentation for:".bold(), name.cyan());
    println!("{}", "=".repeat(20 + name.len()).blue());
    println!();

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();
    let spec_path = root.join("specs/devex_flows.yaml");

    if let Ok(spec) = devex::load_spec(&spec_path) {
        if let Some(cmd_spec) = spec.commands.get(name) {
            println!("{}", cmd_spec.summary);
            println!();
            println!("{}: {}", "Category".bold(), cmd_spec.category);

            // Find flows containing this command
            let mut flows: Vec<_> =
                spec.flows.iter().filter(|(_, f)| f.steps.iter().any(|s| s == name)).collect();
            flows.sort_by_key(|(id, _)| *id);

            if !flows.is_empty() {
                println!();
                println!("{}", "Part of flows:".bold());
                for (flow_id, flow) in flows {
                    println!("  • {} ({})", flow.name.cyan(), flow_id);
                    for doc in &flow.documented_in {
                        println!("    → {}", doc.dimmed());
                    }
                }
            }
        } else {
            println!("No specific documentation found in devex_flows.yaml for '{}'", name);
        }
    } else {
        println!("Warning: Failed to load specs/devex_flows.yaml");
    }

    // Always link to glossary
    println!();
    println!("{}", "See also:".bold());
    println!("  • {}", "docs/GLOSSARY.md".blue());
    println!("  • {}", "docs/AGENT_GUIDE.md".dimmed());

    Ok(())
}
