package main

# Privacy Policy
# Ensures PII fields have owners and valid retention periods

# PII fields must have an owner
deny[msg] {
    some i
    field := input.fields[i]
    is_pii(field)
    not field.owner
    msg := sprintf("PII field %s is missing owner", [field.path])
}

deny[msg] {
    some i
    field := input.fields[i]
    is_pii(field)
    field.owner == ""
    msg := sprintf("PII field %s has empty owner", [field.path])
}

# PII fields must have valid retention period
deny[msg] {
    some i
    field := input.fields[i]
    is_pii(field)
    not field.retention
    msg := sprintf("PII field %s is missing retention period", [field.path])
}

deny[msg] {
    some i
    field := input.fields[i]
    is_pii(field)
    field.retention
    not valid_retention_format(field.retention)
    msg := sprintf("PII field %s has invalid retention format: %s (expected \\d+[dwmy])", [field.path, field.retention])
}

# Helper: Check if field is PII
is_pii(field) {
    field.classification == "PII"
}

is_pii(field) {
    field.pii != ""
    field.pii != false
}

# Helper: Validate retention format (e.g., "365d", "12m", "5y")
valid_retention_format(retention) {
    regex.match(`^\d+[dwmy]$`, retention)
}
