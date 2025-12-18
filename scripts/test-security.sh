#!/bin/bash

# Security Testing Script for Rust Template
# 
# This script tests all security features implemented in the Rust template
# including CORS, security headers, JWT validation, and secret management

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
TEST_ORIGIN="${TEST_ORIGIN:-http://localhost:3000}"
JWT_SECRET="${JWT_SECRET:-test-secret-key-for-testing}"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test if server is running
check_server() {
    log_info "Checking if server is running at $BASE_URL..."
    if curl -s --max-time 5 "$BASE_URL/health" > /dev/null; then
        log_success "Server is running"
    else
        log_error "Server is not running at $BASE_URL"
        log_info "Please start the server first: cargo run"
        exit 1
    fi
}

# Test CORS functionality
test_cors() {
    log_info "Testing CORS functionality..."
    
    # Test preflight request
    log_info "Testing CORS preflight request..."
    cors_response=$(curl -s -w "%{http_code}" -X OPTIONS \
        -H "Origin: $TEST_ORIGIN" \
        -H "Access-Control-Request-Method: POST" \
        -H "Access-Control-Request-Headers: authorization,content-type" \
        "$BASE_URL/api/echo" -o /dev/null)
    
    if [ "$cors_response" = "200" ]; then
        log_success "CORS preflight request allowed"
    else
        log_error "CORS preflight request failed with status: $cors_response"
    fi
    
    # Test actual request with origin
    log_info "Testing actual request with origin..."
    cors_headers=$(curl -s -I \
        -H "Origin: $TEST_ORIGIN" \
        -H "Content-Type: application/json" \
        "$BASE_URL/health")
    
    if echo "$cors_headers" | grep -q "access-control-allow-origin"; then
        log_success "CORS headers present in response"
    else
        log_error "CORS headers missing in response"
    fi
    
    # Test unauthorized origin
    log_info "Testing unauthorized origin..."
    unauthorized_response=$(curl -s -w "%{http_code}" -X OPTIONS \
        -H "Origin: https://malicious-site.com" \
        -H "Access-Control-Request-Method: POST" \
        "$BASE_URL/api/echo" -o /dev/null)
    
    if [ "$unauthorized_response" = "403" ]; then
        log_success "Unauthorized origin correctly rejected"
    else
        log_warning "Unauthorized origin not properly rejected (status: $unauthorized_response)"
    fi
}

# Test security headers
test_security_headers() {
    log_info "Testing security headers..."
    
    # Get response headers
    headers=$(curl -s -I "$BASE_URL/health")
    
    # Check required security headers
    local headers_to_check=(
        "x-frame-options"
        "x-content-type-options"
        "x-xss-protection"
        "content-security-policy"
        "referrer-policy"
    )
    
    for header in "${headers_to_check[@]}"; do
        if echo "$headers" | grep -q "$header"; then
            log_success "$header header present"
        else
            log_error "$header header missing"
        fi
    done
    
    # Check specific header values
    if echo "$headers" | grep -q "X-Frame-Options: DENY"; then
        log_success "X-Frame-Options set to DENY"
    else
        log_warning "X-Frame-Options not set to DENY"
    fi
    
    if echo "$headers" | grep -q "X-Content-Type-Options: nosniff"; then
        log_success "X-Content-Type-Options set to nosniff"
    else
        log_warning "X-Content-Type-Options not set to nosniff"
    fi
    
    if echo "$headers" | grep -q "X-XSS-Protection: 1; mode=block"; then
        log_success "X-XSS-Protection set to 1; mode=block"
    else
        log_warning "X-XSS-Protection not set to 1; mode=block"
    fi
    
    # Check CSP policy
    if echo "$headers" | grep -q "Content-Security-Policy"; then
        csp=$(echo "$headers" | grep "Content-Security-Policy" | cut -d' ' -f2-)
        if echo "$csp" | grep -q "default-src 'self'"; then
            log_success "CSP includes default-src 'self'"
        else
            log_warning "CSP missing default-src 'self'"
        fi
        
        if echo "$csp" | grep -q "frame-ancestors 'none'"; then
            log_success "CSP includes frame-ancestors 'none'"
        else
            log_warning "CSP missing frame-ancestors 'none'"
        fi
    fi
    
    # Check HSTS in production
    if [[ "$BASE_URL" == https://* ]]; then
        if echo "$headers" | grep -q "Strict-Transport-Security"; then
            log_success "HSTS header present (HTTPS detected)"
        else
            log_warning "HSTS header missing (should be present for HTTPS)"
        fi
    else
        log_info "Skipping HSTS check (HTTP detected - HSTS not applicable)"
    fi
}

# Test JWT validation
test_jwt_validation() {
    log_info "Testing JWT validation..."
    
    # Generate a test JWT
    local payload="{\"sub\":\"test-user\",\"exp\":$(($(date +%s)+3600)),\"iat\":$(date +%s),\"iss\":\"rust-template\"}"
    local token=$(echo -n "$payload" | base64 -w 0 | tr '/+' '_-' | tr -d '=')
    
    # Create JWT manually (simplified for testing)
    local header="{\"alg\":\"HS256\",\"typ\":\"JWT\"}"
    local header_b64=$(echo -n "$header" | base64 -w 0 | tr '/+' '_-' | tr -d '=')
    local signature=$(echo -n "${header_b64}.${token}" | openssl dgst -sha256 -hmac "$JWT_SECRET" -binary | base64 -w 0 | tr '/+' '_-' | tr -d '=')
    local jwt="${header_b64}.${token}.${signature}"
    
    # Test with valid JWT
    log_info "Testing with valid JWT..."
    jwt_response=$(curl -s -w "%{http_code}" -X POST \
        -H "Authorization: Bearer $jwt" \
        -H "Content-Type: application/json" \
        -d '{"message": "test"}' \
        "$BASE_URL/api/echo" -o /dev/null)
    
    if [ "$jwt_response" = "200" ]; then
        log_success "Valid JWT accepted"
    else
        log_warning "Valid JWT rejected (status: $jwt_response) - may be expected if auth mode is not JWT"
    fi
    
    # Test with invalid JWT
    log_info "Testing with invalid JWT..."
    invalid_response=$(curl -s -w "%{http_code}" -X POST \
        -H "Authorization: Bearer invalid.jwt.token" \
        -H "Content-Type: application/json" \
        -d '{"message": "test"}' \
        "$BASE_URL/api/echo" -o /dev/null)
    
    if [ "$invalid_response" = "401" ]; then
        log_success "Invalid JWT rejected"
    else
        log_warning "Invalid JWT not properly rejected (status: $invalid_response)"
    fi
    
    # Test JWT with clock skew (future token within leeway)
    log_info "Testing JWT with future timestamp (within leeway)..."
    future_time=$(($(date +%s)+30)) # 30 seconds in future
    future_payload="{\"sub\":\"test-user\",\"exp\":${future_time},\"iat\":$(date +%s),\"iss\":\"rust-template\"}"
    future_token=$(echo -n "$future_payload" | base64 -w 0 | tr '/+' '_-' | tr -d '=')
    future_signature=$(echo -n "${header_b64}.${future_token}" | openssl dgst -sha256 -hmac "$JWT_SECRET" -binary | base64 -w 0 | tr '/+' '_-' | tr -d '=')
    future_jwt="${header_b64}.${future_token}.${future_signature}"
    
    future_response=$(curl -s -w "%{http_code}" -X POST \
        -H "Authorization: Bearer $future_jwt" \
        -H "Content-Type: application/json" \
        -d '{"message": "test"}' \
        "$BASE_URL/api/echo" -o /dev/null)
    
    if [ "$future_response" = "200" ]; then
        log_success "Future JWT accepted (within leeway)"
    else
        log_warning "Future JWT rejected (status: $future_response) - leeway may not be configured"
    fi
}

# Test secret management
test_secret_management() {
    log_info "Testing secret management..."
    
    # Check if config file exists
    if [ -f "config/local.yaml" ]; then
        log_success "config/local.yaml exists"
        
        # Check for placeholder values
        if grep -q "CHANGE_ME" config/local.yaml; then
            log_warning "config/local.yaml contains placeholder values - should be replaced in production"
        else
            log_success "config/local.yaml does not contain placeholder values"
        fi
        
        # Check for obvious secrets
        if grep -q -E "(dev-secret|dev-password|dev-token)" config/local.yaml; then
            log_warning "config/local.yaml contains development secrets - should be replaced in production"
        else
            log_success "config/local.yaml does not contain obvious development secrets"
        fi
    else
        log_warning "config/local.yaml not found"
    fi
    
    # Check for template file
    if [ -f "config/local.yaml.template" ]; then
        log_success "config/local.yaml.template exists"
    else
        log_warning "config/local.yaml.template not found"
    fi
    
    # Check environment variables for secrets
    if [ -n "${PLATFORM_JWT_SECRET:-}" ]; then
        log_success "PLATFORM_JWT_SECRET environment variable is set"
    else
        log_info "PLATFORM_JWT_SECRET environment variable not set (may be using config file)"
    fi
    
    if [ -n "${PLATFORM_AUTH_TOKEN:-}" ]; then
        log_success "PLATFORM_AUTH_TOKEN environment variable is set"
    else
        log_info "PLATFORM_AUTH_TOKEN environment variable not set (may be using config file)"
    fi
}

# Test request ID propagation
test_request_id() {
    log_info "Testing request ID propagation..."
    
    # Test with provided request ID
    request_id_response=$(curl -s -I \
        -H "X-Request-ID: test-request-id-123" \
        "$BASE_URL/health")
    
    if echo "$request_id_response" | grep -q "x-request-id: test-request-id-123"; then
        log_success "Request ID properly propagated"
    else
        log_warning "Request ID not properly propagated"
    fi
    
    # Test without provided request ID (should generate one)
    generated_id_response=$(curl -s -I "$BASE_URL/health")
    
    if echo "$generated_id_response" | grep -q "x-request-id:"; then
        log_success "Request ID generated when not provided"
    else
        log_warning "Request ID not generated when not provided"
    fi
}

# Run unit tests
run_unit_tests() {
    log_info "Running security-related unit tests..."
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found - cannot run unit tests"
        return
    fi
    
    # Run security middleware tests
    log_info "Running security middleware tests..."
    if cargo test security_middleware --quiet; then
        log_success "Security middleware tests passed"
    else
        log_error "Security middleware tests failed"
    fi
    
    # Run JWT validation tests
    log_info "Running JWT validation tests..."
    if cargo test jwt_validation --quiet; then
        log_success "JWT validation tests passed"
    else
        log_error "JWT validation tests failed"
    fi
    
    # Run CORS tests
    log_info "Running CORS tests..."
    if cargo test cors --quiet; then
        log_success "CORS tests passed"
    else
        log_error "CORS tests failed"
    fi
}

# Main execution
main() {
    echo "========================================"
    echo "Rust Template Security Testing"
    echo "========================================"
    echo "Base URL: $BASE_URL"
    echo "Test Origin: $TEST_ORIGIN"
    echo "========================================"
    
    # Check dependencies
    if ! command -v curl &> /dev/null; then
        log_error "curl is required but not installed"
        exit 1
    fi
    
    if ! command -v openssl &> /dev/null; then
        log_error "openssl is required but not installed"
        exit 1
    fi
    
    # Run tests
    check_server
    test_cors
    test_security_headers
    test_jwt_validation
    test_secret_management
    test_request_id
    
    # Ask if user wants to run unit tests
    echo ""
    read -p "Run unit tests? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        run_unit_tests
    fi
    
    echo ""
    echo "========================================"
    echo "Security testing completed!"
    echo "========================================"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --url)
            BASE_URL="$2"
            shift 2
            ;;
        --origin)
            TEST_ORIGIN="$2"
            shift 2
            ;;
        --jwt-secret)
            JWT_SECRET="$2"
            shift 2
            ;;
        --unit-tests-only)
            run_unit_tests
            exit 0
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo "  --url URL           Base URL to test (default: http://localhost:8080)"
            echo "  --origin ORIGIN      Origin for CORS tests (default: http://localhost:3000)"
            echo "  --jwt-secret SECRET  JWT secret for testing (default: test-secret-key-for-testing)"
            echo "  --unit-tests-only    Run only unit tests"
            echo "  --help, -h          Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run main function
main "$@"