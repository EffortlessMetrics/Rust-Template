//! Show environment detection mode for debugging CI/automation contexts.
//!
//! This command displays the current environment mode based on the centralized
//! detection in `crate::env`. Useful for:
//! - Debugging why BDD tests are being skipped
//! - Verifying CI detection is working
//! - Understanding which env vars are being honored

use anyhow::Result;
use colored::Colorize;
use std::env;

use crate::env::{describe_mode, is_ci, is_low_resources, is_noninteractive, should_skip_bdd};

pub struct EnvModeArgs {
    pub json: bool,
}

pub fn run(args: EnvModeArgs) -> Result<()> {
    let mode = describe_mode();
    let ci = is_ci();
    let noninteractive = is_noninteractive();
    let low_resources = is_low_resources();
    let skip_bdd = should_skip_bdd();

    // Collect raw env var values for debugging
    let ci_var = env::var("CI").ok();
    let github_actions = env::var("GITHUB_ACTIONS").ok();
    let gitlab_ci = env::var("GITLAB_CI").ok();
    let noninteractive_var = env::var("XTASK_NONINTERACTIVE").ok();
    let low_resources_var = env::var("XTASK_LOW_RESOURCES").ok();
    let skip_bdd_var = env::var("XTASK_SKIP_BDD").ok();
    let in_nix_shell = env::var("IN_NIX_SHELL").ok();

    if args.json {
        let output = serde_json::json!({
            "mode": mode,
            "flags": {
                "is_ci": ci,
                "is_noninteractive": noninteractive,
                "is_low_resources": low_resources,
                "should_skip_bdd": skip_bdd
            },
            "env_vars": {
                "CI": ci_var,
                "GITHUB_ACTIONS": github_actions,
                "GITLAB_CI": gitlab_ci,
                "XTASK_NONINTERACTIVE": noninteractive_var,
                "XTASK_LOW_RESOURCES": low_resources_var,
                "XTASK_SKIP_BDD": skip_bdd_var,
                "IN_NIX_SHELL": in_nix_shell
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", "Environment Mode".bold());
        println!("  Mode: {}", mode.green());
        println!();

        println!("{}", "Detection Flags".bold());
        println!("  is_ci:            {}", format_bool(ci));
        println!("  is_noninteractive: {}", format_bool(noninteractive));
        println!("  is_low_resources: {}", format_bool(low_resources));
        println!("  should_skip_bdd:  {}", format_bool(skip_bdd));
        println!();

        println!("{}", "Raw Environment Variables".bold());
        println!("  CI=               {}", format_var(&ci_var));
        println!("  GITHUB_ACTIONS=   {}", format_var(&github_actions));
        println!("  GITLAB_CI=        {}", format_var(&gitlab_ci));
        println!("  XTASK_NONINTERACTIVE= {}", format_var(&noninteractive_var));
        println!("  XTASK_LOW_RESOURCES=  {}", format_var(&low_resources_var));
        println!("  XTASK_SKIP_BDD=       {}", format_var(&skip_bdd_var));
        println!("  IN_NIX_SHELL=         {}", format_var(&in_nix_shell));
    }

    Ok(())
}

fn format_bool(value: bool) -> String {
    if value { "true".green().to_string() } else { "false".dimmed().to_string() }
}

fn format_var(value: &Option<String>) -> String {
    match value {
        Some(v) => format!("\"{}\"", v).cyan().to_string(),
        None => "(unset)".dimmed().to_string(),
    }
}
