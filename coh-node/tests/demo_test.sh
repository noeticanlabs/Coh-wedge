#!/bin/bash
# ================================================================================
# CI DEMO TEST - Runs deterministic demo sequences in CI
# ================================================================================
# This script validates core functionality by running the same sequences as demo.bat
# but in a CI-friendly way (bash, explicit exit codes, no interactive prompts)
#
# Demo sequence:
#   1. verify-micro valid.json        -> ACCEPT
#   2. verify-micro tampered.json     -> REJECT (digest mismatch)
#   3. verify-chain valid.jsonl       -> ACCEPT
#   4. verify-chain broken.jsonl      -> REJECT (continuity break)
#   5. build-slab valid.jsonl         -> SLAB_BUILT
#   6. verify-slab slab.json          -> ACCEPT
#   7. verify-slab broken_slab.json   -> REJECT (optional add-on)
# ================================================================================

set -e

BIN="${BIN:-./target/release/coh-validator}"
EXAMPLES="${EXAMPLES:-./examples}"

PASS=0
FAIL=0

echo "================================================================================"
echo "                    COHERENT VALIDATOR - CI DEMO TEST"
echo "================================================================================"
echo "Binary: $BIN"
echo "Examples: $EXAMPLES"
echo ""

# Helper function to run test
run_test() {
    local name="$1"
    local expected="$2"
    local cmd="$3"
    local input="$4"
    
    echo "------------------------------------------------------------------------"
    echo "Test: $name"
    echo "Input: $input"
    echo "Expected: $expected"
    
    if [ ! -f "$EXAMPLES/$input" ]; then
        echo "ERROR: Input file not found: $EXAMPLES/$input"
        exit 1
    fi
    
    local output
    output=$("$BIN" --format json "$cmd" "$EXAMPLES/$input" 2>&1) || true
    
    # Extract ACCEPT/REJECT from output
    local result
    if echo "$output" | grep -q '"decision":"ACCEPT"'; then
        result="ACCEPT"
    elif echo "$output" | grep -q '"decision":"REJECT"'; then
        result="REJECT"
    elif echo "$output" | grep -q "SLAB_BUILT"; then
        result="SLAB_BUILT"
    elif echo "$output" | grep -q "ACCEPT"; then
        result="ACCEPT"
    elif echo "$output" | grep -q "REJECT"; then
        result="REJECT"
    else
        result="ERROR: $output"
    fi
    
    if [ "$result" = "$expected" ]; then
        echo "Result: $result ✓ PASS"
        PASS=$((PASS + 1))
    else
        echo "Result: $result ✗ FAIL (expected $expected)"
        echo "Output: $output"
        FAIL=$((FAIL + 1))
    fi
    echo ""
}

# Build first if needed
if [ ! -f "$BIN" ]; then
    echo "Building binary..."
    cargo build --release -p coh-validator
fi

echo ""
echo "Starting demo sequence..."
echo ""

# Test 1: verify-micro valid -> ACCEPT
run_test "verify-micro valid" "ACCEPT" "verify-micro" "micro_valid.json"

# Test 2: verify-micro invalid digest -> REJECT
run_test "verify-micro invalid digest" "REJECT" "verify-micro" "micro_invalid_digest.json"

# Test 3: verify-chain valid -> ACCEPT
run_test "verify-chain valid" "ACCEPT" "verify-chain" "chain_valid.jsonl"

# Test 4: verify-chain broken -> REJECT
run_test "verify-chain broken" "REJECT" "verify-chain" "chain_invalid_index.jsonl"

# Test 5: verify-slab valid -> ACCEPT
run_test "verify-slab valid" "ACCEPT" "verify-slab" "slab_valid.json"

# Test 6: verify-slab invalid policy -> REJECT
run_test "verify-slab invalid policy" "REJECT" "verify-slab" "slab_invalid_policy.json"

echo "================================================================================"
echo "                              TEST SUMMARY"
echo "================================================================================"
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo ""

if [ $FAIL -gt 0 ]; then
    echo "CI DEMO TEST: FAILED"
    exit 1
else
    echo "CI DEMO TEST: PASSED"
    exit 0
fi