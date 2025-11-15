package template_core

# Template Core Protection Policy
#
# This policy ensures that template foundation elements cannot be accidentally removed.
# Template-core ACs (AC-TPL-*) and features (FT-TPL-CORE) are required for all services
# built from this template - they provide essential health/version endpoints and observability.
#
# See: docs/explanation/template-foundation-vs-examples.md

# Required core acceptance criteria that must always exist
required_core_acs := [
    "AC-TPL-001",  # Health check endpoint - essential for production monitoring
    "AC-TPL-002",  # Version information endpoint - essential for deployment tracking
]

# Deny if any core AC is missing from the ledger
deny[msg] {
    some required_ac
    required_core_acs[required_ac]
    not has_ac_in_ledger(required_ac, input.stories)
    msg := sprintf(
        "Template core AC %s must not be removed. This AC is part of the template foundation. See docs/explanation/template-foundation-vs-examples.md for details.",
        [required_ac]
    )
}

# Deny if core ACs exist but don't have tests mapped
deny[msg] {
    some i, j, k
    story := input.stories[i]
    req := story.requirements[j]
    ac := req.acceptance_criteria[k]
    is_core_ac(ac.id)
    not ac_has_tests(ac)
    msg := sprintf(
        "Template core AC %s must have at least one BDD test. The 'tests' field is either missing or empty. Core ACs enforce template foundation behavior and require test coverage. See docs/explanation/template-foundation-vs-examples.md for details.",
        [ac.id]
    )
}

# Deny if core ACs have empty tests array
deny[msg] {
    some i, j, k
    story := input.stories[i]
    req := story.requirements[j]
    ac := req.acceptance_criteria[k]
    is_core_ac(ac.id)
    ac.tests
    count(ac.tests) == 0
    msg := sprintf(
        "Template core AC %s has an empty 'tests' array. Core ACs must have at least one BDD test to verify template foundation behavior. Add a test entry with 'type: bdd' and appropriate '@%s' tag.",
        [ac.id, ac.id]
    )
}

# Deny if FT-TPL-CORE feature exists but doesn't reference all required core ACs
deny[msg] {
    some i
    feature := input.features[i]
    feature.id == "FT-TPL-CORE"
    some required_ac
    required_core_acs[required_ac]
    not ac_in_feature(required_ac, feature.acceptance_criteria)
    msg := sprintf(
        "FT-TPL-CORE must reference template core AC %s. This feature defines the template foundation and must include all core ACs: [%s]. Update features/FT-TPL-CORE.yaml to include this AC.",
        [required_ac, concat(", ", required_core_acs)]
    )
}

# Deny if FT-TPL-CORE feature exists but references non-core ACs
deny[msg] {
    some i, j
    feature := input.features[i]
    feature.id == "FT-TPL-CORE"
    ac := feature.acceptance_criteria[j]
    not is_core_ac(ac)
    msg := sprintf(
        "FT-TPL-CORE references non-core AC %s. This feature should only reference template core ACs: [%s]. Remove '%s' from features/FT-TPL-CORE.yaml or verify it's a core AC.",
        [ac, concat(", ", required_core_acs), ac]
    )
}

# Helper: Check if AC exists in ledger
has_ac_in_ledger(ac_id, stories) {
    some i, j, k
    stories[i].requirements[j].acceptance_criteria[k].id == ac_id
}

# Helper: Check if AC is a core template AC
is_core_ac(ac_id) {
    required_core_acs[ac_id]
}

# Helper: Check if AC has tests
ac_has_tests(ac) {
    ac.tests
    count(ac.tests) > 0
}

# Helper: Check if AC is in feature's acceptance_criteria list
ac_in_feature(ac_id, feature_acs) {
    ac_id == feature_acs[_]
}
