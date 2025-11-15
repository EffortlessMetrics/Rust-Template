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
        "Template core AC %s must have mapped tests. Core ACs enforce template foundation behavior.",
        [ac.id]
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
