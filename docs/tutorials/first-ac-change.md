# Tutorial: Your First AC Change

This tutorial walks you through the complete AC-first development loop: from spec to code to test to verification.

**Time:** 15 minutes
**Prerequisites:** Template cloned, Nix devShell working

---

## The AC-First Loop

When developing with this template, follow this order:

1. **Spec**: Add or update AC in the ledger
2. **Scenario**: Create/update Gherkin scenario with `@AC-####`
3. **Code**: Implement the behavior
4. **Test**: Run acceptance tests
5. **Verify**: Check `feature_status.md`

Let's walk through each step.

---

## Step 1: Add AC to the Ledger

Open `specs/spec_ledger.yaml` and add a new AC:

```yaml
stories:
  - id: US-42
    requirements:
      - id: REQ-411
        acceptance_criteria:
          - id: AC-123
            text: "Customer can create a refund for an order"
            tests: [{ type: bdd, tag: "@AC-123" }]

          # Add new AC:
          - id: AC-124
            text: "Customer can view refund status"
            tests: [{ type: bdd, tag: "@AC-124" }]
```

**Key points:**
- Use next sequential AC number
- Write clear, testable behavior statement
- Reference the BDD tag you'll use in the scenario

---

## Complete Loop Summary

This tutorial demonstrates the AC-first approach. Always start with specs, then scenarios, then code.

For full examples and troubleshooting, see the template documentation.
