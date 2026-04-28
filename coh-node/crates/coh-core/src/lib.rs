pub mod auth;
pub mod build_slab;
pub mod canon;
pub mod execute;
pub mod fixtures;
pub mod hash;
pub mod math;
pub mod merkle;
pub mod reject;
pub mod semantic;
pub mod trajectory;
pub mod trajectory_probability;
pub mod types;
pub mod vectors;
pub mod verify_chain;
pub mod verify_micro;
pub mod verify_slab;
// V3 extensions
pub mod fuzz;
pub mod types_v3;
pub mod verify_micro_v3;

pub use auth::{
    canonical_signed_transition_bytes, decode_signature, decode_verifying_key, fixture_signing_key,
    sign_micro_receipt, verify_signature, ScopePolicy, TrustedAuthority, VerifierContext,
    COHENC_V1_SIGNED_TRANSITION_TAG, DEFAULT_SCOPE,
};
pub use build_slab::build_slab;
pub use execute::{ExecuteResponse, ExecutionEngine, ExecutionMode};
pub use fixtures::{compute_micro_digest_hex, finalize_micro_receipt};
pub use verify_chain::verify_chain;
pub use verify_micro::{verify_micro, verify_micro_with_context};
pub use verify_slab::{verify_slab_envelope, verify_slab_with_leaves};
// V3 exports
pub use types_v3::{
    MicroReceiptV3, MicroReceiptV3Wire, ObjectiveResult, ObjectiveTarget, PolicyGovernance,
    SequenceGuard, TieredConfig, VerificationMode,
};
pub use verify_micro_v3::{verify_micro_v3, verify_with_mode, VerifyMicroV3Result};
// Trajectory probability exports
pub use trajectory_probability::{
    TrajectoryProbabilityConfig, TrajectoryProbabilityResult, TrajectoryProbabilityVerifier,
};

pub use types::{BuildSlabResult, VerifyChainResult, VerifyMicroResult, VerifySlabResult};
pub use types::{Decision, MicroReceiptWire, RejectCode, SlabReceiptWire};
