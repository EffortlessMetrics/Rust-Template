use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use serde::Serialize;
use spec_runtime::hints::{self, HintEngine};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct SuggestNextArgs {
    /// Filter by owner name
    #[arg(long)]
    pub owner: Option<String>,

    /// Filter by label/tag
    #[arg(long)]
    pub label: Option<String>,

    /// Filter by requirement ID
    #[arg(long)]
    pub requirement: Option<String>,

    /// Maximum number of hints to return
    #[arg(long)]
    pub limit: Option<usize>,

    /// Output format (table or json)
    #[arg(long, default_value = "table")]
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct CliHint {
    pub task_id: String,
    pub title: String,
    pub status: String,
    pub owner: String,
    pub labels: Vec<String>,
    pub requirement_ids: Vec<String>,
    pub ac_ids: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct CliHintsResponse {
    pub hints: Vec<CliHint>,
}

pub fn run(args: SuggestNextArgs) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    // Load tasks and AC coverage
    let tasks_path = root.join("specs/tasks.yaml");
    let task_specs = spec_runtime::load_tasks(&tasks_path)?;

    let feature_status_path = root.join("docs/feature_status.md");
    let ac_index = hints::parse_feature_status(&feature_status_path)?;

    // Convert task specs to runtime tasks
    let runtime_tasks: Vec<spec_runtime::Task> = task_specs
        .tasks
        .iter()
        .map(|t| spec_runtime::Task {
            id: t.id.clone(),
            title: t.title.clone(),
            status: t.status.clone(),
            requirement: t.requirement.clone(),
            acs: t.acs.clone(),
            labels: t.labels.clone(),
            owner: t.owner.clone(),
            docs: None,
            summary: t.summary.clone(),
            recommended_flows: t.recommended_flows.clone(),
            depends_on: vec![],
        })
        .collect();

    // Create HintEngine
    let engine = HintEngine::new(ac_index, runtime_tasks.clone());
    let hint_engine_hints = engine.task_hints();

    // Convert to CLI hints and apply filters
    let mut cli_hints: Vec<CliHint> = hint_engine_hints
        .iter()
        .filter_map(|hint| {
            // Only include Todo and InProgress hints (HintEngine filters these)
            if !matches!(hint.status, hints::HintStatus::Open | hints::HintStatus::InProgress) {
                return None;
            }

            let task_id = match &hint.target {
                hints::HintTarget::Task { id } => id.clone(),
                _ => return None,
            };

            // Find the original task definition for metadata
            let task_def = runtime_tasks.iter().find(|t| t.id == task_id)?;

            // Apply owner filter
            if let Some(ref owner_filter) = args.owner {
                let hint_owner = task_def.owner.as_deref().unwrap_or("unassigned");
                if !hint_owner.eq_ignore_ascii_case(owner_filter) {
                    return None;
                }
            }

            // Apply label filter
            if let Some(ref label_filter) = args.label
                && !hint.tags.iter().any(|l| l.eq_ignore_ascii_case(label_filter))
            {
                return None;
            }

            // Apply requirement filter
            if let Some(ref req_filter) = args.requirement
                && !task_def.requirement.eq_ignore_ascii_case(req_filter)
            {
                return None;
            }

            Some(CliHint {
                task_id,
                title: hint.title.clone(),
                status: match hint.status {
                    hints::HintStatus::Open => "open".to_string(),
                    hints::HintStatus::InProgress => "in_progress".to_string(),
                    hints::HintStatus::Done => "done".to_string(),
                },
                owner: task_def.owner.clone().unwrap_or_else(|| "unassigned".to_string()),
                labels: hint.tags.clone(),
                requirement_ids: vec![task_def.requirement.clone()],
                ac_ids: task_def.acs.clone(),
                reason: hint.reason.details.clone(),
            })
        })
        .collect();

    // Sort by status (in_progress before open) then by ID
    cli_hints.sort_by(|a, b| {
        let status_order_a = if a.status == "in_progress" { 0 } else { 1 };
        let status_order_b = if b.status == "in_progress" { 0 } else { 1 };

        match status_order_a.cmp(&status_order_b) {
            std::cmp::Ordering::Equal => a.task_id.cmp(&b.task_id),
            other => other,
        }
    });

    // Apply limit
    if let Some(limit) = args.limit {
        cli_hints.truncate(limit);
    }

    // Output based on format
    if args.format == "json" {
        let response = CliHintsResponse { hints: cli_hints };
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("{}", "Agent Hints".bold());
        println!();

        if cli_hints.is_empty() {
            println!("No hints match the current filters.");
            return Ok(());
        }

        for (idx, hint) in cli_hints.iter().enumerate() {
            let status_badge = if hint.status == "in_progress" {
                "[IN PROGRESS]".yellow()
            } else {
                "[OPEN]".cyan()
            };

            println!("{} {} ({})", idx + 1, status_badge, hint.task_id.bold());
            println!("  Title: {}", hint.title);
            println!("  Owner: {}", hint.owner);
            if !hint.labels.is_empty() {
                println!("  Labels: {}", hint.labels.join(", ").dimmed());
            }
            println!("  Reason: {}", hint.reason);
            println!("  ACs: {}", hint.ac_ids.join(", ").dimmed());
            println!();
        }
    }

    Ok(())
}
