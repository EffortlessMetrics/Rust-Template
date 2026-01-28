use chrono::Utc;

/// Generate a consistent run_id for receipts.
/// Format: `{timestamp}-pr{pr_number}` or `{timestamp}-pr0` if no PR.
pub fn generate_run_id(pr: Option<u32>) -> String {
    format!(
        "{}-pr{}",
        Utc::now().format("%Y-%m-%dT%H-%M-%SZ"),
        pr.map(|n| n.to_string()).unwrap_or_else(|| "0".to_string())
    )
}
