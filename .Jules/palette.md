## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-25 - Static Script Return Types in Maud
**Learning:** When injecting static, uninterpolated scripts or styles into a `maud` template from a helper function, the helper function should simply return `&'static str` using a raw string literal (`r#"..."#`). The Maud macro will correctly handle it. Attempting to manually wrap it in `maud::Markup` or `maud::PreEscaped` and returning that type can cause type mismatches and compiler errors if the call site expects a string.
**Action:** Always use `fn helper() -> &'static str { r#"..."# }` for static UI script/style injection, rather than dynamically allocating strings or changing return types to `Markup`.
