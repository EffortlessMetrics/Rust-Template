use anyhow::{Context, Result, bail};
use business_core::governance::TaskStatus;
use spec_runtime::{
    ledger::SpecLedger,
    load_spec_ledger, load_tasks,
    tasks::{Task, TaskDocs, TasksSpec},
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn spec_root() -> PathBuf {
    if let Ok(root) = std::env::var("SPEC_ROOT") {
        return PathBuf::from(root);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

pub fn create_task(
    id: &str,
    title: &str,
    requirement: &str,
    acs: &[String],
    owner: Option<String>,
    status: Option<String>,
    labels: &[String],
) -> Result<()> {
    let root = spec_root();
    let ledger = load_spec_ledger(&root.join("specs/spec_ledger.yaml"))?;

    validate_requirement_and_acs(&ledger, requirement, acs)?;

    let tasks_path = root.join("specs/tasks.yaml");
    let mut tasks_spec = load_tasks(&tasks_path)?;

    if tasks_spec.tasks.iter().any(|t| t.id == id) {
        bail!("Task {} already exists in {}", id, tasks_path.display());
    }

    let desired_status =
        status.as_deref().map(parse_status).transpose()?.unwrap_or(TaskStatus::Todo);

    let new_task = Task {
        id: id.to_string(),
        title: title.to_string(),
        requirement: requirement.to_string(),
        acs: acs.to_vec(),
        status: format_status(&desired_status),
        owner,
        labels: labels.to_vec(),
        docs: Some(TaskDocs { design: Vec::new(), plan: Vec::new() }),
        summary: title.to_string(),
        recommended_flows: Vec::new(),
    };

    tasks_spec.tasks.push(new_task);
    write_tasks(&tasks_path, &tasks_spec)?;

    println!(
        "Created task {} (requirement: {}, status: {})",
        id,
        requirement,
        format_status(&desired_status)
    );

    Ok(())
}

pub fn update_task(
    id: &str,
    title: Option<String>,
    owner: Option<String>,
    status: Option<String>,
) -> Result<()> {
    let root = spec_root();
    let tasks_path = root.join("specs/tasks.yaml");
    let mut tasks_spec = load_tasks(&tasks_path)?;

    let task = tasks_spec.tasks.iter_mut().find(|t| t.id == id).context(format!(
        "Task {} not found in {}",
        id,
        tasks_path.display()
    ))?;

    let current_status = parse_status(&task.status)?;

    if let Some(new_title) = title {
        task.title = new_title.clone();
        task.summary = new_title;
    }

    if let Some(new_owner) = owner {
        task.owner = Some(new_owner);
    }

    if let Some(status_text) = status {
        let new_status = parse_status(&status_text)?;
        if current_status != new_status && !current_status.can_transition_to(&new_status) {
            bail!(
                "Invalid status transition for {}: {} -> {}",
                id,
                format_status(&current_status),
                format_status(&new_status)
            );
        }
        task.status = format_status(&new_status);
    }

    write_tasks(&tasks_path, &tasks_spec)?;

    println!("Updated task {}", id);
    Ok(())
}

fn write_tasks(path: &Path, tasks: &TasksSpec) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(tasks)?;
    fs::write(path, yaml)?;
    Ok(())
}

fn parse_status(raw: &str) -> Result<TaskStatus> {
    match raw.trim().to_lowercase().as_str() {
        "todo" | "open" => Ok(TaskStatus::Todo),
        "inprogress" | "in_progress" | "in-progress" | "in progress" => Ok(TaskStatus::InProgress),
        "review" => Ok(TaskStatus::Review),
        "done" | "completed" => Ok(TaskStatus::Done),
        other => bail!("Invalid status '{}'. Use one of: Todo, InProgress, Review, Done.", other),
    }
}

fn format_status(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "InProgress",
        TaskStatus::Review => "Review",
        TaskStatus::Done => "Done",
    }
    .to_string()
}

fn validate_requirement_and_acs(
    ledger: &SpecLedger,
    requirement: &str,
    acs: &[String],
) -> Result<()> {
    let mut req_found = false;
    let mut ac_set: HashSet<&str> = HashSet::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            if req.id == requirement {
                req_found = true;
                for ac in &req.acceptance_criteria {
                    ac_set.insert(ac.id.as_str());
                }
            }
        }
    }

    if !req_found {
        bail!("Requirement {} not found in spec_ledger.yaml", requirement);
    }

    for ac in acs {
        if !ac_set.contains(ac.as_str()) {
            bail!("Acceptance criterion {} not found under requirement {}", ac, requirement);
        }
    }

    Ok(())
}
