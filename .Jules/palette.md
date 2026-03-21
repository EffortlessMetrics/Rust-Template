## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2026-03-21 - Javascript context inside Maud
**Learning:** In Maud templates, javascript inside `<script>` blocks needs to be wrapped in `maud::PreEscaped()`. If a template interpolates javascript directly via a variable/function (e.g. `script { (coverage_script()) }`), it will escape characters like `<` and `>` into `&lt;` and `&gt;`, breaking the javascript syntax.
**Action:** When inserting static javascript or CSS scripts using variable bindings, make sure it is wrapped as `script { (maud::PreEscaped(my_script_variable)) }` to prevent HTML escaping.