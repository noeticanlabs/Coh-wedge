@echo off
setlocal enabledelayedexpansion

echo ============================================================
echo      COH VALIDATOR: HALLUCINATION BREACH DEMO
echo ============================================================
echo.
echo [SCENARIO]:
echo An AI agent is managing a 50-step reconciliation workflow.
echo We have two audit logs:
echo   1. audit_honest.jsonl      (50 valid steps)
echo   2. audit_hallucinated.jsonl (Step 25 introduces a "value print" error)
echo.
echo [TASK]: 
echo Prove that Coh Validator acts as a deterministic circuit breaker
echo even when an agent tries to hide corruption under valid-looking steps.
echo.

echo [1/3] Generating audit logs...
cd coh-node
cargo run --quiet --package coh-core --example integrity_demo_gen
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Failed to generate logs.
    cd ..
    exit /b 1
)
cd ..
echo [OK] audit_honest.jsonl and audit_hallucinated.jsonl ready.
echo.

echo [2/3] Verifying Honest Agent Workflow...
echo Running coh-validator verify-chain coh-node\examples\audit_honest.jsonl
coh-node\target\debug\coh-validator verify-chain coh-node\examples\audit_honest.jsonl
if %ERRORLEVEL% EQU 0 (
    echo.
    echo [RESULT]: ACCEPT
    echo [STATUS]: Audit log is lawful and complete.
) else (
    echo.
    echo [RESULT]: REJECT
    echo [ERROR]: Honest trace failed verification!
    goto :FAIL
)
echo.

echo [3/3] Verifying Hallucinated Agent Workflow...
echo Running coh-validator verify-chain coh-node\examples\audit_hallucinated.jsonl
coh-node\target\debug\coh-validator verify-chain coh-node\examples\audit_hallucinated.jsonl
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [RESULT]: REJECT - DETECTED
    echo [STATUS]: Safety Kernel triggered a circuit break.
    echo [REASON]: Silent integrity decay detected at a micro-step level.
) else (
    echo.
    echo [RESULT]: ACCEPT - FAILED TO DETECT
    echo [ERROR]: Validator failed to catch the hallucination!
    goto :FAIL
)
echo.

echo ============================================================
echo      DEMO COMPLETE: DETERMINISTIC SAFETY PROVEN
echo ============================================================
pause
exit /b 0

:FAIL
echo.
echo [DEMO FAILED]
pause
exit /b 1
