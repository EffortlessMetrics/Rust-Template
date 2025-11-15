package flags

# Feature Flags Policy
# Ensures flags have owners and rollouts reference valid flags

# Every flag must have an owner
deny[msg] {
    some i
    flag := input.flags[i]
    not flag.owner
    msg := sprintf("Flag %s has no owner", [flag.key])
}

deny[msg] {
    some i
    flag := input.flags[i]
    flag.owner == ""
    msg := sprintf("Flag %s has empty owner", [flag.key])
}

# Rollout percentages must be valid
deny[msg] {
    some env, flag_key
    input.rollouts[env][flag_key]
    percent := input.rollouts[env][flag_key]
    percent < 0
    msg := sprintf("Flag %s in %s has invalid percentage %d (< 0)", [flag_key, env, percent])
}

deny[msg] {
    some env, flag_key
    input.rollouts[env][flag_key]
    percent := input.rollouts[env][flag_key]
    percent > 100
    msg := sprintf("Flag %s in %s has invalid percentage %d (> 100)", [flag_key, env, percent])
}

# Rollouts must reference existing flags
deny[msg] {
    some env, flag_key
    input.rollouts[env][flag_key]
    not flag_exists(flag_key, input.flags)
    msg := sprintf("Rollout for %s in %s references unknown flag", [flag_key, env])
}

flag_exists(key, flags) {
    some i
    flags[i].key == key
}
