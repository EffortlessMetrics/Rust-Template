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

#[cfg(test)]
mod tests {
    use super::generate_run_id;

    #[test]
    fn generates_pr_suffix_for_some() {
        let run_id = generate_run_id(Some(42));
        assert!(run_id.ends_with("-pr42"));
    }

    #[test]
    fn defaults_to_pr0_for_none() {
        let run_id = generate_run_id(None);
        assert!(run_id.ends_with("-pr0"));
    }
}
