use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use clap::Parser;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::env;

const DEFAULT_PORT_API_URL: &str = "https://api.getport.io/v1";
const DEFAULT_PLATFORM_URL: &str = "http://localhost:8080";
const BLUEPRINT_ID: &str = "rust-template-service";

#[derive(Debug, Parser)]
#[command(about = "Sync Rust-as-Spec platform data to Port.io")]
struct Args {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Force full sync (accepted for compatibility; sync is already an upsert)
    #[arg(short, long)]
    force: bool,

    /// Fetch from platform and dump entity JSON without Port sync
    #[arg(short, long)]
    dump_only: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();
    let platform_url =
        env::var("PLATFORM_URL").unwrap_or_else(|_| DEFAULT_PLATFORM_URL.to_string());
    let port_api_url =
        env::var("PORT_API_URL").unwrap_or_else(|_| DEFAULT_PORT_API_URL.to_string());

    if args.verbose || args.dump_only {
        eprintln!("Fetching from {platform_url}...");
    }

    let snapshot = fetch_json(&client, &format!("{platform_url}/platform/idp/snapshot"))
        .context("failed to fetch IDP snapshot")?;
    let status = fetch_json(&client, &format!("{platform_url}/platform/status"))
        .context("failed to fetch platform status")?;

    if args.verbose {
        eprintln!(
            "Template version: {}",
            snapshot.get("template_version").and_then(Value::as_str).unwrap_or("unknown")
        );
        eprintln!(
            "Governance: {}",
            snapshot
                .pointer("/governance_health/status")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        );
        if args.force {
            eprintln!("Force requested; Port sync uses an idempotent upsert.");
        }
    }

    let entity = transform_to_port_entity(&snapshot, &status, &platform_url);

    if args.dump_only {
        println!("{}", serde_json::to_string_pretty(&entity)?);
        eprintln!(
            "--dump-only mode: entity '{}' dumped to stdout.",
            entity.get("identifier").and_then(Value::as_str).unwrap_or("unknown")
        );
        return Ok(());
    }

    if args.verbose {
        eprintln!("Entity: {}", serde_json::to_string_pretty(&entity)?);
    }

    let identifier = entity.get("identifier").and_then(Value::as_str).unwrap_or("unknown");
    eprintln!("Syncing {identifier} to Port.io...");
    let token = get_port_token(&client, &port_api_url)?;
    let result = upsert_entity(&client, &port_api_url, &token, &entity, args.verbose)?;

    let synced_identifier =
        result.pointer("/entity/identifier").and_then(Value::as_str).unwrap_or(identifier);
    eprintln!("Success! Entity synced: {synced_identifier}");
    Ok(())
}

fn fetch_json(client: &Client, url: &str) -> Result<Value> {
    let response = client.get(url).send().with_context(|| {
        format!(
            "could not connect to platform; ensure it is running: cargo run -p app-http ({url})"
        )
    })?;
    let status = response.status();
    let body = response.text().context("failed to read response body")?;
    if !status.is_success() {
        bail!("HTTP {status}: {body}");
    }
    serde_json::from_str(&body).context("failed to parse JSON response")
}

fn get_port_token(client: &Client, port_api_url: &str) -> Result<String> {
    let client_id = env::var("PORT_CLIENT_ID").context("PORT_CLIENT_ID must be set")?;
    let client_secret = env::var("PORT_CLIENT_SECRET").context("PORT_CLIENT_SECRET must be set")?;

    let response = client
        .post(format!("{port_api_url}/auth/access_token"))
        .json(&json!({ "clientId": client_id, "clientSecret": client_secret }))
        .send()
        .context("failed to request Port.io access token")?;
    let status = response.status();
    let body = response.text().context("failed to read Port.io token response")?;
    if !status.is_success() {
        bail!("HTTP {status}: {body}");
    }

    let token_response: Value =
        serde_json::from_str(&body).context("failed to parse token JSON")?;
    token_response
        .get("accessToken")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .context("Port.io token response did not include accessToken")
}

fn upsert_entity(
    client: &Client,
    port_api_url: &str,
    token: &str,
    entity: &Value,
    verbose: bool,
) -> Result<Value> {
    let response = client
        .post(format!("{port_api_url}/blueprints/{BLUEPRINT_ID}/entities"))
        .bearer_auth(token)
        .query(&[("upsert", "true")])
        .json(entity)
        .send()
        .context("failed to upsert Port.io entity")?;
    let status = response.status();
    let body = response.text().context("failed to read Port.io response")?;

    if verbose {
        eprintln!("Port.io response: {status}");
        if let Ok(json) = serde_json::from_str::<Value>(&body) {
            eprintln!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            eprintln!("{body}");
        }
    }

    if !status.is_success() {
        bail!("HTTP {status}: {body}");
    }
    serde_json::from_str(&body).context("failed to parse Port.io response JSON")
}

fn transform_to_port_entity(snapshot: &Value, status: &Value, platform_url: &str) -> Value {
    let ac_coverage = snapshot.pointer("/governance_health/ac_coverage").unwrap_or(&Value::Null);
    let docs = snapshot.get("documentation").unwrap_or(&Value::Null);
    let task_hints = snapshot.get("task_hints").unwrap_or(&Value::Null);

    let total = ac_coverage.get("total").and_then(Value::as_f64).unwrap_or(0.0);
    let passing = ac_coverage.get("passing").and_then(Value::as_f64).unwrap_or(0.0);
    let coverage_pct = if total > 0.0 { passing / total * 100.0 } else { 0.0 };

    let governance_passing =
        status.pointer("/governance/policies/status").and_then(Value::as_str) == Some("passing");

    let service_id = snapshot
        .get("service_id")
        .and_then(Value::as_str)
        .or_else(|| status.get("service_id").and_then(Value::as_str))
        .unwrap_or("template");
    let template_version =
        snapshot.get("template_version").and_then(Value::as_str).unwrap_or("unknown");

    json!({
        "identifier": service_id,
        "title": format!("Rust-as-Spec: {service_id}"),
        "blueprint": BLUEPRINT_ID,
        "properties": {
            "service_id": service_id,
            "template_version": template_version,
            "display_name": service_id,
            "description": format!("Platform cell running template v{template_version}"),
            "governance_passing": governance_passing,
            "ac_coverage_percent": (coverage_pct * 10.0).round() / 10.0,
            "ac_total": ac_coverage.get("total").and_then(Value::as_u64).unwrap_or(0),
            "ac_passing": ac_coverage.get("passing").and_then(Value::as_u64).unwrap_or(0),
            "ac_failing": ac_coverage.get("failing").and_then(Value::as_u64).unwrap_or(0),
            "docs_total": docs.get("total").and_then(Value::as_u64).unwrap_or(0),
            "docs_valid": docs.get("valid").and_then(Value::as_u64).unwrap_or(0),
            "docs_with_issues": docs.get("with_issues").and_then(Value::as_u64).unwrap_or(0),
            "tasks_pending": task_hints.get("total_pending").and_then(Value::as_u64).unwrap_or(0),
            "tasks_in_progress": task_hints.get("total_in_progress").and_then(Value::as_u64).unwrap_or(0),
            "friction_count": task_hints.get("friction_count").and_then(Value::as_u64).unwrap_or(0),
            "platform_url": platform_url,
            "last_synced": Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        }
    })
}
