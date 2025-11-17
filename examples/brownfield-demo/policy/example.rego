# Example Policy
# This is a sample Rego policy for demonstration purposes

package policies.example

# Deny rule example
deny[msg] {
    # Add your policy rules here
    false  # This will never trigger
    msg := "Example deny message"
}

# Allow rule example
allow {
    # Add your allow conditions here
    true
}
