# APE Trust Kernel - Architecture Specification

## Core Flow

```
AI (LLM) → Proposed Action → Verifier (Coh Wedge) → Approved Action → Execution
```

### Phase 1: AI Proposes

```rust
// AI generates a proposed state transition
let proposal = llm_adapter.generate("transfer 100 tokens to alice");
```

### Phase 2: APE Generates Candidate

```rust
// APE wraps the proposal as a candidate
let candidate = proposal.to_micro_receipt();
```

### Phase 3: Coh Wedge Verifies

```rust
// Verify against trust kernel invariants
let result = verify_micro(candidate);

match result.decision {
    Decision::Accept => execute(candidate),
    Decision::Reject => log_rejection(result.code),
}
```

### Phase 4: Execute Approved Action

```rust
// Only execute if accepted
if result.decision == Decision::Accept {
    apply_state(candidate.state_hash_next);
}
```

## Trust Invariants

| Invariant | Expression | Verification |
|----------|------------|-------------|
| Accounting | `v_post + spend ≤ v_pre + defect` | VerifyMicro |
| Chain Continuity | `chain_digest[i] == chain_digest_prev[i+1]` | VerifyChain |
| State Continuity | `state_hash[i] == state_hash_prev[i+1]` | VerifyChain |
| Schema | `schema_id == "coh.receipt.micro.v1"` | VerifyMicro |
| No Overflow | `v_pre + defect` arithmetic succeeds | VerifyMicro |

## Trust Kernel Properties

1. **Atomic Gating**: No state change commits without verification
2. **Deterministic**: Same input → same output (reproducible)
3. **Complete**: All invariants checked before execution
4. **Tamper-Evident**: Digest chain enables detection

## Integration with LLM

```rust
// Full integration pattern
loop {
    // 1. AI proposes
    let action = llm.step();
    
    // 2. APE wraps as candidate
    let receipt = action.to_micro_receipt();
    
    // 3. Trust kernel verifies
    let result = verify_micro(receipt);
    
    // 4. Decision gate
    match result.decision {
        Decision::Accept => {
            apply(action);
            commit_state(receipt);
        }
        Decision::Reject => {
            log_failure(result.code);
            // Do NOT execute
        }
        _ => {} // SlabBuilt handled separately
    }
}