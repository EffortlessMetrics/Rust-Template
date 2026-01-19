## 2024-05-22 - Implementing Server-Side Active Navigation States
**Learning:** Working with `maud` for conditional attributes (like `aria-current`) can be tricky with newer syntax. Using standard Rust control flow (`if/else`) inside a helper function is often more readable and robust than trying to force inline conditional attributes when type inference is difficult.
**Action:** When using `maud`, prefer helper functions with explicit control flow for complex attribute logic to maintain readability and avoid type tetris.
