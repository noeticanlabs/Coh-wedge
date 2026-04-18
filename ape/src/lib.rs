//! APE - Adversarial / Exploratory Proposal Engine
//!
//! A deterministic, strategy-driven system that generates structured candidate states
//! for stress-testing Coh Wedge verification.
//!
//! ## Architecture
//!
//! ```mermaid
//! flowchart LR
//!     A[APE / LLM / External Source] --> B[Coh Wedge Verifier]
//!     B --> C[Decision: Accept or Reject]
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ape::{engine::generate, Strategy, load_micro, proposal::Input};
//!
//! let wire = load_micro("valid_micro.json").unwrap();
//! let input = Input::from_micro(wire);
//! let proposal = generate(Strategy::Mutation, &input, 42);
//! ```

pub mod adapter;
pub mod engine;
pub mod fixtures;
pub mod http;
pub mod pipeline;
pub mod proposal;
pub mod realdata;
pub mod seed;
pub mod strategies;

pub use adapter::{LlmAdapter, LlmResponse, MockLlmAdapter};
pub use engine::generate;
pub use fixtures::{load_chain, load_micro, load_slab, FixtureError};
pub use http::{
    execute_verified, save_valid_receipts_to_jsonl, ExecuteVerifiedRequest, SidecarResponse,
};
pub use pipeline::{run_pipeline, PipelineResult};
pub use proposal::Strategy;
pub use proposal::{Candidate, Proposal};
pub use realdata::{
    ensure_output_dir, generate_runtime_ai_chain, generate_runtime_ai_micro, load_ai_demo_chain,
    load_ai_demo_micro, load_dashboard_valid_chain, write_output_json,
};
pub use seed::SeededRng;
