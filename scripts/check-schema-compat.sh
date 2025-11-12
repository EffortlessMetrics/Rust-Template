#!/usr/bin/env bash
set -euo pipefail
SUBJECT="${1:-}"
SCHEMA_FILE="${2:-}"
if [[ -z "$SUBJECT" || -z "$SCHEMA_FILE" ]]; then
  echo "usage: $0 <subject> <schema-file>" >&2
  exit 2
fi
curl -sS -u "$SCHEMA_REGISTRY_AUTH" -X POST   -H "Content-Type: application/vnd.schemaregistry.v1+json"   --data @"<(jq -n --arg schema "$(jq -c . < "$SCHEMA_FILE")" '{schema:$schema}')"   "$SCHEMA_REGISTRY_URL/compatibility/subjects/${SUBJECT}/versions"   | jq -e '.is_compatible == true'
