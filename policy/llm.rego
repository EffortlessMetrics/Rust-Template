package llm

# LLM Contextpack Validation Policy
#
# This policy validates the structure of .llm/contextpack.yaml to ensure:
# - All tasks have valid configuration
# - Required tasks referenced in documentation exist
# - No invalid/unknown fields are present in task definitions
#
# See: .llm/contextpack.yaml

# Required task names that must exist (referenced in project documentation)
required_tasks := [
    "implement_ac",      # Used for implementing acceptance criteria
    "implement_feature", # Used for broader feature development
    "debug_tests",       # Used for debugging test failures
]

# Valid fields allowed in a task definition
valid_task_fields := {
    "max_bytes",
    "include",
    "description",
}

# Deny if any required task is missing
deny[msg] {
    some required_task
    required_tasks[required_task]
    not has_task(required_task, input.tasks)
    msg := sprintf(
        "Required task '%s' is missing. This task is referenced in documentation and must exist.",
        [required_task]
    )
}

# Deny if a task has max_bytes <= 0
deny[msg] {
    some task_name
    task := input.tasks[task_name]
    task.max_bytes <= 0
    msg := sprintf(
        "Task '%s' has invalid max_bytes=%d. Value must be greater than 0.",
        [task_name, task.max_bytes]
    )
}

# Deny if a task has no max_bytes field
deny[msg] {
    some task_name
    task := input.tasks[task_name]
    not task.max_bytes
    msg := sprintf(
        "Task '%s' is missing required field 'max_bytes'.",
        [task_name]
    )
}

# Deny if a task has an empty include list
deny[msg] {
    some task_name
    task := input.tasks[task_name]
    task.include
    count(task.include) == 0
    msg := sprintf(
        "Task '%s' has empty include list. At least one pattern must be specified.",
        [task_name]
    )
}

# Deny if a task has no include field
deny[msg] {
    some task_name
    task := input.tasks[task_name]
    not task.include
    msg := sprintf(
        "Task '%s' is missing required field 'include'.",
        [task_name]
    )
}

# Deny if a task contains unknown/invalid fields
deny[msg] {
    some task_name
    task := input.tasks[task_name]
    some field_name
    task[field_name]
    not valid_task_fields[field_name]
    msg := sprintf(
        "Task '%s' contains unknown field '%s'. Valid fields are: %s",
        [task_name, field_name, concat(", ", valid_task_fields)]
    )
}

# Deny if include list contains non-string values
deny[msg] {
    some task_name
    task := input.tasks[task_name]
    task.include
    some i
    pattern := task.include[i]
    not is_string(pattern)
    msg := sprintf(
        "Task '%s' has non-string pattern at index %d in include list.",
        [task_name, i]
    )
}

# Helper: Check if task exists
has_task(task_name, tasks) {
    tasks[task_name]
}
