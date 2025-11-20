use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use spec_runtime::tasks::Action;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct SuggestNextArgs {
    /// Task ID to suggest next steps for
    #[arg(long)]
    pub task: String,

    /// Output format (table or json)
    #[arg(long, default_value = "table")]
    pub format: String,
}

pub fn run(args: SuggestNextArgs) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))?;
    let devex_spec = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))?;
    let ledger = spec_runtime::load_spec_ledger(&root.join("specs/spec_ledger.yaml"))?;

    let suggestion =
        spec_runtime::tasks::suggest_next(root, &args.task, &tasks_spec, &devex_spec, &ledger)?;

    if args.format == "json" {
        println!("{}", serde_json::to_string_pretty(&suggestion)?);
    } else {
        println!("🧭 Suggested next steps for task \"{}\"", suggestion.task.bold());
        println!();
        println!("{}:", "Goal".bold());
        println!("  {}", suggestion.goal);
        println!();
        println!("{}:", "Recommended flow".bold());
        for flow in &suggestion.recommended_flows {
            println!("  {}", flow.cyan());
        }
        println!();
        println!("{}:", "Sequence".bold());
        for (i, action) in suggestion.recommended_sequence.iter().enumerate() {
            match action {
                Action::Command { cmd, description, status } => {
                    let mark = if *status == spec_runtime::tasks::StepStatus::Satisfied {
                        "✓".green()
                    } else {
                        " ".normal()
                    };
                    let style = if *status == spec_runtime::tasks::StepStatus::Satisfied {
                        colored::Colorize::dimmed
                    } else {
                        colored::Colorize::normal
                    };

                    println!(
                        "  {}. {} {}  {}",
                        i + 1,
                        mark,
                        style(cmd.as_str()),
                        format!("({})", description).dimmed()
                    );
                }
                Action::Edit { file, hint, status } => {
                    let mark = if *status == spec_runtime::tasks::StepStatus::Satisfied {
                        "✓".green()
                    } else {
                        " ".normal()
                    };
                    let style = if *status == spec_runtime::tasks::StepStatus::Satisfied {
                        colored::Colorize::dimmed
                    } else {
                        colored::Colorize::normal
                    };

                    println!(
                        "  {}. {} Edit {}  {}",
                        i + 1,
                        mark,
                        style(file.as_str()),
                        format!("({})", hint).dimmed()
                    );
                }
            }
        }
    }

    Ok(())
}
