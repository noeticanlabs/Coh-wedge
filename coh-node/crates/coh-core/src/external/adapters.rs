use crate::canon::{to_canonical_json_bytes, to_prehash_view, EXPECTED_CANON_PROFILE_HASH};
use crate::hash::compute_chain_digest;
use crate::types::{MetricsWire, MicroReceipt, MicroReceiptWire, RejectCode, SignatureWire};

use super::domain::{FailureInjector, FailureMode};

const POLICY_HASH_ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn reseal_in_place(wire: &mut MicroReceiptWire) -> Result<(), RejectCode> {
    let runtime = MicroReceipt::try_from(wire.clone())?;
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash)?;
    wire.chain_digest_next = compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex();
    Ok(())
}

fn hex64_of(v: u128) -> String {
    format!("{:064x}", v)
}

fn valid_signature(step_index: u64, signer: &str) -> SignatureWire {
    SignatureWire {
        signature: format!("sig-{:016}", step_index),
        signer: signer.to_string(),
        public_key: None,
        timestamp: 1_700_000_000 + step_index,
    }
}

pub trait DomainAdapter: FailureInjector {
    fn name(&self) -> &'static str;
    fn build_valid(&self, step_index: u64, prev_digest: &str, prev_state: &str)
        -> MicroReceiptWire;
}

// ──────────────────────────────────────────────────────────────────────────────
// Financial Adapter
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Default, Clone, Copy)]
pub struct FinancialAdapter;

impl FinancialAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl DomainAdapter for FinancialAdapter {
    fn name(&self) -> &'static str {
        "financial"
    }

    fn build_valid(
        &self,
        step_index: u64,
        prev_digest: &str,
        prev_state: &str,
    ) -> MicroReceiptWire {
        // Derive a budget; if prev_state is zero, start at 100_000
        let v_pre = match u128::from_str_radix(prev_state, 16).ok() {
            Some(0) | None => 100_000u128,
            Some(v) => v,
        };
        let spend = (v_pre / 20).max(1); // 5% spend
        let v_post = v_pre.saturating_sub(spend);

        let mut wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: format!("financial.workflow.{}", step_index),
            public_key: None,
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: POLICY_HASH_ZERO.to_string(),
            step_index,
            step_type: Some("financial".to_string()),
            signatures: Some(vec![valid_signature(step_index, "finance-signer")]),
            state_hash_prev: prev_state.to_string(),
            state_hash_next: hex64_of(v_post),
            chain_digest_prev: prev_digest.to_string(),
            chain_digest_next: POLICY_HASH_ZERO.to_string(),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
            },
        };

        // Safe to unwrap in builder context
        reseal_in_place(&mut wire).expect("reseal failed");
        wire
    }
}

impl FailureInjector for FinancialAdapter {
    fn inject(&self, receipt: &mut MicroReceiptWire, mode: FailureMode) {
        match mode {
            FailureMode::OverBudget => {
                // spend > v_pre but satisfy policy inequality to trigger SpendExceedsBalance
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(100);
                let bump: u128 = 5;
                let spend = v_pre + bump;
                receipt.metrics.spend = spend.to_string();
                receipt.metrics.v_post = "0".to_string();
                // Set defect to cover arithmetic: 0 + spend <= v_pre + defect
                receipt.metrics.defect = bump.to_string();
                reseal_in_place(receipt).ok();
            }
            FailureMode::MissingApproval => {
                // Simulate missing approval by breaking accounting without authority
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(100);
                let spend: u128 = (v_pre / 10).max(1);
                let v_post = v_pre.saturating_sub(spend).saturating_add(1); // +1 corruption
                receipt.metrics.v_post = v_post.to_string();
                receipt.metrics.authority = "0".to_string();
                receipt.metrics.defect = "0".to_string();
                reseal_in_place(receipt).ok();
            }
            FailureMode::StateCorruption => {
                // Mutate state after a valid seal to force RejectChainDigest
                reseal_in_place(receipt).ok();
                receipt.state_hash_next = hex64_of(42);
                // do NOT reseal
            }
            FailureMode::MissingInspection => {
                // Not typical for finance, but map to signature missing
                receipt.signatures = None;
                reseal_in_place(receipt).ok();
            }
            FailureMode::TokenHallucination
            | FailureMode::HiddenToolFailure
            | FailureMode::Overtime
            | FailureMode::InventoryCorruption => {
                // Map to generic policy violation: make v_post too high
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(100);
                let spend: u128 = (v_pre / 20).max(1);
                let v_post = v_pre.saturating_sub(spend).saturating_add(10);
                receipt.metrics.v_post = v_post.to_string();
                receipt.metrics.defect = "0".to_string();
                receipt.metrics.authority = "0".to_string();
                reseal_in_place(receipt).ok();
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Agent Adapter
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Default, Clone, Copy)]
pub struct AgentAdapter;

impl AgentAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl DomainAdapter for AgentAdapter {
    fn name(&self) -> &'static str {
        "agent"
    }

    fn build_valid(
        &self,
        step_index: u64,
        prev_digest: &str,
        prev_state: &str,
    ) -> MicroReceiptWire {
        // Token budget defaults to 1000 if prev_state is zero
        let v_pre = match u128::from_str_radix(prev_state, 16).ok() {
            Some(0) | None => 1_000u128,
            Some(v) => v,
        };
        let spend = (v_pre / 8).max(1); // 12.5% tokens used
        let v_post = v_pre.saturating_sub(spend);

        let mut wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: format!("agent.workflow.{}", step_index),
            public_key: None,
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: POLICY_HASH_ZERO.to_string(),
            step_index,
            step_type: Some("agent".to_string()),
            signatures: Some(vec![valid_signature(step_index, "agent-signer")]),
            state_hash_prev: prev_state.to_string(),
            state_hash_next: hex64_of(v_post),
            chain_digest_prev: prev_digest.to_string(),
            chain_digest_next: POLICY_HASH_ZERO.to_string(),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: "0".to_string(),
                authority: "1".to_string(), // tool success
            },
        };
        reseal_in_place(&mut wire).expect("reseal failed");
        wire
    }
}

impl FailureInjector for AgentAdapter {
    fn inject(&self, receipt: &mut MicroReceiptWire, mode: FailureMode) {
        match mode {
            FailureMode::TokenHallucination => {
                // spend > v_pre but keep inequality satisfied
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(100);
                let bump: u128 = 25;
                let spend = v_pre + bump;
                receipt.metrics.spend = spend.to_string();
                receipt.metrics.v_post = "0".to_string();
                receipt.metrics.defect = bump.to_string();
                reseal_in_place(receipt).ok();
            }
            FailureMode::HiddenToolFailure => {
                // Pretend success but violate accounting slightly
                receipt.metrics.authority = "1".to_string();
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(100);
                let spend: u128 = (v_pre / 10).max(1);
                let v_post = v_pre.saturating_sub(spend).saturating_add(2);
                receipt.metrics.v_post = v_post.to_string();
                reseal_in_place(receipt).ok();
            }
            FailureMode::StateCorruption => {
                reseal_in_place(receipt).ok();
                receipt.state_hash_next = hex64_of(1337);
            }
            FailureMode::MissingInspection => {
                // For agent, treat as missing authorization signature
                receipt.signatures = None;
                reseal_in_place(receipt).ok();
            }
            FailureMode::OverBudget
            | FailureMode::MissingApproval
            | FailureMode::Overtime
            | FailureMode::InventoryCorruption => {
                // Generic policy violation
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(100);
                let spend: u128 = (v_pre / 8).max(1);
                let v_post = v_pre.saturating_sub(spend).saturating_add(5);
                receipt.metrics.v_post = v_post.to_string();
                receipt.metrics.defect = "0".to_string();
                receipt.metrics.authority = "0".to_string();
                reseal_in_place(receipt).ok();
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Ops Adapter
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Default, Clone, Copy)]
pub struct OpsAdapter;

impl OpsAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl DomainAdapter for OpsAdapter {
    fn name(&self) -> &'static str {
        "ops"
    }

    fn build_valid(
        &self,
        step_index: u64,
        prev_digest: &str,
        prev_state: &str,
    ) -> MicroReceiptWire {
        // Labor-hour budget: default 40
        let v_pre = match u128::from_str_radix(prev_state, 16).ok() {
            Some(0) | None => 40u128,
            Some(v) => v,
        };
        let spend = (v_pre / 5).max(1); // 20% hours used
        let v_post = v_pre.saturating_sub(spend);

        let mut wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: format!("ops.workflow.{}", step_index),
            public_key: None,
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: POLICY_HASH_ZERO.to_string(),
            step_index,
            step_type: Some("ops".to_string()),
            signatures: Some(vec![valid_signature(step_index, "ops-signer")]),
            state_hash_prev: prev_state.to_string(),
            state_hash_next: hex64_of(v_post),
            chain_digest_prev: prev_digest.to_string(),
            chain_digest_next: POLICY_HASH_ZERO.to_string(),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: "0".to_string(),
                authority: "1".to_string(), // inspector approved
            },
        };
        reseal_in_place(&mut wire).expect("reseal failed");
        wire
    }
}

impl FailureInjector for OpsAdapter {
    fn inject(&self, receipt: &mut MicroReceiptWire, mode: FailureMode) {
        match mode {
            FailureMode::Overtime => {
                // spend > v_pre and satisfy inequality
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(10);
                let bump: u128 = 3;
                let spend = v_pre + bump;
                receipt.metrics.spend = spend.to_string();
                receipt.metrics.v_post = "0".to_string();
                receipt.metrics.defect = bump.to_string();
                reseal_in_place(receipt).ok();
            }
            FailureMode::MissingInspection => {
                receipt.signatures = Some(vec![]);
                reseal_in_place(receipt).ok();
            }
            FailureMode::InventoryCorruption => {
                // v_post increases while spending occurs -> Policy violation
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(10);
                let spend: u128 = (v_pre / 4).max(1);
                let v_post = v_pre.saturating_sub(spend).saturating_add(spend + 1);
                receipt.metrics.spend = spend.to_string();
                receipt.metrics.v_post = v_post.to_string();
                receipt.metrics.authority = "0".to_string();
                receipt.metrics.defect = "0".to_string();
                reseal_in_place(receipt).ok();
            }
            FailureMode::StateCorruption => {
                reseal_in_place(receipt).ok();
                receipt.state_hash_next = hex64_of(7);
            }
            FailureMode::OverBudget
            | FailureMode::MissingApproval
            | FailureMode::TokenHallucination
            | FailureMode::HiddenToolFailure => {
                // Generic policy violation
                let v_pre: u128 = receipt.metrics.v_pre.parse().unwrap_or(10);
                let spend: u128 = (v_pre / 5).max(1);
                let v_post = v_pre.saturating_sub(spend).saturating_add(1);
                receipt.metrics.v_post = v_post.to_string();
                receipt.metrics.defect = "0".to_string();
                receipt.metrics.authority = "0".to_string();
                reseal_in_place(receipt).ok();
            }
        }
    }
}
