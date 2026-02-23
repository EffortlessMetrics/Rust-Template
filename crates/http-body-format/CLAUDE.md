# http-body-format – CLAUDE.md

Focused classifier for HTTP request body formats based on `content-type`.

## Responsibilities

- Normalize `content-type` values (trim/case-insensitive/ignore parameters)
- Classify supported media types (`application/json`, `application/x-www-form-urlencoded`)
- Return deterministic `Unknown` for unsupported or missing values

## Non-responsibilities

- Request body parsing/decoding
- HTTP header map extraction
- Validation of parsed domain payloads
