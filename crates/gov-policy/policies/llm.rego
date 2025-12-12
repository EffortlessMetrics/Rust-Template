package main

# LLM Contextpack Policy
# Ensures .llm/contextpack.yaml has required structure

# Required task names
required_tasks := {"implement_ac", "selftest"}

# Allowed task fields
allowed_task_fields := {"max_bytes", "include", "description"}

# All required tasks must exist
deny[msg] {
    task := required_tasks[_]
    not input.tasks[task]
    msg := sprintf("Required task '%s' is missing from contextpack", [task])
}

# Each task must have max_bytes field
deny[msg] {
    task_name := input.tasks[_]
    task := input.tasks[task_name]
    not task.max_bytes
    msg := sprintf("Task '%s' is missing required field 'max_bytes'", [task_name])
}

# Each task must have max_bytes > 0
deny[msg] {
    task_name := input.tasks[_]
    task := input.tasks[task_name]
    task.max_bytes
    task.max_bytes <= 0
    msg := sprintf("Task '%s' must have max_bytes > 0 (found: %d)", [task_name, task.max_bytes])
}

# Each task must have include field
deny[msg] {
    task_name := input.tasks[_]
    task := input.tasks[task_name]
    not task.include
    msg := sprintf("Task '%s' is missing required field 'include'", [task_name])
}

# Each task must have non-empty include array
deny[msg] {
    task_name := input.tasks[_]
    task := input.tasks[task_name]
    task.include
    not is_array(task.include)
    msg := sprintf("Task '%s' field 'include' must be an array", [task_name])
}

deny[msg] {
    task_name := input.tasks[_]
    task := input.tasks[task_name]
    is_array(task.include)
    count(task.include) == 0
    msg := sprintf("Task '%s' must have non-empty 'include' array", [task_name])
}

# Each include entry must be a string
deny[msg] {
    task_name := input.tasks[_]
    task := input.tasks[task_name]
    is_array(task.include)
    entry := task.include[_]
    not is_string(entry)
    msg := sprintf("Task '%s' include array must contain only strings", [task_name])
}

# No unknown fields in tasks
deny[msg] {
    task_name := input.tasks[_]
    task := input.tasks[task_name]
    field := object.keys(task)[_]
    not allowed_task_fields[field]
    msg := sprintf("Task '%s' has unknown field '%s' (allowed: max_bytes, include, description)", [task_name, field])
}
