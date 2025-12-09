use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use serde::Serialize;
use spec_runtime::hints::{self, HintEngine};

use super::tasks::spec_root;

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

/// Warning about referential integrity issues in CLI output
#[derive(Debug, Serialize)]
pub struct CliWarning {
    pub invalid_id: String,
    pub ref_type: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct CliHintsResponse {
    pub hints: Vec<CliHint>,
    /// Warnings about referential integrity issues (invalid AC/REQ references)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<CliWarning>,
}

pub fn run(args: SuggestNextArgs) -> Result<()> {
    let root = spec_root();

    // Load tasks and AC coverage
    let tasks_path = root.join("specs/tasks.yaml");
    let task_specs = spec_runtime::load_tasks(&tasks_path)?;

    let feature_status_path = root.join("docs/feature_status.md");
    let ac_index = hints::parse_feature_status(&feature_status_path)?;

    // Load spec_ledger for referential integrity validation (AC-TPL-HINTS-REFERENTIAL-INTEGRITY)
    let ledger_path = root.join("specs/spec_ledger.yaml");
    let ledger = spec_runtime::load_spec_ledger(&ledger_path)?;
    let valid_ac_ids = spec_runtime::build_ac_id_index(&ledger);
    let valid_req_ids = spec_runtime::build_req_id_index(&ledger);

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

    // Create HintEngine with referential integrity validation
    let mut engine = HintEngine::with_validation(
        ac_index.clone(),
        runtime_tasks.clone(),
        valid_ac_ids,
        valid_req_ids,
    );

    // Build kernel AC statuses for governance hints (AC-TPL-HINTS-KERNEL-SIGNALS)
    let kernel_acs = spec_runtime::build_kernel_ac_statuses(&ledger, &ac_index);
    engine.set_kernel_acs(kernel_acs);

    let hint_engine_hints = engine.task_hints();
    let kernel_governance_hints = engine.kernel_governance_hints();

    // Collect any referential integrity warnings
    let warnings: Vec<CliWarning> = engine
        .warnings()
        .iter()
        .map(|w| CliWarning {
            invalid_id: w.invalid_id.clone(),
            ref_type: w.ref_type.clone(),
            source: w.source.clone(),
            message: w.message.clone(),
        })
        .collect();

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

    // Add kernel governance hints for failing kernel ACs (AC-TPL-HINTS-KERNEL-SIGNALS)
    // These are high-priority hints to fix failing kernel ACs before other work
    for hint in &kernel_governance_hints {
        let ac_id = match &hint.target {
            hints::HintTarget::Ac { id } => id.clone(),
            _ => continue,
        };

        cli_hints.push(CliHint {
            task_id: ac_id.clone(), // Use AC ID as task_id for compatibility
            title: hint.title.clone(),
            status: "open".to_string(),
            owner: "kernel".to_string(), // Kernel ACs are owned by the kernel
            labels: hint.tags.clone(),
            requirement_ids: vec![], // Kernel hints don't have explicit requirements
            ac_ids: vec![ac_id],
            reason: hint.reason.details.clone(),
        });
    }

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
        let response = CliHintsResponse { hints: cli_hints, warnings };
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        // Show warnings first if any
        if !warnings.is_empty() {
            println!("{}", "Referential Integrity Warnings".yellow().bold());
            println!();
            for warn in &warnings {
                println!("  {} {}: {}", "[WARN]".yellow(), warn.source, warn.message);
            }
            println!();
        }

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
