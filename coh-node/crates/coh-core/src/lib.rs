pub mod build_slab;
pub mod canon;
pub mod execute;
pub mod hash;
pub mod math;
pub mod merkle;
pub mod reject;
pub mod trajectory_probability;
pub mod types;
pub mod vectors;
pub mod verify_chain;
pub mod verify_micro;
pub mod verify_slab;
// V3 extensions
pub mod types_v3;
pub mod verify_micro_v3;

pub use build_slab::build_slab;
pub use execute::{ExecuteResponse, ExecutionEngine, ExecutionMode};
pub use verify_chain::verify_chain;
pub use verify_micro::verify_micro;
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
