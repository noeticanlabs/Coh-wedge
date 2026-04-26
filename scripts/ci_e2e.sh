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

echo -e "\n[ci_e2e] Sidecar is UP. Running APE integration..."

# Run APE demo
# Note: we might need to install python deps first in the CI job
python3 scripts/ape_coh_level2_integration.py --url http://127.0.0.1:3030

echo "[ci_e2e] SUCCESS: Integration test passed"
