#![allow(clippy::needless_update)]
#![allow(clippy::too_many_arguments)]

pub mod auth;
pub mod build_slab;
pub mod canon;
pub mod execute;
pub mod fixtures;
pub mod hash;
pub mod math;
pub mod measurement;
pub mod merkle;
pub mod phaseloom;
pub mod reject;
pub mod semantic;
pub mod trajectory_probability;
pub mod types;
pub mod types_v3;
pub mod vectors;
pub mod vectors_measurement;
pub mod verify_chain;
pub mod verify_micro;
pub mod verify_micro_v3;
pub mod verify_slab;

pub mod rv_kernel;
pub mod tools;

pub use build_slab::build_slab;
pub use fixtures::finalize_micro_receipt;
pub use types::Decision;
pub use verify_chain::verify_chain;
pub use verify_micro::verify_micro;
pub use verify_slab::verify_slab_envelope;

#[cfg(test)]
mod fuzz;
