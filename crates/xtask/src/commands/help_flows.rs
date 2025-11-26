use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

/// Print a flow-based map of xtask commands loaded from specs/devex_flows.yaml
pub fn run() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let spec_path = root.join("specs/devex_flows.yaml");
    let spec = crate::devex::load_spec(&spec_path)?;

    println!("{}", "FLOWS & COMMAND GROUPS".bold());
    println!();
    println!("{}", format!("Source: {}", "specs/devex_flows.yaml").dimmed());
    println!();
    println!("{}", "Workflow Categories:".cyan().bold());
    println!();

    // Group commands by category
    let mut by_category: std::collections::HashMap<
        String,
        Vec<(&String, &crate::devex::spec::CommandSpec)>,
    > = std::collections::HashMap::new();

    for (cmd_name, cmd_spec) in &spec.commands {
        by_category.entry(cmd_spec.category.clone()).or_default().push((cmd_name, cmd_spec));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_help_flows_command_exists() {
        // Verify that the run function is accessible and has the correct signature
        let _: fn() -> Result<()> = run;
    }

    #[test]
    fn test_devex_flows_spec_file_exists() {
        // Verify that the devex_flows.yaml file exists at the expected location
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap().parent().unwrap();
        let spec_path = root.join("specs/devex_flows.yaml");

        assert!(
            spec_path.exists(),
            "specs/devex_flows.yaml should exist at {}",
            spec_path.display()
        );
    }

    #[test]
    fn test_devex_spec_contains_required_categories() {
        // Verify that the devex spec contains the expected categories
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap().parent().unwrap();
        let spec_path = root.join("specs/devex_flows.yaml");

        if spec_path.exists() {
            let spec = crate::devex::load_spec(&spec_path).expect("load devex spec");

            // Check that we have commands in expected categories
            let categories: std::collections::HashSet<_> =
                spec.commands.values().map(|cmd| cmd.category.as_str()).collect();

            assert!(
                categories.contains("onboarding"),
                "devex_flows.yaml should define onboarding category"
            );
            assert!(
                categories.contains("release"),
                "devex_flows.yaml should define release category"
            );
        }
    }
}
