# AI Workflow Demo — Customer Support Agent Trace

Task ID: AGENT-2026-0001
Tenant: demo.customer
Objective: summarize inbound customer ticket and draft a safe response

## Timeline

08:00:12 TASK_RECEIVED
- Ticket ID: CS-88421
- Customer asks why invoice changed
- Priority: normal

08:00:14 PLAN_CREATED
- Agent plans:
  1. retrieve account notes
  2. retrieve invoice history
  3. summarize issue
  4. draft response

08:00:18 TOOL_CALLED
- Tool: crm_lookup
- Query: account history for CS-88421

08:00:19 TOOL_RESULT_APPLIED
- CRM notes added to workspace
- Invoice explanation context attached

08:00:24 FINAL_RESPONSE_EMITTED
- Draft response prepared for human review
- No policy breach

## State Hashes

- State 0 (TASK_RECEIVED): sha256:a1b2c3d4e5f6...
- State 1 (PLAN_CREATED): sha256:b2c3d4e5f6a7...
- State 2 (TOOL_CALLED): sha256:c3d4e5f6a7b8...
- State 3 (TOOL_RESULT_APPLIED): sha256:d4e5f6a7b8c9...
- State 4 (FINAL_RESPONSE_EMITTED): sha256:e5f6a7b8c9d0...

## Metrics Interpretation

- v_pre = unresolved task risk before step
- v_post = unresolved task risk after step  
- spend = compute/operation cost of step
- defect = tolerated uncertainty or audit slack

Step 0: v_pre=100, v_post=88, spend=12, defect=0
Step 1: v_pre=88, v_post=80, spend=7, defect=1
Step 2: v_pre=80, v_post=68, spend=11, defect=0
Step 3: v_pre=68, v_post=55, spend=12, defect=0

Total spend = 42
Total defect = 1
First v_pre = 100
Last v_post = 55

Macro check: 55 + 42 <= 100 + 1 => 97 <= 101 ✓