#!/usr/bin/env bash
set -euo pipefail

# Required environment variables
: "${SCHEMA_REGISTRY_URL:?Environment variable SCHEMA_REGISTRY_URL must be set}"
# SCHEMA_REGISTRY_AUTH is optional

echo "Checking schema compatibility against $SCHEMA_REGISTRY_URL"

# Auth setup
AUTH_OPTS=()
if [ -n "${SCHEMA_REGISTRY_AUTH:-}" ]; then
  AUTH_OPTS=("-u" "$SCHEMA_REGISTRY_AUTH")
fi

# Iterate over JSON schemas
for schema_file in specs/events/json-schema/*.json; do
  [ -e "$schema_file" ] || continue

  filename=$(basename "$schema_file")
  subject="${filename%.*}-value"

  echo "---------------------------------------------------"
  echo "Subject: $subject"
  echo "File:    $schema_file"

  # Read schema content
  schema_content=$(cat "$schema_file")

  # Construct payload
  # jq -n --arg schema "$schema_content" '{schemaType: "JSON", schema: $schema}'
  # Note: schemaType is required for JSON Schema in Confluent Schema Registry
  payload=$(jq -n --arg schema "$schema_content" '{schemaType: "JSON", schema: $schema}')

  # Check compatibility
  # We use a temporary file for curl output to handle HTTP codes and body
  response_body=$(mktemp)
  # Capture http code
  http_code=$(curl -s -w "%{http_code}" -o "$response_body" \
    "${AUTH_OPTS[@]}" \
    -H "Content-Type: application/vnd.schemaregistry.v1+json" \
    -X POST \
    -d "$payload" \
    "$SCHEMA_REGISTRY_URL/compatibility/subjects/$subject/versions/latest")

  echo "HTTP Code: $http_code"
  cat "$response_body"
  echo ""

  if [ "$http_code" -eq 200 ]; then
    is_compatible=$(jq -r '.is_compatible' "$response_body")
    if [ "$is_compatible" = "true" ]; then
      echo "✅ Compatible"
    else
      echo "❌ Incompatible"
      jq -r '.messages[]' "$response_body"
      rm "$response_body"
      exit 1
    fi
  elif [ "$http_code" -eq 404 ]; then
    # Check if error code is 40401 (Subject not found) or 40402 (Version not found)
    error_code=$(jq -r '.error_code' "$response_body")
    if [ "$error_code" = "40401" ] || [ "$error_code" = "40402" ]; then
       echo "ℹ️  Subject or version not found (likely first version). Treated as compatible."
    else
       echo "❌ Error 404: $(cat "$response_body")"
       rm "$response_body"
       exit 1
    fi
  else
    echo "❌ Error checking compatibility: HTTP $http_code"
    cat "$response_body"
    rm "$response_body"
    exit 1
  fi

  rm "$response_body"
done

echo "---------------------------------------------------"
echo "All schemas checked."
