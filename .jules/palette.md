## $(date +%Y-%m-%d) - Symmetrical UI Changes
**Learning:** The UI codebase is duplicated across `crates/app-http/src/platform/ui.rs` and `crates/http-platform/src/ui.rs`. When making UI enhancements like adding ARIA labels or changing input semantics, changes must be applied symmetrically to both files to maintain consistency.
**Action:** Always search (`grep`) across both UI directories when modifying Maud templates to ensure the changes are mirrored.
