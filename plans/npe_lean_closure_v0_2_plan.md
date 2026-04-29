# NPE-Lean Closure Attempt v0.2 Plan

## Objective
Target `isRationalInf_pairwise_add` for full closure using PhaseLoom learned weights from previous benchmark loops.

## Architecture for 'Closure' Mode
1. **State Initialization**: Initialize PhaseLoom with heavily biased weights based on previously learned successful strategies. For `isRationalInf_pairwise_add`, this means prioritizing `ApproximationLemma`, `ExistsLtUsed`, and `InfAddCompatibility`.
2. **Closure Runner**: Create a dedicated executable (e.g., `coh-node/crates/coh-genesis/examples/npe_lean_closure_v0_2.rs`). This runner will operate in "closure" mode, meaning it focuses entirely on exploiting learned weights rather than uniform exploration.
3. **Biasing Mechanism**: The runner will simulate a high-exploitation sweep (e.g., 100-200 iterations) where the probability of selecting forbidden or unproductive strategies is minimized.
4. **Evaluation Loop**: Execute the sweep and track `FullPairwiseAddCompiled` outcomes versus `LeanNearMiss` or isolated lemmas.
5. **Artifact Generation**: Emit detailed `proof_graph.json` and `receipts.jsonl` artifacts that document the closure attempt, margins, and final status of the target theorem to a dedicated directory: `target/npe_wbt/lean_phaseloom/closure_v0_2/`.

## Execution Steps
1. Create the `npe_lean_closure_v0_2.rs` script in the `coh-genesis` examples directory.
2. Run the script to perform the closure sweep.
3. Analyze the results (useful outcomes vs. near misses).
4. Update the project's proof graph and receipt records with the outcomes.