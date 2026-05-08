use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use reqwest::blocking::Client;
use serde_json::{Value, json};

const DEFAULT_PLATFORM_URL: &str = "http://localhost:8080";
const DEFAULT_PORT_API_URL: &str = "https://api.getport.io/v1";
const DEFAULT_BLUEPRINT_ID: &str = "rust_service";

#[derive(Debug, Clone)]
pub struct PortSyncArgs {
    pub verbose: bool,
    pub force: bool,
    pub dump_only: bool,
}

pub fn run(args: PortSyncArgs) -> Result<()> {
    let platform_url = env_or_default("PLATFORM_URL", DEFAULT_PLATFORM_URL);
    let port_api_url = env_or_default("PORT_API_URL", DEFAULT_PORT_API_URL);
    let blueprint_id = env_or_default("PORT_BLUEPRINT_ID", DEFAULT_BLUEPRINT_ID);
    let client = Client::builder()
        .user_agent(format!("rust-as-spec-xtask/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to build HTTP client")?;

    if args.force && args.verbose {
        eprintln!("--force requested; Port upsert is idempotent, so a full sync is always sent");
    }

    if args.verbose || args.dump_only {
        eprintln!("Fetching from {platform_url}...");
    }

    let snapshot = fetch_json(&client, &format!("{platform_url}/platform/idp/snapshot"))
        .with_context(|| format!("could not fetch IDP snapshot from {platform_url}"))?;
    let status = fetch_json(&client, &format!("{platform_url}/platform/status"))
        .with_context(|| format!("could not fetch platform status from {platform_url}"))?;

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
    }

    let entity = transform_to_port_entity(&snapshot, &status, &platform_url, &blueprint_id);

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
    eprintln!("Syncing '{identifier}' to Port.io...");
    let token = get_port_token(&client, &port_api_url)?;
    let result =
        upsert_entity(&client, &port_api_url, &blueprint_id, &token, &entity, args.verbose)?;

    eprintln!(
        "Success! Entity synced: {}",
        result.pointer("/entity/identifier").and_then(Value::as_str).unwrap_or(identifier)
    );
    Ok(())
}

fn env_or_default(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn fetch_json(client: &Client, url: &str) -> Result<Value> {
    let response = client.get(url).send().with_context(|| format!("request failed: {url}"))?;
    let status = response.status();
    let body = response.text().context("failed to read response body")?;
    if !status.is_success() {
        bail!("HTTP {status}: {body}");
    }
    serde_json::from_str(&body).with_context(|| format!("failed to parse JSON from {url}"))
}

fn get_port_token(client: &Client, port_api_url: &str) -> Result<String> {
    let client_id = std::env::var("PORT_CLIENT_ID").context("PORT_CLIENT_ID must be set")?;
    let client_secret =
        std::env::var("PORT_CLIENT_SECRET").context("PORT_CLIENT_SECRET must be set")?;
    let url = format!("{port_api_url}/auth/access_token");
    let response = client
        .post(&url)
        .json(&json!({ "clientId": client_id, "clientSecret": client_secret }))
        .send()
        .with_context(|| format!("request failed: {url}"))?;
    let status = response.status();
    let body = response.text().context("failed to read Port token response body")?;
    if !status.is_success() {
        bail!("HTTP {status}: {body}");
    }
    let parsed: Value =
        serde_json::from_str(&body).context("failed to parse Port token response")?;
    parsed
        .get("accessToken")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .context("Port token response did not include accessToken")
}

fn upsert_entity(
    client: &Client,
    port_api_url: &str,
    blueprint_id: &str,
    token: &str,
    entity: &Value,
    verbose: bool,
) -> Result<Value> {
    let url = format!("{port_api_url}/blueprints/{blueprint_id}/entities");
    let response = client
        .post(&url)
        .bearer_auth(token)
        .query(&[("upsert", "true")])
        .json(entity)
        .send()
        .with_context(|| format!("request failed: {url}"))?;
    let status = response.status();
    let body = response.text().context("failed to read Port upsert response body")?;

    if verbose {
        eprintln!("Port.io response: {status}");
        if let Ok(parsed) = serde_json::from_str::<Value>(&body) {
            eprintln!("{}", serde_json::to_string_pretty(&parsed)?);
        } else {
            eprintln!("{body}");
        }
    }

    if !status.is_success() {
        bail!("HTTP {status}: {body}");
    }
    serde_json::from_str(&body).context("failed to parse Port upsert response")
}

fn transform_to_port_entity(
    snapshot: &Value,
    status: &Value,
    platform_url: &str,
    blueprint_id: &str,
) -> Value {
    let total = u64_at(snapshot, "/governance_health/ac_coverage/total");
    let passing = u64_at(snapshot, "/governance_health/ac_coverage/passing");
    let coverage_pct =
        if total > 0 { ((passing as f64 / total as f64) * 1000.0).round() / 10.0 } else { 0.0 };
    let policy_status =
        status.pointer("/governance/policies/status").and_then(Value::as_str).unwrap_or("unknown");
    let service_id = snapshot
        .get("service_id")
        .and_then(Value::as_str)
        .or_else(|| status.pointer("/service/service_id").and_then(Value::as_str))
        .or_else(|| status.get("service_id").and_then(Value::as_str))
        .unwrap_or("template");
    let template_version =
        snapshot.get("template_version").and_then(Value::as_str).unwrap_or("unknown");

    json!({
        "identifier": service_id,
        "title": format!("Rust-as-Spec: {service_id}"),
        "blueprint": blueprint_id,
        "properties": {
            "service_id": service_id,
            "template_version": template_version,
            "display_name": service_id,
            "description": format!("Platform cell running template v{template_version}"),
            "governance_passing": matches!(policy_status, "pass" | "passing"),
            "ac_coverage_percent": coverage_pct,
            "ac_total": total,
            "ac_passing": passing,
            "ac_failing": u64_at(snapshot, "/governance_health/ac_coverage/failing"),
            "docs_total": u64_at(snapshot, "/documentation/total"),
            "docs_valid": u64_at(snapshot, "/documentation/valid"),
            "docs_with_issues": u64_at(snapshot, "/documentation/with_issues"),
            "tasks_pending": u64_at(snapshot, "/task_hints/total_pending"),
            "tasks_in_progress": u64_at(snapshot, "/task_hints/total_in_progress"),
            "friction_count": u64_at(snapshot, "/task_hints/friction_count"),
            "platform_url": platform_url,
            "last_synced": Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        }
    })
}

fn u64_at(value: &Value, pointer: &str) -> u64 {
    value.pointer(pointer).and_then(Value::as_u64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_supports_current_status_shape() {
        let snapshot = json!({
            "service_id": "demo",
            "template_version": "3.3.15",
            "governance_health": { "ac_coverage": { "total": 4, "passing": 3, "failing": 1 } },
            "documentation": { "total": 2, "valid": 1, "with_issues": 1 },
            "task_hints": { "total_pending": 5, "total_in_progress": 6, "friction_count": 7 }
        });
        let status = json!({ "governance": { "policies": { "status": "pass" } } });

        let entity =
            transform_to_port_entity(&snapshot, &status, "http://localhost:8080", "rust_service");

        assert_eq!(entity["identifier"], "demo");
        assert_eq!(entity["blueprint"], "rust_service");
        assert_eq!(entity["properties"]["governance_passing"], true);
        assert_eq!(entity["properties"]["ac_coverage_percent"], 75.0);
        assert_eq!(entity["properties"]["docs_with_issues"], 1);
    }

    #[test]
    fn transform_falls_back_to_status_service_id() {
        let snapshot = json!({});
        let status = json!({ "service": { "service_id": "status-demo" } });

        let entity =
            transform_to_port_entity(&snapshot, &status, "http://localhost:8080", "rust_service");

        assert_eq!(entity["identifier"], "status-demo");
        assert_eq!(entity["properties"]["ac_coverage_percent"], 0.0);
    }
}
