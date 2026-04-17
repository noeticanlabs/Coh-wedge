@echo off
REM COHERENT VALIDATOR - COMPREHENSIVE DEMO
REM This demo showcases all system capabilities with detailed output

echo ================================================================================
echo                   COHERENT VALIDATOR - COMPREHENSIVE DEMO
echo ================================================================================
echo.
echo This demo shows the complete verification pipeline:
echo   - Micro-receipt verification (single receipt)
echo   - Chain verification (linked receipts)
echo   - Slab building (aggregate receipts)
echo   - Slab verification (aggregate validation)
echo   - Tamper detection
echo   - Adversarial Vector Suite (Hardened Reject Paths)
echo   - Python Bridge Fuzzing (Interface Robustness)
echo.
pause

echo ================================================================================
echo PART 1: MICRO-RECEIPT VERIFICATION
echo ================================================================================
echo.
echo [1.1] Verify a VALID micro-receipt
coh-node\target\debug\coh-validator.exe verify-micro coh-node\examples\micro_valid.json
echo.
pause

echo ================================================================================
echo PART 2: CHAIN VERIFICATION
echo ================================================================================
echo.
echo [2.1] Verify a VALID chain (2 linked receipts)
coh-node\target\debug\coh-validator.exe verify-chain coh-node\examples\chain_valid.jsonl
echo.
pause

echo ================================================================================
echo PART 3: SLAB OPERATIONS
echo ================================================================================
echo.
echo [3.1] BUILD SLAB from valid chain
coh-node\target\debug\coh-validator.exe build-slab coh-node\examples\chain_valid.jsonl --out coh-node\examples\demo_slab.json
echo.
echo [3.2] VERIFY the built slab
coh-node\target\debug\coh-validator.exe verify-slab coh-node\examples\demo_slab.json
echo.
pause

echo ================================================================================
echo PART 4: ADVERSARIAL VECTOR SUITE (HARDENING)
echo ================================================================================
echo.
echo [4.1] REJECT: Digest Link Tampering
coh-node\target\debug\coh-validator.exe verify-chain coh-node\vectors\adversarial\reject_chain_digest.jsonl
echo.
echo [4.2] REJECT: State Discontinuity
coh-node\target\debug\coh-validator.exe verify-chain coh-node\vectors\adversarial\reject_state_link.jsonl
echo.
echo [4.3] REJECT: Policy Accounting Violation
coh-node\target\debug\coh-validator.exe verify-chain coh-node\vectors\adversarial\reject_policy_violation.jsonl
echo.
echo [4.4] REJECT: Numeric Parse Error
coh-node\target\debug\coh-validator.exe verify-chain coh-node\vectors\adversarial\reject_numeric_parse.jsonl
echo.
echo [4.5] REJECT: Arithmetic Overflow Check
coh-node\target\debug\coh-validator.exe verify-chain coh-node\vectors\adversarial\reject_overflow.jsonl
echo.
pause

echo ================================================================================
echo PART 5: BRIDGE FUZZING (INTERFACE RIGOR)
echo ================================================================================
echo.
echo Running Python Bridge Fuzz tests...
echo This ensures the PyO3 boundary never panics on malformed data.
echo.
python coh-node\tests\fuzz_bridge.py
echo.
pause

echo ================================================================================
echo                   DEMO COMPLETE
echo ================================================================================
echo.
echo You now have a production-ready, hardened validator.
echo.
pause
