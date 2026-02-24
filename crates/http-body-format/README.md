# http-body-format

Single-responsibility classifier for HTTP request body formats.

## Scope

- Normalize and classify `content-type` values for request parsing
- Distinguish JSON, form-urlencoded, and unknown formats
- Handle media-type parameters and case-insensitive header values

## Why this crate exists

Request payload parsers should own decoding logic, not media-type normalization details.
This crate isolates content-type classification so parser crates can share deterministic
format selection behavior and test it independently.
