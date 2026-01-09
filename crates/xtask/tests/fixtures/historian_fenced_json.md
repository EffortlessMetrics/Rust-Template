# Quality Assessment

This PR has a fenced JSON block which should be rejected.

## Analysis

The appendix contract requires raw JSON, not fenced blocks.

<!-- historian:appendix:start -->
```json
{
  "boundary_rating": "improved",
  "boundary_notes": ["This is wrapped in code fences"],
  "test_depth_rating": "mixed",
  "confidence": "medium"
}
```
<!-- historian:appendix:end -->
