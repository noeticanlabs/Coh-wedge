# APE Runtime Strategies Implementation Plan

## Overview
Add 5 new runtime/codebase failure strategies to APE, making 15 total strategies across 3 families:
1. **Structural** (5): mutation, recombination, violation, overflow, contradiction
2. **AI Failure Modes** (5): spec_gaming, dist_shift, temporal_drift, ambiguity, adv_alignment
3. **Runtime/Codebase** (5): non_termination, livelock, state_explosion, resource_exhaustion, parser_pathology

## New Strategies

### 1. NonTermination
- **Goal**: Create candidates that cause non-termination or infinite loop patterns
- **Techniques**:
  - Repeated identical state transitions
  - Oscillation between two states  
  - Zero-progress transitions
- **Invariant Tested**: Bounded step progress

### 2. ParserPathology  
- **Goal**: Generate structurally nasty but superficially plausible inputs
- **Techniques**:
  - Duplicate keys in JSON
  - Giant strings
  - Malformed optional values
  - Weird encodings
- **Invariant Tested**: Parsing must be total/deterministic/schema-safe

### 3. Livelock
- **Goal**: Generate candidates that trigger retry storms
- **Techniques**:
  - Reject → retry → reject cycles
  - Repeated regeneration under same invalid conditions
- **Invariant Tested**: Retry budget

### 4. StateExplosion
- **Goal**: Generate candidates causing combinatorial growth
- **Techniques**:
  - Oversized object graphs
  - Deeply nested receipts
  - Massive fan-out
- **Invariant Tested**: Verification complexity within bounds

### 5. ResourceExhaustion
- **Goal**: Generate candidates near memory/time/depth limits
- **Techniques**:
  - Near-maximum chain lengths
  - Large numeric values approaching limits
  - Deep recursion in structure
- **Invariant Tested**: Resource budgets

## Implementation Steps

### Step 1: Update Strategy Enum
File: `ape/src/proposal.rs`
- Add 5 new variants to Strategy enum
- Add name() mappings
- Add note() mappings  
- Add generate() dispatch cases
- Update all() to return [Strategy; 15]

### Step 2: Create Runtime Strategies Module
File: `ape/src/strategies/runtime.rs`
- Implement non_termination()
- Implement parser_pathology()
- Implement livelock()
- Implement state_explosion()
- Implement resource_exhaustion()

### Step 3: Register Module
File: `ape/src/strategies/mod.rs`
- Add `pub mod runtime;`

### Step 4: Test
Run: `cargo run --manifest-path ape/Cargo.toml --bin ape -- strategy-demo --iterations 20`

## Expected Output
15 strategies × 20 iterations = 300 candidates tested
