use anyhow::{Context, Result};
use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Regex pattern for parsing feature_status.md AC lines
static AC_STATUS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\|\s*AC-[A-Z0-9-]+\s*\|.*\|\s*(✅|❌|❓)\s*(pass|fail|unknown)\s*\|").unwrap()
});

// Ledger structures
#[derive(Debug, Deserialize)]
struct Ledger {
    metadata: LedgerMetadata,
    stories: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    template_version: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Story {
    #[allow(dead_code)]
    id: String,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
struct Requirement {
    #[allow(dead_code)]
    id: String,
    acceptance_criteria: Vec<AcceptanceCriteria>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriteria {
    #[allow(dead_code)]
    id: String,
}

// Tasks structures
#[derive(Debug, Deserialize)]
struct TasksFile {
    tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Deserialize)]
struct TaskDefinition {
    #[allow(dead_code)]
    id: String,
    status: Option<String>,
}

// AC Coverage structures
#[derive(Debug, Default)]
struct AcCoverage {
    pass: usize,
    fail: usize,
    unknown: usize,
}

impl AcCoverage {
    fn total(&self) -> usize {
        self.pass + self.fail + self.unknown
    }

    #[allow(dead_code)]
    fn with_tests(&self) -> usize {
        self.pass + self.fail
    }
}

/// Display governance status dashboard
pub fn run() -> Result<()> {
    let ledger_path = Path::new("specs/spec_ledger.yaml");
    let tasks_path = Path::new("specs/tasks.yaml");
    let feature_status_path = Path::new("docs/feature_status.md");

    // Parse ledger
    let ledger = parse_ledger(ledger_path)?;
    let (story_count, req_count, ac_count) = count_governance(&ledger);

    // Parse tasks
    let task_counts = parse_tasks(tasks_path)?;

    // Parse AC coverage
    let ac_coverage = parse_ac_coverage(feature_status_path);

    // Display status
    print_status_dashboard(
        &ledger.metadata.template_version,
        story_count,
        req_count,
        ac_count,
        &task_counts,
        &ac_coverage,
    );

    Ok(())
}

fn parse_ledger(ledger_path: &Path) -> Result<Ledger> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger YAML: {}", ledger_path.display()))?;

    Ok(ledger)
}

fn count_governance(ledger: &Ledger) -> (usize, usize, usize) {
    let story_count = ledger.stories.len();
    let req_count: usize = ledger.stories.iter().map(|s| s.requirements.len()).sum();
    let ac_count: usize = ledger
        .stories
        .iter()
        .flat_map(|s| &s.requirements)
        .map(|r| r.acceptance_criteria.len())
        .sum();

    (story_count, req_count, ac_count)
}

fn parse_tasks(tasks_path: &Path) -> Result<HashMap<String, usize>> {
    if !tasks_path.exists() {
        // Tasks file is optional
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(tasks_path)
        .with_context(|| format!("Failed to read tasks: {}", tasks_path.display()))?;

    let tasks_file: TasksFile = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse tasks YAML: {}", tasks_path.display()))?;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for task in tasks_file.tasks {
        let status = task.status.unwrap_or_else(|| "open".to_string());
        *counts.entry(status).or_insert(0) += 1;
    }

    Ok(counts)
}

fn parse_ac_coverage(feature_status_path: &Path) -> Option<AcCoverage> {
    // If the file doesn't exist, AC coverage is not available
    if !feature_status_path.exists() {
        return None;
    }

    let content = match fs::read_to_string(feature_status_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let mut coverage = AcCoverage::default();

    for line in content.lines() {
        if let Some(caps) = AC_STATUS_PATTERN.captures(line) {
            // Extract status text (pass/fail/unknown)
            let status = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            match status {
                "pass" => coverage.pass += 1,
                "fail" => coverage.fail += 1,
                "unknown" => coverage.unknown += 1,
                _ => {}
            }
        }
    }

    // Only return coverage if we found at least one AC
    if coverage.total() > 0 { Some(coverage) } else { None }
}

fn print_status_dashboard(
    version: &str,
    story_count: usize,
    req_count: usize,
    ac_count: usize,
    task_counts: &HashMap<String, usize>,
    ac_coverage: &Option<AcCoverage>,
) {
    println!();
    println!("{}", "======================================".blue());
    println!("{}", format!("Rust-as-Spec – {}", version).bold());
    println!("{}", "======================================".blue());
    println!();

    // Governance metrics
    println!("{}", "Governance:".bold());
    println!("  Stories:      {}", story_count);
    println!("  Requirements: {}", req_count);
    println!("  ACs:          {}", ac_count);

    // AC Coverage metrics (integrated into Governance section)
    if let Some(coverage) = ac_coverage {
        // Build the coverage line with color based on failures
        let coverage_line = if coverage.fail > 0 {
            format!(
                "  AC coverage:  {} pass, {} fail, {} unknown (BDD)",
                coverage.pass, coverage.fail, coverage.unknown
            )
            .yellow()
            .to_string()
        } else if coverage.unknown == 0 && coverage.pass > 0 {
            format!(
                "  AC coverage:  {} pass, {} fail, {} unknown (BDD)",
                coverage.pass, coverage.fail, coverage.unknown
            )
            .green()
            .to_string()
        } else {
            format!(
                "  AC coverage:  {} pass, {} fail, {} unknown (BDD)",
                coverage.pass, coverage.fail, coverage.unknown
            )
        };

        println!("{}", coverage_line);
        println!("  See: {}", "docs/feature_status.md".blue());
    } else {
        // No AC coverage data available yet
        println!("  AC coverage:  N/A (run acceptance tests)");
    }
    println!();

    // Task metrics
    if !task_counts.is_empty() {
        println!("{}", "Tasks:".bold());
        for (status, count) in task_counts.iter() {
            let status_display = match status.as_str() {
                "open" => format!("Todo:        {}", count),
                "in_progress" => format!("InProgress:  {}", count).yellow().to_string(),
                "review" => format!("Review:      {}", count).cyan().to_string(),
                "done" => format!("Done:        {}", count).green().to_string(),
                _ => format!("{:12} {}", status, count),
            };
            println!("  {}", status_display);
        }
        println!();
    }

    // Suggested next steps
    println!("{}", "Next steps:".bold());
    println!("  • View tasks:     {}", "cargo xtask tasks-list".blue());
    println!("  • Run selftest:   {}", "cargo xtask selftest".blue());
    println!("  • Start platform: {}", "cargo run -p app-http".blue());
    println!("  • View UI:        {}", "http://localhost:8080/ui".blue());
    println!("{}", "======================================".blue());
    println!();
}
