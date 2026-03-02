# http-task-board

SRP microcrate for rendering the `/ui/tasks` HTML board.

## Responsibility

- Convert governance tasks into status columns
- Render task cards and status transition buttons
- Output complete HTML document for task board

This crate intentionally contains **presentation-only** logic so request parsing,
I/O, and domain orchestration can remain in `http-tasks`.
