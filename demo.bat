@echo off
REM ================================================================================
REM COHERENT VALIDATOR - 60-90 SECOND DEMO SCRIPT
REM ================================================================================
REM
REM Run from workspace root. Binary must be built:
REM   cd coh-node && cargo build
REM
REM Demo sequence:
REM   1. verify-micro valid.json        -> ACCEPT
REM   2. verify-micro tampered.json     -> REJECT (digest mismatch)
REM   3. verify-chain valid.jsonl       -> ACCEPT
REM   4. verify-chain broken.jsonl      -> REJECT (continuity break)
REM   5. build-slab valid.jsonl          -> SLAB_BUILT
REM   6. verify-slab slab.json          -> ACCEPT
REM   7. verify-slab broken_slab.json   -> REJECT (optional add-on)
REM
REM ================================================================================

setlocal EnableDelayedExpansion

set BIN=coh-node\target\debug\coh-validator.exe
set EXAMPLES=coh-node\examples

echo ================================================================================
echo                 COHERENT VALIDATOR - PRODUCT DEMO
echo ================================================================================
echo.
echo This demo shows a deterministic validator for state transitions,
echo with tamper detection and explicit rejection reasons.
echo.
echo ================================================================================
echo.

REM === PART 1: Valid micro receipt ===
echo --- PART 1: Valid micro receipt ---
echo.
echo Verifying a single state transition receipt.
echo If the metrics, arithmetic, schema, and digest all line up, it returns ACCEPT.
echo.
%BIN% verify-micro %EXAMPLES%\micro_valid.json
echo.
echo This is deterministic. Same file, same answer, every time.
echo.
echo ================================================================================
echo.

REM === PART 2: Tampered micro receipt ===
echo --- PART 2: Tampered micro receipt ---
echo.
echo Using a nearly identical receipt, but the digest has been tampered with.
echo The validator should reject it immediately.
echo.
%BIN% verify-micro %EXAMPLES%\micro_invalid_digest.json
echo.
echo This is the integrity boundary.
echo If someone changes the receipt without recomputing a valid chain digest, it gets caught.
echo.
echo ================================================================================
echo.

REM === PART 3: Valid chain ===
echo --- PART 3: Valid chain ---
echo.
echo Verifying a full execution chain, not just one step.
echo This checks each micro receipt individually, then enforces continuity
echo across the whole sequence: step order, state linkage, and chain digest linkage.
echo.
%BIN% verify-chain %EXAMPLES%\chain_valid.jsonl
echo.
echo So this is not just validating one action. It's validating a history of actions.
echo.
echo ================================================================================
echo.

REM === PART 4: Broken chain ===
echo --- PART 4: Broken chain (state link break) ---
echo.
echo A broken chain where the state linkage between steps has been corrupted.
echo The validator identifies the exact step where continuity fails.
echo.
%BIN% verify-chain %EXAMPLES%\chain_invalid_state_link.jsonl
echo.
echo This is useful because it tells you exactly where the chain stopped being trustworthy.
echo.
echo ================================================================================
echo.

REM === PART 5: Build slab ===
echo --- PART 5: Build slab from valid chain ---
echo.
echo Compressing a valid execution chain into a macro receipt called a slab.
echo The slab keeps the chain's essential accounting and continuity summary,
echo plus a Merkle root for integrity.
echo.
%BIN% build-slab %EXAMPLES%\chain_valid.jsonl --out %EXAMPLES%\demo_slab.json
echo.
echo This is the compression step.
echo You don't always want to replay the whole chain - sometimes you want
echo a verified macro artifact.
echo.
echo ================================================================================
echo.

REM === PART 6: Verify slab ===
echo --- PART 6: Verify the slab ---
echo.
echo Verifying the slab as a standalone artifact.
echo This does not replay the original chain.
echo It validates the slab's own macro invariants and accounting consistency.
echo.
%BIN% verify-slab %EXAMPLES%\demo_slab.json
echo.
echo So the system supports both fine-grained verification and macro-level
echo compressed verification.
echo.
echo ================================================================================
echo.

REM === PART 7: Broken slab (optional add-on) ===
echo --- PART 7: Broken slab (optional) ---
echo.
echo If the slab summary is tampered with, it gets rejected at the macro layer too.
echo.
%BIN% verify-slab %EXAMPLES%\slab_invalid_summary.json
echo.
echo ================================================================================
echo.

REM === CLEANUP ===
if exist %EXAMPLES%\demo_slab.json del %EXAMPLES%\demo_slab.json

echo ================================================================================
echo CLOSING
echo ================================================================================
echo.
echo It's a deterministic validator for state transitions, chains, and
echo compressed execution receipts - with tamper detection and explicit
echo rejection reasons built in.
echo.
echo ================================================================================
echo                          DEMO COMPLETE
echo ================================================================================