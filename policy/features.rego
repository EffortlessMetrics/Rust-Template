package features

# Feature-AC Linkage Policy
# Ensures features only reference ACs that exist in the ledger

deny[msg] {
    some i, j
    feature := input.features[i]
    ac := feature.acceptance_criteria[j]
    not ac_in_ledger(ac, input.ledger_ac_ids)
    msg := sprintf("Feature %s references unknown AC %s", [feature.id, ac])
}

ac_in_ledger(ac, ids) {
    ac == ids[_]
}
