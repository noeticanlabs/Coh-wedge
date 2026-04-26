#!/bin/bash
set -euo pipefail

echo "[verify_adversarial] Auditing rejection vectors..."

# Ensure we are in the root
cd "$(dirname "$0")/.."

VALIDATOR_BIN="./coh-node/target/release/coh-validator"

echo "[verify_adversarial] Working directory: $(pwd)"
echo "[verify_adversarial] Validator binary: $VALIDATOR_BIN"

if [ ! -f "$VALIDATOR_BIN" ]; then
    echo "Error: coh-validator not found at $VALIDATOR_BIN. Build it first."
    exit 1
fi

FAILED=0

for f in coh-node/vectors/adversarial/reject_*.jsonl; do
    echo "Checking $f..."
    if [ ! -f "$f" ]; then
        echo "FAIL: missing adversarial vector $f"
        FAILED=$((FAILED + 1))
        continue
    fi

    # Run validator and capture output without letting expected REJECT exits trip `set -e`
    if OUTPUT=$("$VALIDATOR_BIN" verify-chain "$f" 2>&1); then
        EXIT_CODE=0
    else
        EXIT_CODE=$?
    fi

    echo "[verify_adversarial] Validator exit code for $f: $EXIT_CODE"
    
    if [ $EXIT_CODE -eq 0 ]; then
        echo "FAIL: $f was ACCEPTED but should have been REJECTED"
        echo "$OUTPUT"
        FAILED=$((FAILED + 1))
    elif [ $EXIT_CODE -gt 3 ]; then
        echo "FAIL: $f caused a CRASH or internal error (code $EXIT_CODE)"
        echo "$OUTPUT"
        FAILED=$((FAILED + 1))
    else
        echo "PASS: $f rejected with code $EXIT_CODE"
    fi
done

if [ $FAILED -gt 0 ]; then
    echo "[verify_adversarial] AUDIT FAILED with $FAILED errors"
    exit 1
else
    echo "[verify_adversarial] AUDIT PASSED"
fi
