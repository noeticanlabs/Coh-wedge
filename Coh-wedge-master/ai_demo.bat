@echo off
REM AI workflow demo for Coh Validator

echo ================================================================================
echo                    COH VALIDATOR - AI WORKFLOW DEMO
echo ================================================================================
echo.
echo [1] Valid AI workflow micro receipt
coh-node\target\debug\coh-validator.exe verify-micro coh-node\examples\ai_demo\ai_workflow_micro_valid.json
echo.
echo [2] Tampered AI workflow micro receipt
coh-node\target\debug\coh-validator.exe verify-micro coh-node\examples\ai_demo\ai_workflow_micro_invalid_digest.json
echo.
echo [3] Valid AI workflow chain
coh-node\target\debug\coh-validator.exe verify-chain coh-node\examples\ai_demo\ai_workflow_chain_valid.jsonl
echo.
echo [4] Broken AI workflow state link
coh-node\target\debug\coh-validator.exe verify-chain coh-node\examples\ai_demo\ai_workflow_chain_invalid_state_link.jsonl
echo.
echo [5] Broken AI workflow step index
coh-node\target\debug\coh-validator.exe verify-chain coh-node\examples\ai_demo\ai_workflow_chain_invalid_step_index.jsonl
echo.
echo [6] Build AI workflow slab
coh-node\target\debug\coh-validator.exe build-slab coh-node\examples\ai_demo\ai_workflow_chain_valid.jsonl --out coh-node\examples\ai_demo\ai_workflow_slab_valid.json
echo.
echo [7] Verify valid AI workflow slab
coh-node\target\debug\coh-validator.exe verify-slab coh-node\examples\ai_demo\ai_workflow_slab_valid.json
echo.
echo [8] Verify invalid AI workflow slab summary
coh-node\target\debug\coh-validator.exe verify-slab coh-node\examples\ai_demo\ai_workflow_slab_invalid_summary.json
echo.
echo ================================================================================
echo AI WORKFLOW DEMO COMPLETE
echo ================================================================================
