pub mod kernel;
pub mod closure;
pub mod engine;
pub mod lineage;
pub mod loop_engine;
pub mod receipt;
pub mod rewrite;
pub mod store;
pub mod templates;
pub mod traits;
pub mod weights;
pub mod candidate;
pub mod generator;
pub mod failure_taxonomy;
pub mod tools;

pub use closure::LeanClosureStatus;
pub use engine::{NpeError, NpeProposal, ProposalStatus};
pub use lineage::NpeEdge;
#[cfg(feature = "npe-graph")]
pub use lineage::NpeProposalGraph;
pub use loop_engine::{NpeConfig, NpeEngine, NpeState};
pub use receipt::{BoundaryReceiptSummary, MathlibEffect};
pub use weights::StrategyWeights;

#[cfg(feature = "npe-store")]
pub use store::NpeStore;

#[cfg(feature = "npe-rewrite")]
pub use rewrite::NpeRewriter;

pub use traits::{NpeGenerator, NpeScorer, NpeVerifier};

/// NPE Structural Memory Update
/// Returns the updated receipt with template information
pub fn enrich_receipt_with_template(
    mut receipt: BoundaryReceiptSummary,
    goal_text: &str,
) -> BoundaryReceiptSummary {
    if let Some(template) = templates::classify_coh_template(goal_text) {
        receipt.coh_template = Some(template);
    }
    receipt
}
