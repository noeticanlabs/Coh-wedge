use crate::trajectory::domain::{admissible_actions, derive_state, is_transition_valid_semantic};
use crate::trajectory::scoring::{calculate_weighted_score, evaluate_path, ScoringWeights};
use crate::trajectory::search_result::SearchResult;
use crate::trajectory::types::{
    witness_vector, AcceptWitness, Action, AdmissibleTrajectory, CandidateEdge, DomainState,
    VerifiedStep,
};
use crate::types::{Decision, Hash32, MicroReceiptWire};
use crate::verify_micro;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContext {
    pub initial_state: DomainState,
    pub target_state: DomainState,
    pub max_depth: usize,
    pub beam_width: usize,
    pub weights: ScoringWeights,
}

/// The core Trajectory Engine implementing the 6-step pipeline
pub fn search(ctx: &SearchContext) -> SearchResult {
    let mut result = SearchResult::new();
    let mut frontier = vec![AdmissibleTrajectory::new()];

    for depth in 0..ctx.max_depth {
        let mut next_frontier = Vec::new();
        result.frontier_stats.max_depth_reached = depth + 1;

        for traj in frontier {
            let current_semantic_state = traj
                .steps
                .last()
                .map(|s| &s.state_next)
                .unwrap_or(&ctx.initial_state);

            let prev_digest = traj
                .steps
                .last()
                .map(|s| s.receipt_digest)
                .unwrap_or_default();

            // Step 1: Expand
            let actions = admissible_actions(current_semantic_state);
            
            if actions.is_empty() {
                if !traj.steps.is_empty() {
                    result.admissible.push(traj.clone());
                }
                continue;
            }

            for action in actions {
                result.frontier_stats.total_expanded += 1;

                // Step 2: Construct (and Derive state)
                let next_semantic_state = derive_state(current_semantic_state, &action);

                // Phase 2 Rigor: Explicit Semantic Guard
                let is_legally_valid = is_transition_valid_semantic(
                    current_semantic_state,
                    &action,
                    &next_semantic_state,
                );

                let wire = grounded_receipt_wire(
                    current_semantic_state,
                    &action,
                    &next_semantic_state,
                    prev_digest,
                    depth as u64,
                );

                // Step 3: Verify
                let mut verification = verify_micro(wire.clone());

                // If semantic guard failed, force Reject in search logic even if kernel accepted (defense in depth)
                if !is_legally_valid {
                    verification.decision = Decision::Reject;
                    // Note: We'd normally use a specific RejectCode here
                }

                // Step 4: Filter & Map Witness
                let witnesses = witness_vector(&verification);
                let is_accepted = verification.decision == Decision::Accept;

                if is_accepted {
                    // Step 5: Extend (Requires AcceptWitness)
                    let step = VerifiedStep::new(
                        current_semantic_state.clone(),
                        action.clone(),
                        next_semantic_state.clone(),
                        verification
                            .chain_digest_next
                            .clone()
                            .and_then(|h| Hash32::from_hex(&h).ok())
                            .unwrap_or_default(),
                        Hash32::from_hex(&wire.chain_digest_prev).unwrap_or_default(),
                        AcceptWitness, // Type-enforced admissibility
                    );

                    let mut next_traj = traj.clone();
                    next_traj.push(step);

                    // Step 6: Score Admissible Only (Lexicographic + UI Scalar)
                    let eval = evaluate_path(&next_traj, ctx.max_depth);
                    next_traj.evaluation = Some(eval);
                    next_traj.cumulative_score = calculate_weighted_score(&eval, &ctx.weights);

                    next_frontier.push(next_traj);
                    result.frontier_stats.admissible_found += 1;
                } else {
                    // Step 6b: Track Peak Violation
                    if let Some(delta) = verification.violation_delta {
                        if delta > result.max_violation_seen {
                            result.max_violation_seen = delta;
                        }
                    }

                    // Capture for Rejected graph
                    result.rejected.push(CandidateEdge {
                        state_prev: current_semantic_state.clone(),
                        action: action.clone(),
                        state_next: next_semantic_state.clone(),
                        receipt: wire,
                        verification,
                        witnesses,
                    });
                    result.frontier_stats.rejected_found += 1;
                }
            }
        }

        // Beam pruning: Lexicographic (Safety > Progress > -Cost)
        next_frontier.sort_by(|a, b| {
            let eval_a = a.evaluation.as_ref().unwrap();
            let eval_b = b.evaluation.as_ref().unwrap();
            eval_b.cmp(eval_a) // Sort descending
        });
        frontier = next_frontier.into_iter().take(ctx.beam_width).collect();

        if frontier.is_empty() {
            break;
        }
        
        if depth == ctx.max_depth - 1 {
            result.admissible.extend(frontier.clone());
        }
    }

    result
}

/// Helper to create a grounded wire receipt using actual domain evidence
fn grounded_receipt_wire(
    prev: &DomainState,
    action: &Action,
    next: &DomainState,
    prev_digest: Hash32,
    depth: u64,
) -> MicroReceiptWire {
    // Derive grounded metrics
    let (v_pre, v_post_or_meta) = prev.to_metrics_tuple();
    let (v_post, _) = next.to_metrics_tuple();

    // Derive grounded state hashes from canonical serialization
    let state_hash_prev = crate::hash::sha256(&serde_json::to_vec(prev).unwrap()).to_hex();
    let state_hash_next = crate::hash::sha256(&serde_json::to_vec(next).unwrap()).to_hex();

    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "traj.edge".to_string(),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: depth,
        step_type: Some(format!("{:?}", action)),
        signatures: Some(vec![crate::types::SignatureWire {
            signature: "sig".to_string(),
            signer: "system".to_string(),
            timestamp: 0,
        }]),
        state_hash_prev,
        state_hash_next,
        chain_digest_prev: prev_digest.to_hex(),
        chain_digest_next: "0".repeat(64),
        metrics: crate::types::MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: (v_pre.saturating_sub(v_post)).to_string(), // Simplified balance-metric logic
            defect: (if matches!(prev, DomainState::Ops(_)) {
                v_post_or_meta
            } else {
                0
            })
            .to_string(),
            authority: (if matches!(prev, DomainState::Agent(_)) {
                v_post_or_meta
            } else {
                0
            })
            .to_string(),
        },
    };

    // Compute valid chain digest to satisfy C5
    if let Ok(r) = crate::types::MicroReceipt::try_from(wire.clone()) {
        let prehash = crate::canon::to_prehash_view(&r);
        if let Ok(bytes) = crate::canon::to_canonical_json_bytes(&prehash) {
            wire.chain_digest_next =
                crate::hash::compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
        }
    }

    wire
}
