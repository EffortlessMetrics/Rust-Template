#!/bin/bash
set -e

echo "K8s Secrets Policy Test (Manual Verification)"
echo "=============================================="
echo ""

# Test 1: Valid deployment with envFrom
echo "Test 1: Valid deployment with envFrom (should PASS)"
cat testdata/k8s_secrets_valid.json | \
  opa eval -d k8s.rego -I 'data.main.deny' 2>&1 | grep -q '[]' && \
  echo "  ✓ PASS: No violations found" || \
  echo "  ✗ FAIL: Unexpected violations"
echo ""

# Test 2: Invalid - literal DATABASE_URL
echo "Test 2: Deployment with literal DATABASE_URL (should FAIL)"
result=$(cat testdata/k8s_secrets_literal_database_url.json | \
  opa eval -d k8s.rego -I 'data.main.deny' 2>&1 || true)
if echo "$result" | grep -q "literal value for sensitive env var 'DATABASE_URL'"; then
  echo "  ✓ PASS: Correctly detected literal DATABASE_URL"
else
  echo "  ✗ FAIL: Did not detect literal DATABASE_URL"
  echo "  Output: $result"
fi
echo ""

# Test 3: Invalid - literal API_KEY
echo "Test 3: Deployment with literal API keys (should FAIL)"
result=$(cat testdata/k8s_secrets_literal_api_key.json | \
  opa eval -d k8s.rego -I 'data.main.deny' 2>&1 || true)
if echo "$result" | grep -q "API_KEY"; then
  echo "  ✓ PASS: Correctly detected literal API_KEY"
else
  echo "  ✗ FAIL: Did not detect literal API_KEY"
  echo "  Output: $result"
fi
echo ""

# Test 4: Invalid - configMapKeyRef for sensitive data
echo "Test 4: Deployment using ConfigMap for passwords (should FAIL)"
result=$(cat testdata/k8s_secrets_configmap_for_secret.json | \
  opa eval -d k8s.rego -I 'data.main.deny' 2>&1 || true)
if echo "$result" | grep -q "configMapKeyRef"; then
  echo "  ✓ PASS: Correctly detected configMapKeyRef for sensitive data"
else
  echo "  ✗ FAIL: Did not detect configMapKeyRef misuse"
  echo "  Output: $result"
fi
echo ""

echo "=============================================="
echo "Manual policy verification complete"
