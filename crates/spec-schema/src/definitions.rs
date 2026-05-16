use crate::types::SchemaInfo;

// ============================================================================
// Schema Definitions
// ============================================================================

/// Spec Ledger JSON Schema.
pub(crate) fn get_spec_ledger_schema() -> SchemaInfo {
    SchemaInfo {
        name: "spec_ledger".to_string(),
        version: "1.0".to_string(),
        description: "Story → Requirement → Acceptance Criterion traceability ledger".to_string(),
        source_file: "specs/spec_ledger.yaml".to_string(),
        json_schema: serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["metadata", "stories"],
            "properties": {
                "metadata": {
                    "type": "object",
                    "required": ["schema_version", "template_version", "last_updated", "description"],
                    "properties": {
                        "schema_version": {
                            "type": "string",
                            "description": "Schema version for ledger format"
                        },
                        "template_version": {
                            "type": "string",
                            "description": "Template version this ledger conforms to"
                        },
                        "last_updated": {
                            "type": "string",
                            "format": "date",
                            "description": "Date of last update (YYYY-MM-DD)"
                        },
                        "description": {
                            "type": "string",
                            "description": "Human-readable ledger description"
                        },
                        "adrs": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Template-wide ADR references"
                        }
                    }
                },
                "stories": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "requirements"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^US-[A-Z0-9-]+-\\d+$",
                                "description": "Unique story identifier"
                            },
                            "title": {
                                "type": "string",
                                "description": "Short title for story"
                            },
                            "description": {
                                "type": "string",
                                "description": "Detailed story description"
                            },
                            "adr": {
                                "oneOf": [
                                    { "type": "string" },
                                    { "type": "array", "items": { "type": "string" } }
                                ],
                                "description": "Associated ADRs"
                            },
                            "requirements": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "required": ["id", "title", "acceptance_criteria"],
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "pattern": "^REQ-[A-Z0-9-]+$",
                                            "description": "Unique requirement identifier"
                                        },
                                        "title": {
                                            "type": "string",
                                            "description": "Requirement title"
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "Requirement description"
                                        },
                                        "tags": {
                                            "type": "array",
                                            "items": { "type": "string" },
                                            "description": "Requirement tags"
                                        },
                                        "must_have_ac": {
                                            "type": "boolean",
                                            "description": "Whether this requirement must have at least one AC"
                                        },
                                        "adr": {
                                            "oneOf": [
                                                { "type": "string" },
                                                { "type": "array", "items": { "type": "string" } }
                                            ],
                                            "description": "Associated ADRs"
                                        },
                                        "docs": {
                                            "type": "array",
                                            "items": { "type": "string" },
                                            "description": "Supporting documentation references"
                                        },
                                        "ci_workflows": {
                                            "type": "array",
                                            "items": { "type": "string" },
                                            "description": "Related CI workflows"
                                        },
                                        "acceptance_criteria": {
                                            "type": "array",
                                            "items": {
                                                "type": "object",
                                                "required": ["id", "text"],
                                                "properties": {
                                                    "id": {
                                                        "type": "string",
                                                        "pattern": "^AC-[A-Z0-9-]+$",
                                                        "description": "Unique AC identifier"
                                                    },
                                                    "text": {
                                                        "type": "string",
                                                        "description": "Acceptance criterion text"
                                                    },
                                                    "tags": {
                                                        "type": "array",
                                                        "items": { "type": "string" },
                                                        "description": "AC tags"
                                                    },
                                                    "must_have_ac": {
                                                        "type": "boolean",
                                                        "description": "Whether this AC is required"
                                                    },
                                                    "note": {
                                                        "type": "string",
                                                        "description": "Optional AC notes"
                                                    },
                                                    "docs": {
                                                        "type": "array",
                                                        "items": { "type": "string" },
                                                        "description": "Supporting documentation references"
                                                    },
                                                    "adr": {
                                                        "oneOf": [
                                                            { "type": "string" },
                                                            { "type": "array", "items": { "type": "string" } }
                                                        ],
                                                        "description": "Associated ADRs"
                                                    },
                                                    "tests": {
                                                        "type": "array",
                                                        "items": {
                                                            "type": "object",
                                                            "required": ["type", "tag"],
                                                            "properties": {
                                                                "type": {
                                                                    "type": "string",
                                                                    "description": "Test type (bdd, unit, integration, ci, manual, docs)"
                                                                },
                                                                "tag": {
                                                                    "type": "string",
                                                                    "description": "Test tag or scenario identifier"
                                                                },
                                                                "file": {
                                                                    "type": "string",
                                                                    "description": "File path where test is defined"
                                                                },
                                                                "module": {
                                                                    "type": "string",
                                                                    "description": "Module path for test"
                                                                },
                                                                "workflow": {
                                                                    "type": "string",
                                                                    "description": "CI workflow name"
                                                                }
                                                            }
                                                        },
                                                        "description": "Test mappings for this AC"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }),
    }
}

/// Tasks JSON Schema.
pub(crate) fn get_tasks_schema() -> SchemaInfo {
    SchemaInfo {
        name: "tasks".to_string(),
        version: "1.0".to_string(),
        description: "Work item tracking and task management".to_string(),
        source_file: "specs/tasks.yaml".to_string(),
        json_schema: serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["schema_version", "tasks"],
            "properties": {
                "schema_version": {
                    "type": "string",
                    "description": "Schema version for tasks format"
                },
                "template_version": {
                    "type": "string",
                    "description": "Template version"
                },
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "requirement", "acs", "status"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique task identifier"
                            },
                            "title": {
                                "type": "string",
                                "description": "Task title"
                            },
                            "requirement": {
                                "type": "string",
                                "pattern": "^REQ-[A-Z0-9]+-[A-Z0-9-]+$",
                                "description": "Related requirement ID"
                            },
                            "acs": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                    "pattern": "^AC-[A-Z0-9]+-[A-Z0-9-]+$"
                                },
                                "description": "List of AC IDs this task implements"
                            },
                            "status": {
                                "type": "string",
                                "enum": ["open", "in_progress", "review", "done"],
                                "description": "Task status"
                            },
                            "owner": {
                                "type": "string",
                                "description": "Task owner"
                            },
                            "labels": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Task labels/tags"
                            },
                            "summary": {
                                "type": "string",
                                "description": "Task summary"
                            },
                            "recommended_flows": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Recommended workflow names"
                            },
                            "docs": {
                                "type": "object",
                                "properties": {
                                    "design": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                        "description": "Design doc IDs"
                                    },
                                    "plan": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                        "description": "Plan doc IDs"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }),
    }
}

/// Questions JSON Schema.
pub(crate) fn get_questions_schema() -> SchemaInfo {
    SchemaInfo {
        name: "questions".to_string(),
        version: "1.0".to_string(),
        description: "Structured ambiguity artifacts for capturing unresolved decisions"
            .to_string(),
        source_file: "specs/questions_schema.yaml".to_string(),
        json_schema: serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["id", "created_by", "created_at", "summary", "context"],
            "properties": {
                "id": {
                    "type": "string",
                    "pattern": "^Q-[A-Z0-9]+-\\d{3}$",
                    "description": "Unique question identifier (e.g., Q-TPL-001)"
                },
                "task_id": {
                    "type": "string",
                    "description": "Optional task ID this question is blocking or related to"
                },
                "req_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Related requirement IDs from spec_ledger.yaml"
                },
                "ac_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Related AC IDs from spec_ledger.yaml"
                },
                "summary": {
                    "type": "string",
                    "maxLength": 200,
                    "description": "Brief summary of question or ambiguity"
                },
                "context": {
                    "type": "object",
                    "required": ["flow", "phase", "description"],
                    "properties": {
                        "flow": {
                            "type": "string",
                            "description": "Flow that generated this question"
                        },
                        "phase": {
                            "type": "string",
                            "description": "Phase within flow where ambiguity occurred"
                        },
                        "description": {
                            "type": "string",
                            "description": "Detailed description of ambiguity"
                        },
                        "files_involved": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Files being processed when ambiguity was detected"
                        }
                    }
                },
                "options": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["label", "description"],
                        "properties": {
                            "label": { "type": "string" },
                            "description": { "type": "string" },
                            "risk": {
                                "type": "string",
                                "enum": ["low", "medium", "high"]
                            },
                            "reversible": { "type": "boolean" }
                        }
                    }
                },
                "recommendation": {
                    "type": "object",
                    "properties": {
                        "option_label": { "type": "string" },
                        "rationale": { "type": "string" },
                        "confidence": {
                            "type": "string",
                            "enum": ["low", "medium", "high"]
                        }
                    }
                },
                "created_by": {
                    "type": "string",
                    "enum": ["agent", "human", "flow"]
                },
                "created_at": {
                    "type": "string",
                    "format": "date-time"
                },
                "status": {
                    "type": "string",
                    "enum": ["open", "answered", "resolved", "obsolete"],
                    "default": "open"
                },
                "resolution": {
                    "type": "object",
                    "properties": {
                        "resolved_by": { "type": "string" },
                        "resolved_at": {
                            "type": "string",
                            "format": "date-time"
                        },
                        "chosen_option": { "type": "string" },
                        "notes": { "type": "string" }
                    }
                }
            }
        }),
    }
}

/// DevEx Flows JSON Schema.
pub(crate) fn get_devex_flows_schema() -> SchemaInfo {
    SchemaInfo {
        name: "devex_flows".to_string(),
        version: "1.0".to_string(),
        description: "Developer experience workflows and xtask commands".to_string(),
        source_file: "specs/devex_flows.yaml".to_string(),
        json_schema: serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["schema_version", "commands"],
            "properties": {
                "schema_version": {
                    "type": "string",
                    "description": "Schema version"
                },
                "template_version": {
                    "type": "string",
                    "description": "Template version"
                },
                "commands": {
                    "type": "object",
                    "additionalProperties": {
                        "type": "object",
                        "required": ["category", "summary", "required"],
                        "properties": {
                            "category": {
                                "type": "string",
                                "enum": ["onboarding", "design_ac", "testing", "governance", "release"],
                                "description": "Command category"
                            },
                            "summary": {
                                "type": "string",
                                "description": "Brief command description"
                            },
                            "required": {
                                "type": "boolean",
                                "description": "Whether this command is required"
                            },
                            "docs": {
                                "type": "object",
                                "properties": {
                                    "readme_table": { "type": "boolean" },
                                    "contributing_flow": { "type": "boolean" },
                                    "claude_golden_path": { "type": "boolean" }
                                }
                            }
                        }
                    }
                },
                "flows": {
                    "type": "object",
                    "additionalProperties": {
                        "type": "object",
                        "required": ["description", "steps"],
                        "properties": {
                            "description": { "type": "string" },
                            "steps": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        }
                    }
                }
            }
        }),
    }
}

/// Config JSON Schema.
pub(crate) fn get_config_schema() -> SchemaInfo {
    SchemaInfo {
        name: "config".to_string(),
        version: "1.0".to_string(),
        description: "Service configuration schema (settings and secrets)".to_string(),
        source_file: "specs/config_schema.yaml".to_string(),
        json_schema: serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["schema_version", "settings", "secrets"],
            "properties": {
                "schema_version": {
                    "type": "number",
                    "description": "Schema version"
                },
                "envs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": { "type": "string" },
                            "required": { "type": "boolean" }
                        }
                    }
                },
                "settings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["key", "type"],
                        "properties": {
                            "key": { "type": "string" },
                            "type": {
                                "type": "string",
                                "enum": ["string", "int", "bool", "float"]
                            },
                            "default": {},
                            "description": { "type": "string" },
                            "required": { "type": "boolean" }
                        }
                    }
                },
                "secrets": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["key", "type"],
                        "properties": {
                            "key": { "type": "string" },
                            "type": {
                                "type": "string",
                                "enum": ["string", "int", "bool", "float"]
                            },
                            "description": { "type": "string" },
                            "required": { "type": "boolean" }
                        }
                    }
                }
            }
        }),
    }
}

/// Doc Index JSON Schema.
pub(crate) fn get_doc_index_schema() -> SchemaInfo {
    SchemaInfo {
        name: "doc_index".to_string(),
        version: "1.0".to_string(),
        description: "Documentation inventory mapping docs to stories/requirements/ACs/ADRs"
            .to_string(),
        source_file: "specs/doc_index.yaml".to_string(),
        json_schema: serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["schema_version", "docs"],
            "properties": {
                "schema_version": {
                    "type": "string",
                    "description": "Schema version"
                },
                "template_version": {
                    "type": "string",
                    "description": "Template version"
                },
                "docs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "file", "doc_type"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique doc identifier (e.g., DESIGN-TPL-001)"
                            },
                            "file": {
                                "type": "string",
                                "description": "Relative path to doc file"
                            },
                            "doc_type": {
                                "type": "string",
                                "enum": ["design_doc", "impl_plan", "requirements_doc", "runbook"],
                                "description": "Type of documentation"
                            },
                            "stories": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Related story IDs"
                            },
                            "requirements": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Related requirement IDs"
                            },
                            "acs": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Related AC IDs"
                            },
                            "adrs": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Related ADR IDs"
                            }
                        }
                    }
                }
            }
        }),
    }
}

/// Service Metadata JSON Schema.
pub(crate) fn get_service_metadata_schema() -> SchemaInfo {
    SchemaInfo {
        name: "service_metadata".to_string(),
        version: "1.0".to_string(),
        description: "Service identity and metadata".to_string(),
        source_file: "specs/service_metadata.yaml".to_string(),
        json_schema: serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["service_id"],
            "properties": {
                "service_id": {
                    "type": "string",
                    "description": "Unique service identifier"
                },
                "template_version": {
                    "type": "string",
                    "description": "Template version"
                },
                "display_name": {
                    "type": "string",
                    "description": "Human-readable service name"
                },
                "description": {
                    "type": "string",
                    "description": "Service description"
                },
                "links": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "Related links (docs, repos, etc.)"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Service tags/labels"
                }
            }
        }),
    }
}
