//! PhaseLoom / Coh-Core Fusion Wedge
//!
//! This module integrates PhaseLoom's thermodynamic governance with coh-core's 
//! micro-verification kernel. It ensures that every micro-step is accounted for
//! within the PhaseLoom manifold.

use coh_core::types_v3::MicroReceiptV3Wire;
use coh_core::verify_micro_v3::{verify_micro_v3, VerifyMicroV3Result};
use coh_phaseloom::{PhaseLoomState, PhaseLoomConfig};
use coh_npe::receipt::BoundaryReceiptSummary;

/// Governed Micro-Step Verification
/// Wraps coh-core's verifier with PhaseLoom state injection and ingestion.
pub fn verify_governed_step(
    state: &mut PhaseLoomState,
    config: &PhaseLoomConfig,
    mut wire: MicroReceiptV3Wire,
    tiered_config: &coh_core::types_v3::TieredConfig,
    sequence_guard: &coh_core::types_v3::SequenceGuard,
    policy_gov: &coh_core::types_v3::PolicyGovernance,
    prev_state: Option<coh_core::types::Hash32>,
    prev_chain_digest: Option<coh_core::types::Hash32>,
) -> (VerifyMicroV3Result, Option<BoundaryReceiptSummary>) {
    // 1. Inject PhaseLoom state into the wire for the verifier to check
    wire.metrics.pl_tau = state.tau.to_string();
    wire.metrics.pl_budget = state.budget.to_string();
    // Note: pl_provenance is provided by the wire (the step's declared authority).
    // 2. Perform core verification
    let result = verify_micro_v3(
        wire.clone(),
        tiered_config,
        sequence_guard,
        policy_gov,
        prev_state,
        prev_chain_digest,
    );

    // 3. Map result to PhaseLoom receipt for ingestion
    let accepted = matches!(result.decision, coh_core::types::Decision::Accept);
    let receipt = BoundaryReceiptSummary {
        domain: "fusion_wedge".to_string(),
        target: wire.object_id.clone(),
        strategy_class: "core_verification".to_string(),
        accepted,
        outcome: if accepted { "accepted".to_string() } else { "rejected".to_string() },
        first_failure: result.message.clone(),
        record_tau: state.tau,
        provenance: wire.metrics.pl_provenance.clone(),
        ..BoundaryReceiptSummary::default()
    };

    // 4. Ingest and update PhaseLoom state
    state.ingest(&receipt, config);

    (result, Some(receipt))
}
