## Investigation Report: Issue #27 - Type Safety with Newtypes

### Status
**Status:** fix-ready - Clear pattern exists, migration needed
**Local gates:** Code audit completed

### Evidence

**String IDs needing newtypes:**
- `spec-runtime::Task { id: String, requirement: String, acs: Vec<String> }`
- `spec-runtime::Ledger { Story.id: String, Requirement.id: String, AC.id: String }`
- `model::ExampleTask { id: String }`, `model::Todo { id: String }`
- `app-http::AgentHint { status: String }`

**Existing pattern (EXCELLENT):**
- `gov-model::TaskId(pub String)` - with Display, FromStr, serde
- `gov-model::TaskStatus` enum - with alias parsing and transition validation

**Result<T, String> patterns:**
- `business-core::ExampleTaskRepository` - 4 methods return `Result<_, String>`

### Plan

**Phase 1:** Extend gov-model with `RequirementId`, `AcId`, `StoryId`
**Phase 2:** Migrate spec-runtime to use typed IDs
**Phase 3:** Update business-core with typed errors
**Phase 4:** Update HTTP layer serialization

**Effort:** 13-18 hours (aligns with issue estimate)

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. Related to #18 (error strategy) and #55 (typed errors).
