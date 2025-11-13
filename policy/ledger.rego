package ledger

# Ledger Coverage Policy
# Ensures every AC has at least one mapped test

deny[msg] {
    some i, j, k
    story := input.stories[i]
    req := story.requirements[j]
    ac := req.acceptance_criteria[k]
    not ac_has_tests(ac)
    msg := sprintf("AC %s has no mapped tests", [ac.id])
}

ac_has_tests(ac) {
    ac.tests
    count(ac.tests) > 0
}
