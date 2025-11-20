# Governed Feature Development

**Skill:** governed-feature-dev  
**Purpose:** Implement a new feature following the AC-First workflow

---

## When to Use This Skill

Use this skill when asked to:
- Add a new feature
- Implement a requirement
- Build new functionality
- "Add code for X"

---

## Prerequisites

- Repository is governed (has `specs/spec_ledger.yaml`)
- Task exists in `specs/tasks.yaml` or can be discovered via `/platform/tasks`
- You have access to `cargo xtask` commands

---

## Workflow Steps

### 1. Check if REQ/AC exists

```bash
# Search for existing requirement
grep -r "description of feature" specs/spec_ledger.yaml

# Or query platform API
curl http://localhost:8080/platform/graph | jq
```

**Decision:**
- **If AC exists:** Proceed to step 2
- **If AC missing:** Ask user for details, then:
  ```bash
  cargo xtask ac-new AC-ID "Description" --story US-ID --requirement REQ-ID
  ```

### 2. Generate bounded context

```bash
cargo xtask bundle implement_ac
```

**Output:** `.llm/bundle/implement_ac.md`

Read this file to understand:
- Relevant requirements and ACs
- Existing code structure
- Related design docs

### 3. Write BDD scenario (Test-First)

Create or update `specs/features/*.feature`:

```gherkin
@AC-YOUR-ID
Scenario: Description matching AC text
  Given preconditions
  When action
  Then expected outcome
```

**Verify:**
```bash
cargo xtask bdd
# Expect: Failing test (not implemented yet)
```

### 4. Implement the feature

Write code in appropriate crate (`crates/business-core`, `crates/app-http`, etc.)

**Guidelines:**
- Follow hexagonal architecture (adapters → core)
- Keep business logic in `business-core`
- HTTP handlers in `app-http`
- Use existing patterns from bundle

### 5. Run tests

```bash
cargo xtask bdd
# Expect: Passing test
```

### 6. Full validation

```bash
cargo xtask selftest
```

**If selftest fails:**
- Read error output carefully
- Common issues:
  - Graph invariants (missing AC refs, unreachable commands)
  - Policy violations (see `cargo xtask policy-test`)
  - AC mapping (missing test tags)

**If selftest passes:** Feature is complete ✅

---

## Example Execution

```bash
# 1. User asks: "Add endpoint to list todos"

# 2. Check spec
grep -i "todo" specs/spec_ledger.yaml
# If missing:
cargo xtask ac-new AC-MYSERV-TODO-LIST \
  "GET /todos returns list of todos" \
  --story US-MYSERV-001 \
  --requirement REQ-MYSERV-TODOS

# 3. Bundle context
cargo xtask bundle implement_ac
cat .llm/bundle/implement_ac.md

# 4. Write BDD
cat >> specs/features/todos.feature <<EOF
@AC-MYSERV-TODO-LIST
Scenario: List todos
  Given I am an authenticated user
  When I GET /todos
  Then the response status should be 200
  And the response should contain a list of todos
EOF

# 5. Implement
# ... write code in crates/app-http/src/routes/todos.rs ...

# 6. Test
cargo xtask bdd

# 7. Validate
cargo xtask selftest
```

---

## Boundaries

**What this skill does:**
✅ Guide you through AC-first development  
✅ Ensure governance contracts are followed  
✅ Validate work via selftest

**What this skill does NOT do:**
❌ Generate code (you write the implementation)  
❌ Bypass governance (selftest is mandatory)  
❌ Make architecture decisions (those need ADRs)

---

## Error Recovery

**If selftest fails at step 6:**

1. **Read the error:**
   ```
   [5/7] Running policy tests...
     ✗ AC-MYSERV-TODO-LIST has no tests
   ```

2. **Fix the root cause:**
   - Add test tag to AC in `spec_ledger.yaml`
   - OR fix the BDD scenario tag

3. **Re-run:**
   ```bash
   cargo xtask selftest
   ```

**If you're stuck:**
- Ask human for guidance
- Check `docs/AGENT_GUIDE.md`
- Review similar ACs in `specs/spec_ledger.yaml`

---

## Success Criteria

Feature is complete when:
- ✅ AC exists in `spec_ledger.yaml`
- ✅ BDD scenario exists with matching tag
- ✅ Code implements the behavior
- ✅ `cargo xtask bdd` passes
- ✅ `cargo xtask selftest` passes

**Then:** Feature is ready for commit/PR.
