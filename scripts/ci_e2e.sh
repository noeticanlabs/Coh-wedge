#!/bin/bash
set -e

echo "[ci_e2e] Starting Sidecar in background..."

# Ensure we are in the root
cd "$(dirname "$0")/.."

# Start sidecar

SIDECAR_BIN="./coh-node/target/release/coh-sidecar"
if [ ! -f "$SIDECAR_BIN" ]; then
    echo "Error: coh-sidecar not found at $SIDECAR_BIN. Build it first."
    exit 1
fi

$SIDECAR_BIN > sidecar.log 2>&1 &
SIDECAR_PID=$!

function cleanup {
  echo "[ci_e2e] Stopping Sidecar (PID $SIDECAR_PID)..."
  kill $SIDECAR_PID || true
}
trap cleanup EXIT

echo "[ci_e2e] Waiting for /health..."
MAX_RETRIES=60
COUNT=0
until $(curl --output /dev/null --silent --head --fail http://127.0.0.1:3030/health); do
    if [ $COUNT -eq $MAX_RETRIES ]; then
      echo "[ci_e2e] Error: Sidecar timed out"
      cat sidecar.log
      exit 1
    fi
    printf '.'
    sleep 1
    COUNT=$((COUNT+1))
done

echo -e "\n[ci_e2e] Sidecar is UP. Running NPE integration tests..."

# Run NPE unit tests
echo "[ci_e2e] Running NPE tests..."
cd coh-node && cargo test -p coh-npe -- --nocapture

# Also verify the core validator works
echo "[ci_e2e] Running core validator tests..."
cargo test -p coh-core --lib -- --nocapture

echo "[ci_e2e] SUCCESS: NPE integration tests passed"
