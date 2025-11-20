use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

/// Print a flow-based map of xtask commands loaded from specs/devex_flows.yaml
pub fn run() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let spec_path = root.join("specs/devex_flows.yaml");
    let spec = crate::devex::load_spec(&spec_path)?;

    println!("{}", "FLOWS & COMMAND GROUPS".bold());
    println!();

    // Group commands by category
    let mut by_category: std::collections::HashMap<
        String,
        Vec<(&String, &crate::devex::spec::CommandSpec)>,
    > = std::collections::HashMap::new();

    for (cmd_name, cmd_spec) in &spec.commands {
        by_category
            .entry(cmd_spec.category.clone())
            .or_default()
            .push((cmd_name, cmd_spec));
    }

    // Render each category in canonical order
    for (category_id, heading) in [
        ("onboarding", "🚀 Onboarding (New Developer / Machine)"),
        ("design_ac", "✨ Design & Acceptance Criteria"),
        ("security", "🔒 Security & Dependencies"),
        ("release", "📦 Release Management"),
        ("docs", "📚 Documentation & Consistency"),
        ("infrastructure", "🛠️  Infrastructure & Maintenance"),
        ("meta", "🔍 Meta"),
    ] {
        if let Some(mut cmds) = by_category.get(category_id).cloned() {
            println!("{}", heading.cyan().bold());
            cmds.sort_by_key(|(name, _)| *name);
            for (name, spec) in cmds {
                println!("  {:<15} {}", name.bold(), spec.summary);
            }
            println!();
        }
    }

    println!("{}", "For full details:".dimmed());
    println!("  • {}", "README.md → Developer Workflows".dimmed());
    println!("  • {}", "CLAUDE.md → Golden Path Workflows".dimmed());
    println!("  • {}", "CONTRIBUTING.md → Common Workflows".dimmed());

    Ok(())
}
