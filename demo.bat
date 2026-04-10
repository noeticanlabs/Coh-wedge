@echo off
REM COHERENT VALIDATOR DEMO - 60 seconds
REM Shows: valid → ACCEPT, tampered → REJECT, chain → verified, slab → built + verified

echo === COHERENT VALIDATOR DEMO ===
echo.

echo [1] Verify valid micro-receipt (should ACCEPT)
coh-node\target\debug\coh-validator.exe verify-micro coh-node\examples\micro_valid.json
echo.

echo [2] Verify tampered micro-receipt (should REJECT - policy violation)
coh-node\target\debug\coh-validator.exe verify-micro coh-node\examples\micro_invalid_policy.json
echo.

echo [3] Verify valid chain (should ACCEPT)
coh-node\target\debug\coh-validator.exe verify-chain coh-node\examples\chain_valid.jsonl
echo.

echo [4] Build slab from chain
coh-node\target\debug\coh-validator.exe build-slab coh-node\examples\chain_valid.jsonl --out coh-node\examples\demo_slab.json
echo.

echo [5] Verify slab (should ACCEPT)
coh-node\target\debug\coh-validator.exe verify-slab coh-node\examples\demo_slab.json
echo.

echo [CLEANUP] Removing temporary slab
del coh-node\examples\demo_slab.json

echo.
echo === DEMO COMPLETE ===