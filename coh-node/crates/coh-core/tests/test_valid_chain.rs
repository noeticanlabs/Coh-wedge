//! Valid acceptance-path and bounded-chain verification tests.
//!
//! Scope lock:
//! - in scope: acceptance of contract-valid micro receipts and bounded chains
//! - in scope: bounded-chain stress at the current verifier limit
//! - in scope: mixed/semi-realistic valid distributions
//! - out of scope: unbounded trajectory-space acceptance claims

use coh_core::canon::*;
use coh_core::hash::compute_chain_digest;
use coh_core::types::*;
use coh_core::verify_chain::verify_chain;
use std::convert::TryFrom;
use std::time::Instant;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn signature_for(index: u64) -> SignatureWire {
    SignatureWire {
        signature: format!("sig-{:016x}", index),
        signer: format!("tester-{}", index % 4),
        timestamp: 1_700_000_000 + index,
    }
}

fn defect_for_step(index: u64) -> u64 {
    match index % 3 {
        0 => 2,
        1 => 1,
        _ => 0,
    }
}

fn next_state_for(index: u64) -> String {
    format!("{:064x}", 0xabc000_u64 + index + 1)
}

fn seal_wire(wire: &mut MicroReceiptWire) {
    let r = MicroReceipt::try_from(wire.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    wire.chain_digest_next = compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
}

fn create_valid_wire(
    index: u64,
    prev_digest: String,
    prev_state: String,
    next_state: String,
) -> MicroReceiptWire {
    let defect = defect_for_step(index);
    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "test.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: index,
        step_type: Some("compute".to_string()),
        signatures: Some(vec![signature_for(index)]),
        state_hash_prev: prev_state,
        state_hash_next: next_state,
        chain_digest_prev: prev_digest,
        chain_digest_next: ZERO_HASH.to_string(),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "99".to_string(),
            spend: "1".to_string(),
            defect: defect.to_string(),
            authority: "0".to_string(),
        },
    };
    seal_wire(&mut wire);
    wire
}

#[allow(clippy::too_many_arguments)]
fn create_wire_with_metrics(
    index: u64,
    prev_digest: String,
    prev_state: String,
    next_state: String,
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
    step_type: &str,
) -> MicroReceiptWire {
    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "test.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: index,
        step_type: Some(step_type.to_string()),
        signatures: Some(vec![signature_for(index)]),
        state_hash_prev: prev_state,
        state_hash_next: next_state,
        chain_digest_prev: prev_digest,
        chain_digest_next: ZERO_HASH.to_string(),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
            authority: "0".to_string(),
        },
    };
    seal_wire(&mut wire);
    wire
}

fn create_valid_chain(len: usize) -> Vec<MicroReceiptWire> {
    let mut receipts = Vec::with_capacity(len);
    let mut prev_digest = ZERO_HASH.to_string();
    let mut prev_state = ZERO_HASH.to_string();

    for i in 0..len {
        let next_state = next_state_for(i as u64);
        let wire = create_valid_wire(
            i as u64,
            prev_digest.clone(),
            prev_state.clone(),
            next_state.clone(),
        );
        prev_digest = wire.chain_digest_next.clone();
        prev_state = next_state;
        receipts.push(wire);
    }

    receipts
}

#[test]
fn test_valid_chain_accept_1_step() {
    let chain = create_valid_chain(1);
    let res = verify_chain(chain);
    assert_eq!(res.decision, Decision::Accept);
    assert_eq!(res.steps_verified, 1);
    assert_eq!(res.last_step_index, 0);
}

#[test]
fn test_valid_chain_accept_5_steps() {
    let chain = create_valid_chain(5);
    let res = verify_chain(chain);
    assert_eq!(res.decision, Decision::Accept);
    assert_eq!(res.steps_verified, 5);
    assert_eq!(res.last_step_index, 4);
}

#[test]
fn test_valid_chain_accept_100_steps() {
    let chain = create_valid_chain(100);
    let res = verify_chain(chain);
    assert_eq!(res.decision, Decision::Accept);
    assert_eq!(res.steps_verified, 100);
    assert_eq!(res.last_step_index, 99);
}

#[test]
fn test_valid_chain_accept_1000_steps_at_bound() {
    let chain = create_valid_chain(1000);
    let res = verify_chain(chain);
    assert_eq!(res.decision, Decision::Accept);
    assert_eq!(res.steps_verified, 1000);
    assert_eq!(res.last_step_index, 999);
}

#[test]
fn test_valid_profile_standard_accepts() {
    let wire = create_wire_with_metrics(
        0,
        ZERO_HASH.to_string(),
        ZERO_HASH.to_string(),
        next_state_for(0),
        "100",
        "80",
        "20",
        "0",
        "compute",
    );
    let res = verify_chain(vec![wire]);
    assert_eq!(res.decision, Decision::Accept);
}

#[test]
fn test_valid_profile_minimal_accepts() {
    // Uses small nonzero values -- vacuous zero receipts are rejected by C1.
    let wire = create_wire_with_metrics(
        0,
        ZERO_HASH.to_string(),
        ZERO_HASH.to_string(),
        next_state_for(0),
        "1",
        "1",
        "0",
        "0",
        "noop",
    );
    let res = verify_chain(vec![wire]);
    assert_eq!(res.decision, Decision::Accept);
}

#[test]
fn test_valid_profile_maximal_single_step_accepts() {
    let wire = create_wire_with_metrics(
        0,
        ZERO_HASH.to_string(),
        ZERO_HASH.to_string(),
        next_state_for(0),
        "999999999999",
        "999999999990",
        "9",
        "0",
        "synthesis",
    );
    let res = verify_chain(vec![wire]);
    assert_eq!(res.decision, Decision::Accept);
}

#[test]
fn test_valid_non_identity_state_transition_accepts() {
    let wire = create_wire_with_metrics(
        0,
        ZERO_HASH.to_string(),
        format!("{:064x}", 0x1111_u64),
        format!("{:064x}", 0x2222_u64),
        "100",
        "99",
        "1",
        "0",
        "transition",
    );
    let res = verify_chain(vec![wire]);
    assert_eq!(res.decision, Decision::Accept);
}

#[test]
fn test_multiple_correct_chains_accept() {
    for seed in 0..3_u64 {
        let mut receipts = Vec::with_capacity(3);
        let mut prev_digest = format!("{:064x}", seed);
        let mut prev_state = format!("{:064x}", seed + 100);

        for step in 0..3_u64 {
            let next_state = format!("{:064x}", seed * 1000 + step + 1);
            let wire = create_valid_wire(
                step,
                prev_digest.clone(),
                prev_state.clone(),
                next_state.clone(),
            );
            prev_digest = wire.chain_digest_next.clone();
            prev_state = next_state;
            receipts.push(wire);
        }

        let res = verify_chain(receipts);
        assert_eq!(res.decision, Decision::Accept, "seed={seed}");
    }
}

#[test]
fn test_latency_valid_vs_invalid_same_order() {
    let valid_chain = create_valid_chain(100);

    let mut invalid_chain = create_valid_chain(100);
    invalid_chain[50].state_hash_prev = format!("{:064x}", 0xdeadbeef_u64);
    seal_wire(&mut invalid_chain[50]);

    let t0 = Instant::now();
    let valid_res = verify_chain(valid_chain);
    let valid_time = t0.elapsed();

    let t1 = Instant::now();
    let invalid_res = verify_chain(invalid_chain);
    let invalid_time = t1.elapsed();

    assert_eq!(valid_res.decision, Decision::Accept);
    assert_eq!(invalid_res.decision, Decision::Reject);
    assert_eq!(invalid_res.failing_step_index, Some(50));

    assert!(
        valid_time.as_millis() < 2000,
        "valid path too slow: {:?}",
        valid_time
    );
    assert!(
        invalid_time.as_millis() < 2000,
        "invalid path too slow: {:?}",
        invalid_time
    );

    let slower = valid_time.max(invalid_time).as_nanos() as f64;
    let faster = valid_time.min(invalid_time).as_nanos().max(1) as f64;
    let ratio = slower / faster;
    assert!(ratio < 10.0, "latency ratio too large: {}", ratio);
}

#[test]
fn test_bounded_chain_upper_limit_is_validated() {
    let start = Instant::now();
    let res = verify_chain(create_valid_chain(1000));
    let elapsed = start.elapsed();

    assert_eq!(res.decision, Decision::Accept);
    assert_eq!(res.steps_verified, 1000);
    assert!(
        elapsed.as_secs() < 10,
        "bounded chain verification too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_chain_above_bound_aborts_budget() {
    let start = Instant::now();
    let res = verify_chain(create_valid_chain(1001));
    let elapsed = start.elapsed();

    assert_eq!(res.decision, Decision::AbortBudget);
    assert_eq!(res.code, Some(RejectCode::DepthLimitExceeded));
    assert!(
        elapsed.as_secs() < 2,
        "abort-budget path too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_mixed_distribution_valid_accepts() {
    let mut receipts = Vec::with_capacity(12);
    let mut prev_digest = ZERO_HASH.to_string();
    let mut prev_state = ZERO_HASH.to_string();

    for i in 0..12_u64 {
        let defect = defect_for_step(i).to_string();
        let step_type = match i % 4 {
            0 => "plan",
            1 => "tool",
            2 => "reflect",
            _ => "synthesize",
        };
        // Use economics that satisfy the telescoping bound:
        // v_post_last + cumulative_spend <= v_pre_first + total_defect
        // With large v_pre and modest spending, this always holds.
        let spend = if i % 5 == 0 { "2" } else { "1" };
        // v_post << v_pre ensures telescoping bound is satisfied even with cumulative spend.
        let v_post = "50000";
        let next_state = format!("{:064x}", 0x5000_u64 + i * 17 + 1);

        let wire = create_wire_with_metrics(
            i,
            prev_digest.clone(),
            prev_state.clone(),
            next_state.clone(),
            "100000",
            v_post,
            spend,
            &defect,
            step_type,
        );
        prev_digest = wire.chain_digest_next.clone();
        prev_state = next_state;
        receipts.push(wire);
    }

    let res = verify_chain(receipts);
    assert_eq!(res.decision, Decision::Accept);
}

#[test]
fn test_semi_realistic_ai_workflow_accepts() {
    let steps = [
        ("ingest", "100", "98", "2", "2"),
        ("plan", "98", "97", "1", "1"),
        ("tool", "97", "96", "1", "0"),
        ("reflect", "96", "95", "1", "2"),
        ("revise", "95", "94", "1", "1"),
        ("finalize", "94", "93", "1", "0"),
    ];

    let mut receipts = Vec::with_capacity(steps.len());
    let mut prev_digest = ZERO_HASH.to_string();
    let mut prev_state = ZERO_HASH.to_string();

    for (i, (step_type, v_pre, v_post, spend, defect)) in steps.iter().enumerate() {
        let next_state = format!("{:064x}", 0x9000_u64 + i as u64 + 1);
        let wire = create_wire_with_metrics(
            i as u64,
            prev_digest.clone(),
            prev_state.clone(),
            next_state.clone(),
            v_pre,
            v_post,
            spend,
            defect,
            step_type,
        );
        prev_digest = wire.chain_digest_next.clone();
        prev_state = next_state;
        receipts.push(wire);
    }

    let res = verify_chain(receipts);
    assert_eq!(res.decision, Decision::Accept);
}
