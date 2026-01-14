package main

deny[msg] {
    # Iterate over each acceptance criterion
    ac := input.stories[_].requirements[_].acceptance_criteria[_]

    # Check if the 'tests' field is missing or is an empty array
    not has_tests(ac)

    # Format the error message
    msg := sprintf("AC '%s' must have a non-empty 'tests' array.", [ac.id])
}

# Helper function to check for a non-empty 'tests' array.
# is_array(ac.tests) handles the case where 'tests' is missing.
has_tests(ac) {
    is_array(ac.tests)
    count(ac.tests) > 0
}
