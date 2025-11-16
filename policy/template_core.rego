package main

# Template Core Policy
# Ensures core ACs exist and FT-TPL-CORE feature references all core ACs

# Required core AC IDs
required_core_acs := {"AC-TPL-CORE-001", "AC-TPL-CORE-002"}

# FT-TPL-CORE feature must exist
deny[msg] {
    not has_tpl_core_feature
    msg := "Feature 'FT-TPL-CORE' is required"
}

# All required core ACs must exist in the ledger
deny[msg] {
    core_ac := required_core_acs[_]
    not ac_in_ledger_ids(core_ac)
    msg := sprintf("Required core AC '%s' is not in ledger_ac_ids", [core_ac])
}

# All required core ACs must exist in stories
deny[msg] {
    core_ac := required_core_acs[_]
    not ac_exists_in_stories(core_ac)
    msg := sprintf("Required core AC '%s' does not exist in stories", [core_ac])
}

# All required core ACs must have tests
deny[msg] {
    core_ac := required_core_acs[_]
    ac_exists_in_stories(core_ac)
    ac := get_ac_from_stories(core_ac)
    not has_tests(ac)
    msg := sprintf("Required core AC '%s' must have a non-empty 'tests' array", [core_ac])
}

# FT-TPL-CORE must reference all required core ACs
deny[msg] {
    has_tpl_core_feature
    core_ac := required_core_acs[_]
    not ac_in_tpl_core_feature(core_ac)
    msg := sprintf("Feature 'FT-TPL-CORE' must reference core AC '%s'", [core_ac])
}

# FT-TPL-CORE must not reference non-core ACs
deny[msg] {
    has_tpl_core_feature
    feature := input.features[_]
    feature.id == "FT-TPL-CORE"
    referenced_ac := feature.acceptance_criteria[_]
    not is_core_ac(referenced_ac)
    msg := sprintf("Feature 'FT-TPL-CORE' references non-core AC '%s' (only core ACs allowed)", [referenced_ac])
}

# Helper: Check if feature FT-TPL-CORE exists
has_tpl_core_feature {
    feature := input.features[_]
    feature.id == "FT-TPL-CORE"
}

# Helper: Check if AC is in ledger_ac_ids array
ac_in_ledger_ids(ac_id) {
    input.ledger_ac_ids[_] == ac_id
}

# Helper: Check if AC exists in stories
ac_exists_in_stories(ac_id) {
    story := input.stories[_]
    req := story.requirements[_]
    ac := req.acceptance_criteria[_]
    ac.id == ac_id
}

# Helper: Get AC from stories by ID
get_ac_from_stories(ac_id) := ac {
    story := input.stories[_]
    req := story.requirements[_]
    ac := req.acceptance_criteria[_]
    ac.id == ac_id
}

# Helper: Check if AC has tests
has_tests(ac) {
    is_array(ac.tests)
    count(ac.tests) > 0
}

# Helper: Check if AC is in FT-TPL-CORE feature
ac_in_tpl_core_feature(ac_id) {
    feature := input.features[_]
    feature.id == "FT-TPL-CORE"
    feature.acceptance_criteria[_] == ac_id
}

# Helper: Check if AC is a core AC
is_core_ac(ac_id) {
    required_core_acs[ac_id]
}
