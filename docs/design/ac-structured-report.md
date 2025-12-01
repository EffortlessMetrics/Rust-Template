---
id: DESIGN-TPL-AC-REPORT-001
doc_type: design_doc
title: "AC Structured Report Design"
author: platform-team
date: 2025-11-30
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DEVEX-CONTRACT]
tags: [platform, devex, testing]
acs: []
adrs: [ADR-0005]
---
<!-- doclint:disable orphan-version -->

# AC Structured Report Design

## Overview

This document proposes a structured JSON report format for acceptance test results that `xtask ac-status` can consume to replace the current JUnit XML parsing approach.

## Current State Analysis

### Acceptance Test Setup

**Location:** `crates/acceptance/tests/acceptance.rs`

**Current Configuration:**
- Uses `cucumber` crate v0.21.1 with `output-junit` feature
- Generates JUnit XML to `target/junit/acceptance.xml`
- Uses `writer::Basic::stdout()` combined with `writer::JUnit`
- Employs `.tee()` combinator to output to both console and JUnit file

**Key Code:**
```rust
World::cucumber()
    .with_writer(
        writer::Basic::stdout()
            .summarized()
            .tee::<World, _>(writer::JUnit::new(junit_file, 0)),
    )
    .before(|_feature, _rule, _scenario, world| {
        Box::pin(async move {
            *world = World::new();
        })
    })
    .run(features_path.to_str().unwrap())
    .await;
```

### Current AC Status Command

**Location:** `crates/xtask/src/commands/ac_status.rs`

**Current Approach:**
1. Parses YAML ledger (`specs/spec_ledger.yaml`) for AC definitions
2. Parses `.feature` files manually with regex to extract scenarios and AC tags
3. Parses JUnit XML to extract test results
4. Correlates scenarios to ACs via tags
5. Generates markdown report

**Limitations:**
- JUnit XML doesn't include Gherkin metadata (tags, line numbers)
- Requires dual parsing of feature files and JUnit output
- Cannot extract duration per scenario easily from JUnit
- Feature file parsing is fragile (regex-based)
- No structured data format for programmatic consumption

## Proposed JSON Schema

### Schema Design

Based on Cucumber's JSON format and the needs of `xtask ac-status`, here's a proposed schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Acceptance Test Report",
  "type": "object",
  "properties": {
    "generated_at": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp when report was generated"
    },
    "features": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Feature"
      }
    },
    "summary": {
      "$ref": "#/definitions/Summary"
    }
  },
  "required": ["generated_at", "features", "summary"],
  "definitions": {
    "Feature": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Feature name"
        },
        "uri": {
          "type": "string",
          "description": "Relative path to feature file"
        },
        "tags": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Feature-level tags"
        },
        "scenarios": {
          "type": "array",
          "items": {
            "$ref": "#/definitions/Scenario"
          }
        }
      },
      "required": ["name", "uri", "scenarios"]
    },
    "Scenario": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Scenario name"
        },
        "line": {
          "type": "integer",
          "description": "Line number in feature file"
        },
        "tags": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Scenario tags (includes AC IDs like @AC-TPL-001)"
        },
        "status": {
          "type": "string",
          "enum": ["passed", "failed", "skipped", "pending"],
          "description": "Test execution status"
        },
        "duration_ns": {
          "type": "integer",
          "description": "Duration in nanoseconds (optional)",
          "minimum": 0
        },
        "error_message": {
          "type": "string",
          "description": "Error message if status is failed (optional)"
        },
        "steps": {
          "type": "array",
          "items": {
            "$ref": "#/definitions/Step"
          }
        }
      },
      "required": ["name", "line", "tags", "status", "steps"]
    },
    "Step": {
      "type": "object",
      "properties": {
        "keyword": {
          "type": "string",
          "description": "Step keyword (Given, When, Then, And, But)"
        },
        "name": {
          "type": "string",
          "description": "Step text"
        },
        "line": {
          "type": "integer",
          "description": "Line number in feature file"
        },
        "status": {
          "type": "string",
          "enum": ["passed", "failed", "skipped", "pending"],
          "description": "Step execution status"
        },
        "duration_ns": {
          "type": "integer",
          "description": "Duration in nanoseconds (optional)",
          "minimum": 0
        },
        "error_message": {
          "type": "string",
          "description": "Error message if step failed (optional)"
        }
      },
      "required": ["keyword", "name", "line", "status"]
    },
    "Summary": {
      "type": "object",
      "properties": {
        "total_scenarios": {
          "type": "integer",
          "minimum": 0
        },
        "passed_scenarios": {
          "type": "integer",
          "minimum": 0
        },
        "failed_scenarios": {
          "type": "integer",
          "minimum": 0
        },
        "skipped_scenarios": {
          "type": "integer",
          "minimum": 0
        },
        "total_steps": {
          "type": "integer",
          "minimum": 0
        },
        "passed_steps": {
          "type": "integer",
          "minimum": 0
        },
        "failed_steps": {
          "type": "integer",
          "minimum": 0
        },
        "skipped_steps": {
          "type": "integer",
          "minimum": 0
        }
      },
      "required": [
        "total_scenarios",
        "passed_scenarios",
        "failed_scenarios",
        "skipped_scenarios",
        "total_steps",
        "passed_steps",
        "failed_steps",
        "skipped_steps"
      ]
    }
  }
}
```

### Example Output

```json
{
  "generated_at": "2025-11-14T19:41:30.249176144Z",
  "features": [
    {
      "name": "Template Core Endpoints",
      "uri": "specs/features/template_core.feature",
      "tags": [],
      "scenarios": [
        {
          "name": "Health endpoint reports service is healthy",
          "line": 7,
          "tags": ["@AC-TPL-001", "@smoke"],
          "status": "passed",
          "duration_ns": 4100835,
          "steps": [
            {
              "keyword": "When",
              "name": "I GET /health",
              "line": 8,
              "status": "passed",
              "duration_ns": 2050000
            },
            {
              "keyword": "Then",
              "name": "I receive 200 with status \"ok\"",
              "line": 9,
              "status": "passed",
              "duration_ns": 2050835
            }
          ]
        },
        {
          "name": "Version endpoint reports build information",
          "line": 12,
          "tags": ["@AC-TPL-002"],
          "status": "passed",
          "duration_ns": 3874238,
          "steps": [
            {
              "keyword": "When",
              "name": "I GET /version",
              "line": 13,
              "status": "passed",
              "duration_ns": 1937119
            },
            {
              "keyword": "Then",
              "name": "I receive 200 with JSON containing \"version\" and \"gitSha\"",
              "line": 14,
              "status": "passed",
              "duration_ns": 1937119
            }
          ]
        }
      ]
    },
    {
      "name": "Refunds",
      "uri": "specs/features/refund.feature",
      "tags": [],
      "scenarios": [
        {
          "name": "Create a refund",
          "line": 3,
          "tags": ["@smoke", "@AC-123"],
          "status": "passed",
          "duration_ns": 4500983,
          "steps": [
            {
              "keyword": "Given",
              "name": "an order \"ORD-1\" totalling 5000 cents",
              "line": 4,
              "status": "passed",
              "duration_ns": 1500327
            },
            {
              "keyword": "When",
              "name": "I POST /refunds with { \"orderId\": \"ORD-1\", \"amountCents\": 5000 }",
              "line": 5,
              "status": "passed",
              "duration_ns": 2000328
            },
            {
              "keyword": "Then",
              "name": "I receive 201 with a \"refundId\"",
              "line": 6,
              "status": "passed",
              "duration_ns": 1000328
            }
          ]
        }
      ]
    }
  ],
  "summary": {
    "total_scenarios": 3,
    "passed_scenarios": 3,
    "failed_scenarios": 0,
    "skipped_scenarios": 0,
    "total_steps": 7,
    "passed_steps": 7,
    "failed_steps": 0,
    "skipped_steps": 0
  }
}
```

## Implementation Approaches

### Approach 1: Use Built-in Cucumber JSON Writer (RECOMMENDED)

**Description:** Use cucumber's built-in `writer::Json` which already outputs to Cucumber's JSON format.

**Implementation:**

1. Enable the `output-json` feature in `crates/acceptance/Cargo.toml`
2. Add JSON output alongside JUnit XML using `.tee()`
3. Optionally support environment variable for output path

**Code Changes:**

```rust
// In crates/acceptance/Cargo.toml
cucumber = { version = "0.21.1", features = ["output-junit", "output-json"] }

// In crates/acceptance/tests/acceptance.rs
let json_path = std::env::var("AC_REPORT_JSON")
    .unwrap_or_else(|_| workspace_root.join("target/acceptance-report.json").to_string_lossy().to_string());
let json_file = File::create(&json_path).expect("Failed to create JSON output file");

World::cucumber()
    .with_writer(
        writer::Basic::stdout()
            .summarized()
            .tee::<World, _>(
                writer::JUnit::new(junit_file, 0)
                    .tee::<World, _>(writer::Json::for_tee(json_file))
            ),
    )
    .before(|_feature, _rule, _scenario, world| {
        Box::pin(async move {
            *world = World::new();
        })
    })
    .run(features_path.to_str().unwrap())
    .await;
```

**Pros:**
- Uses official Cucumber JSON format
- No custom code to maintain
- Already normalized and tested
- Compatible with other Cucumber tooling
- Includes all Gherkin metadata automatically

**Cons:**
- Cucumber's JSON format may include more data than needed
- Format is in "maintenance mode" (though still supported)
- Need to add `output-json` feature dependency

### Approach 2: Custom Cucumber Writer

**Description:** Implement a custom `Writer<World>` trait that outputs our exact schema.

**Implementation:**

1. Create a new module `crates/acceptance/src/json_writer.rs`
2. Implement the `Writer` trait to handle Cucumber events
3. Build up state and serialize to JSON on completion

**Example Structure:**

```rust
use cucumber::{event, writer, Event, World as _};
use std::io::Write;

pub struct AcReportWriter<W: Write> {
    output: W,
    features: Vec<FeatureReport>,
    // ... state tracking
}

impl<World: cucumber::World, W: Write> writer::Writer<World> for AcReportWriter<W> {
    type Cli = cucumber::cli::Empty;

    async fn handle_event(
        &mut self,
        ev: Result<Event<event::Cucumber<World>>>,
        _cli: &Self::Cli,
    ) {
        match ev {
            Ok(Event { value: event::Cucumber::Feature(feature, ev), .. }) => {
                // Handle feature events
            }
            Ok(Event { value: event::Cucumber::Scenario(scenario, ev), .. }) => {
                // Handle scenario events
            }
            Ok(Event { value: event::Cucumber::Step(step, ev), .. }) => {
                // Handle step events
            }
            _ => {}
        }
    }
}
```

**Pros:**
- Complete control over output format
- Can include exactly what we need
- Smaller output files
- Direct integration with xtask needs

**Cons:**
- Must maintain custom code
- Need to handle all event types correctly
- More complex implementation
- Need to handle normalization manually

### Approach 3: Post-Processing Cucumber JSON

**Description:** Use Cucumber's built-in JSON writer, then transform it to our schema.

**Implementation:**

1. Output Cucumber's JSON format
2. Create a post-processing tool in xtask
3. Transform to our simplified schema

**Pros:**
- Leverages existing tested code
- Flexible - can change our format without touching acceptance tests
- Can add computed fields during transformation

**Cons:**
- Extra processing step
- Two formats to understand
- Potential for data loss during transformation

## Available Hooks and Events

### Cucumber Events System

The `cucumber` crate uses an event-driven architecture. The `Writer` trait receives events via `handle_event()`:

**Event Types:**
- `Cucumber::Feature` - Feature started/finished
- `Cucumber::Rule` - Rule started/finished
- `Cucumber::Scenario` - Scenario started/finished
- `Cucumber::Step` - Step started/passed/failed/skipped
- `Cucumber::Hook` - Before/After hooks

**Event Phases:**
- `Started` - Event is beginning
- `Passed` - Event succeeded
- `Failed` - Event failed with error
- `Skipped` - Event was skipped

**Key Event Data Available:**
- `Feature` - name, tags, location, description
- `Scenario` - name, tags, location (line number)
- `Step` - keyword, name, location (line number), duration
- Errors and panic information for failures

### Writer Combinators

Cucumber provides several writer combinators:

- `.tee()` - Duplicate events to multiple writers
- `.normalized()` - Ensure events arrive in order
- `.summarized()` - Add summary output
- `.fail_on_skipped()` - Treat skipped as failures
- `.repeat_failed()` - Re-output failures at end

The current setup uses `.tee()` to send events to both console and JUnit simultaneously.

## Recommended Implementation Approach

**Use Approach 1: Built-in Cucumber JSON Writer**

### Rationale

1. **Mature & Tested:** Cucumber's JSON format is well-established and tested
2. **Standards-Based:** Follows Cucumber's standard format used across implementations
3. **Low Maintenance:** No custom code to maintain
4. **Rich Data:** Includes all necessary metadata (tags, line numbers, durations)
5. **Easy Integration:** Simple to add via feature flag and `.tee()`

### Implementation Steps

#### Step 1: Update Dependencies

Add `output-json` feature to `crates/acceptance/Cargo.toml`:

```toml
cucumber = { version = "0.21.1", features = ["output-junit", "output-json"] }
```

#### Step 2: Modify Acceptance Test Runner

Update `crates/acceptance/tests/acceptance.rs`:

```rust
use cucumber::{writer, World as _, WriterExt};
use std::fs::File;

#[tokio::main]
async fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find workspace root");

    let features_path = workspace_root.join("specs/features");
    let junit_dir = workspace_root.join("target/junit");
    let junit_path = junit_dir.join("acceptance.xml");

    // Support environment variable for JSON output path
    let json_path = std::env::var("AC_REPORT_JSON")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target/acceptance-report.json"));

    // Ensure target/junit directory exists
    std::fs::create_dir_all(&junit_dir).expect("Failed to create junit directory");

    // Ensure JSON parent directory exists
    if let Some(parent) = json_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create JSON output directory");
    }

    let junit_file = File::create(&junit_path).expect("Failed to create JUnit output file");
    let json_file = File::create(&json_path).expect("Failed to create JSON output file");

    // Triple output: console + JUnit + JSON
    World::cucumber()
        .with_writer(
            writer::Basic::stdout()
                .summarized()
                .tee::<World, _>(writer::JUnit::new(junit_file, 0))
                .tee::<World, _>(writer::Json::for_tee(json_file)),
        )
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                *world = World::new();
            })
        })
        .run(features_path.to_str().unwrap())
        .await;
}
```

#### Step 3: Update AC Status Command

Modify `crates/xtask/src/commands/ac_status.rs`:

1. Add `json_report` field to `AcStatusArgs`
2. Add JSON parsing function using `serde_json`
3. Replace JUnit + feature file parsing with JSON parsing

**New parsing function:**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CucumberReport(Vec<CucumberFeature>);

#[derive(Debug, Deserialize)]
struct CucumberFeature {
    name: String,
    uri: String,
    elements: Vec<CucumberElement>,
}

#[derive(Debug, Deserialize)]
struct CucumberElement {
    name: String,
    #[serde(rename = "type")]
    element_type: String,
    tags: Vec<CucumberTag>,
    line: u32,
    steps: Vec<CucumberStep>,
}

#[derive(Debug, Deserialize)]
struct CucumberTag {
    name: String,
    line: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CucumberStep {
    keyword: String,
    name: String,
    line: u32,
    result: CucumberStepResult,
}

#[derive(Debug, Deserialize)]
struct CucumberStepResult {
    status: String,
    duration: Option<u64>, // nanoseconds
    error_message: Option<String>,
}

fn parse_cucumber_json(
    json_path: &Path,
) -> Result<HashMap<String, AcStatus>> {
    let content = fs::read_to_string(json_path)?;
    let report: CucumberReport = serde_json::from_str(&content)?;

    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();
    let ac_pattern = Regex::new(r"^@(AC-[A-Z0-9-]+)$")?;

    for feature in report.0 {
        for element in feature.elements {
            if element.element_type == "scenario" {
                // Extract AC IDs from tags
                let ac_ids: Vec<String> = element.tags
                    .iter()
                    .filter_map(|tag| {
                        ac_pattern.captures(&tag.name)
                            .map(|caps| caps[1].to_string())
                    })
                    .collect();

                // Determine if scenario passed
                let passed = element.steps
                    .iter()
                    .all(|step| step.result.status == "passed");

                // Record result for each AC ID
                for ac_id in ac_ids {
                    ac_results
                        .entry(ac_id)
                        .or_default()
                        .push(passed);
                }
            }
        }
    }

    // Aggregate: AC passes only if all scenarios pass
    let mut ac_status = HashMap::new();
    for (ac_id, results) in ac_results {
        let status = if results.iter().all(|&passed| passed) {
            AcStatus::Pass
        } else {
            AcStatus::Fail
        };
        ac_status.insert(ac_id, status);
    }

    Ok(ac_status)
}
```

4. Update `run()` function to use JSON instead of JUnit + feature files
5. Remove regex-based feature file parsing
6. Simplify the pipeline

#### Step 4: Update Documentation

1. Document the `AC_REPORT_JSON` environment variable
2. Update CI/CD to ensure JSON report is generated
3. Add examples of consuming the JSON report

#### Step 5: Testing

1. Run acceptance tests with JSON output
2. Verify JSON structure matches expectations
3. Test `xtask ac-status` with JSON input
4. Ensure backward compatibility (keep JUnit for now)

## Challenges and Dependencies

### Identified Challenges

1. **Feature Dependency:** Need to add `output-json` to cucumber dependencies
2. **File I/O:** Must ensure JSON file is created before writer is used
3. **Writer Chaining:** `.tee()` combinator requires careful ordering
4. **Normalization:** JSON writer requires normalized events (already handled)
5. **Error Handling:** Must handle JSON parse errors gracefully

### Dependencies

**Rust Crates:**
- `cucumber = "0.21.1"` with `output-json` feature (already available)
- `serde_json = "1.0"` (already in workspace)
- No new dependencies required!

**Environment:**
- Optional: `AC_REPORT_JSON` environment variable for custom output path
- Default: `target/acceptance-report.json`

**Build System:**
- No changes required
- Works with existing `cargo test -p acceptance`

## Migration Path

### Phase 1: Add JSON Output (No Breaking Changes)

1. Add JSON output alongside existing JUnit
2. Keep current `ac_status` implementation unchanged
3. Validate JSON format

### Phase 2: Update AC Status (Simplification)

1. Switch `ac_status` to read JSON instead of JUnit + feature files
2. Remove regex-based feature file parsing
3. Keep backward compatibility by supporting both formats

### Phase 3: Deprecate JUnit (Optional Future)

1. Once JSON is stable, consider deprecating JUnit XML
2. Remove JUnit output if no longer needed
3. Simplify acceptance test runner

## Alternative: Custom Schema with Transformation

If Cucumber's JSON format proves too verbose, we can add a transformation layer:

**Option:** Create `xtask transform-report` command that:
1. Reads Cucumber JSON
2. Transforms to simplified schema (our proposed format)
3. Outputs to custom location

This gives us the best of both worlds:
- Use proven Cucumber JSON generation
- Get simplified format for xtask consumption
- Can evolve our format independently

## Conclusion

**Recommended Path Forward:**

1. **Implement Approach 1** (Built-in Cucumber JSON Writer)
2. **Add environment variable support** for flexible output paths
3. **Update ac_status** to consume JSON instead of JUnit
4. **Keep JUnit output** for backward compatibility initially
5. **Consider custom transformation** only if Cucumber JSON proves inadequate

This approach provides:
- Minimal code to maintain
- Standards-based format
- All required metadata (tags, line numbers, durations)
- Simple integration with existing infrastructure
- Clear migration path

The implementation is low-risk and can be completed incrementally without breaking existing functionality.
