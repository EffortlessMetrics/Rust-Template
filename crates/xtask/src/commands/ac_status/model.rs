use ac_kernel::{Ac, AcSource, AcStatus};
use anyhow::Result;
use spec_runtime::ledger::TestMapping;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn parse_ledger(ledger_path: &Path) -> Result<HashMap<String, Ac>> {
    // Use ac-kernel's ledger parser for consistency
    let metadata = ac_kernel::parse_ledger_with_metadata(ledger_path)?;

    let mut acs = HashMap::new();
    for (id, m) in metadata {
        let tests_total = m.tests.iter().filter(|t| is_automated_test(t)).count();
        acs.insert(
            id.clone(),
            Ac {
                id,
                story_id: m.story_id,
                req_id: m.req_id,
                text: m.text,
                status: AcStatus::Unknown,
                source: AcSource::Inferred,
                scenarios: Vec::new(),
                tests: m.tests,
                tests_total,
                tests_executed: 0,
                tags: m.tags,
                must_have_ac: m.must_have_ac,
            },
        );
    }

    Ok(acs)
}

pub(super) fn is_automated_test(test: &TestMapping) -> bool {
    matches!(test.test_type.to_lowercase().as_str(), "unit" | "integration" | "bdd")
}

pub(super) fn is_meta_ac(ac: &Ac) -> bool {
    // Meta ACs are those with tags indicating they're test harness or example-level,
    // not service-level contracts
    ac.tags.iter().any(|t| matches!(t.as_str(), "harness" | "example" | "ci-only"))
        || ac.tests.iter().any(|t| t.test_type.eq_ignore_ascii_case("ci"))
}
