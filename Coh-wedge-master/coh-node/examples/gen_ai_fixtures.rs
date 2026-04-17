//! Generate AI workflow demo fixtures with proper digests.
//!
//! Scope:
//! - emits bounded valid-chain fixtures aligned with current verifier rules
//! - emits semi-realistic AI workflow fixtures
//! - does not claim unbounded trajectory coverage

use coh_core::types::{MetricsWire, MicroReceiptWire, SignatureWire};
use coh_core::{canon::*, hash::compute_chain_digest};
use std::convert::TryFrom;
use std::fs::{self, File};
use std::io::Write;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn main() {
    let out_valid_dir = "vectors/valid";
    let out_realistic_dir = "vectors/semi_realistic";
    fs::create_dir_all(out_valid_dir).unwrap();
    fs::create_dir_all(out_realistic_dir).unwrap();

    // AI workflow states
    let state_hashes = [
        "a1b2c3d4e5f6789012345678901234567890123456789012345678901234", // state_0
        "b2c3d4e5f6a7890123456789012345678901234567890123456789012345", // state_1
        "c3d4e5f6a7b8901234567890123456789012345678901234567890123456", // state_2
        "d4e5f6a7b8c9012345678901234567890123456789012345678901234567", // state_3
        "e5f6a7b8c9d0123456789012345678901234567890123456789012345678", // state_4
    ];

    // Create step 0: TASK_RECEIVED -> PLAN_CREATED
    let mut receipt0 = create_receipt(
        0,
        &state_hashes[0],
        &state_hashes[1],
        "100",
        "88",
        "12",
        "0",
    );
    let digest0 = compute_digest(&receipt0);
    receipt0.chain_digest_next = digest0.clone();
    println!("Step 0 digest: {}", digest0);

    // Create step 1: PLAN_CREATED -> TOOL_CALLED
    let mut receipt1 = create_receipt(1, &state_hashes[1], &state_hashes[2], "88", "80", "7", "1");
    receipt1.chain_digest_prev = digest0.clone();
    let digest1 = compute_digest(&receipt1);
    receipt1.chain_digest_next = digest1.clone();
    println!("Step 1 digest: {}", digest1);

    // Create step 2: TOOL_CALLED -> TOOL_RESULT_APPLIED
    let mut receipt2 = create_receipt(2, &state_hashes[2], &state_hashes[3], "80", "68", "11", "0");
    receipt2.chain_digest_prev = digest1.clone();
    let digest2 = compute_digest(&receipt2);
    receipt2.chain_digest_next = digest2.clone();
    println!("Step 2 digest: {}", digest2);

    // Create step 3: TOOL_RESULT_APPLIED -> FINAL_RESPONSE_EMITTED
    let mut receipt3 = create_receipt(3, &state_hashes[3], &state_hashes[4], "68", "55", "12", "0");
    receipt3.chain_digest_prev = digest2.clone();
    let digest3 = compute_digest(&receipt3);
    receipt3.chain_digest_next = digest3.clone();
    println!("Step 3 digest: {}", digest3);

    println!("\n=== Valid micro receipt (step 0) ===");
    println!("{}", serde_json::to_string_pretty(&receipt0).unwrap());

    println!("\n=== Valid chain JSONL ===");
    let jsonl0 = serde_json::to_string(&receipt0).unwrap();
    let jsonl1 = serde_json::to_string(&receipt1).unwrap();
    let jsonl2 = serde_json::to_string(&receipt2).unwrap();
    let jsonl3 = serde_json::to_string(&receipt3).unwrap();
    println!("{}", jsonl0);
    println!("{}", jsonl1);
    println!("{}", jsonl2);
    println!("{}", jsonl3);

    write_jsonl(
        "vectors/semi_realistic/ai_workflow_valid.jsonl",
        &[
            receipt0.clone(),
            receipt1.clone(),
            receipt2.clone(),
            receipt3.clone(),
        ],
    );

    let valid_chain_10 = build_valid_chain("agent.valid.acceptance", 10);
    let valid_chain_100 = build_valid_chain("agent.valid.acceptance", 100);
    let bounded_chain_1000 = build_valid_chain("agent.valid.acceptance", 1000);
    let noisy_chain_24 = build_semi_realistic_chain("agent.workflow.noisy", 24);

    write_jsonl("vectors/valid/valid_chain_10.jsonl", &valid_chain_10);
    write_jsonl("vectors/valid/valid_chain_100.jsonl", &valid_chain_100);
    write_jsonl("vectors/valid/valid_chain_1000.jsonl", &bounded_chain_1000);
    write_jsonl(
        "vectors/semi_realistic/ai_workflow_noisy.jsonl",
        &noisy_chain_24,
    );

    println!("\n=== Wrote reproducible fixtures ===");
    println!("- vectors/valid/valid_chain_10.jsonl");
    println!("- vectors/valid/valid_chain_100.jsonl");
    println!("- vectors/valid/valid_chain_1000.jsonl");
    println!("- vectors/semi_realistic/ai_workflow_valid.jsonl");
    println!("- vectors/semi_realistic/ai_workflow_noisy.jsonl");

    println!("\n=== Invalid digest micro (step 0 tampered) ===");
    let mut tampered = receipt0.clone();
    tampered.chain_digest_next =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    println!("{}", serde_json::to_string_pretty(&tampered).unwrap());

    println!("\n=== Invalid state link (step 2 with wrong prev) ===");
    let mut bad_state = receipt2.clone();
    bad_state.state_hash_prev =
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string();
    println!("{}", serde_json::to_string_pretty(&bad_state).unwrap());
}

fn create_receipt(
    step: u64,
    prev_state: &str,
    next_state: &str,
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
) -> MicroReceiptWire {
    MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.workflow.demo".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("workflow".to_string()),
        signatures: Some(vec![signature_for(step)]),
        state_hash_prev: prev_state.to_string(),
        state_hash_next: next_state.to_string(),
        chain_digest_prev: ZERO_HASH.to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
        },
    }
}

fn compute_digest(receipt: &MicroReceiptWire) -> String {
    let r = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(r.chain_digest_prev, &bytes).to_hex()
}

fn signature_for(step: u64) -> SignatureWire {
    SignatureWire {
        signature: format!("sig-{:016x}", step),
        signer: format!("fixture-signer-{}", step % 3),
        timestamp: 1_700_000_000 + step,
    }
}

fn next_state(seed: u64) -> String {
    format!("{:064x}", 0x100000_u64 + seed)
}

fn defect_for_step(step: u64) -> &'static str {
    match step % 3 {
        0 => "2",
        1 => "1",
        _ => "0",
    }
}

fn build_valid_chain(object_id: &str, len: usize) -> Vec<MicroReceiptWire> {
    let mut out = Vec::with_capacity(len);
    let mut prev_digest = ZERO_HASH.to_string();
    let mut prev_state = ZERO_HASH.to_string();

    for step in 0..len as u64 {
        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: object_id.to_string(),
            canon_profile_hash: VALID_PROFILE.to_string(),
            policy_hash: ZERO_HASH.to_string(),
            step_index: step,
            step_type: Some("acceptance".to_string()),
            signatures: Some(vec![signature_for(step)]),
            state_hash_prev: prev_state.clone(),
            state_hash_next: next_state(step + 1),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: ZERO_HASH.to_string(),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: "99".to_string(),
                spend: "1".to_string(),
                defect: defect_for_step(step).to_string(),
            },
        };
        receipt.chain_digest_next = compute_digest(&receipt);
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = receipt.state_hash_next.clone();
        out.push(receipt);
    }

    out
}

fn build_semi_realistic_chain(object_id: &str, len: usize) -> Vec<MicroReceiptWire> {
    let step_types = ["ingest", "plan", "tool", "reflect", "revise", "finalize"];
    let spends = ["2", "1", "1", "2", "1", "1"];
    let posts = ["98", "99", "99", "98", "99", "99"];

    let mut out = Vec::with_capacity(len);
    let mut prev_digest = ZERO_HASH.to_string();
    let mut prev_state = ZERO_HASH.to_string();

    for step in 0..len as u64 {
        let idx = (step as usize) % step_types.len();
        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: object_id.to_string(),
            canon_profile_hash: VALID_PROFILE.to_string(),
            policy_hash: ZERO_HASH.to_string(),
            step_index: step,
            step_type: Some(step_types[idx].to_string()),
            signatures: Some(vec![signature_for(step)]),
            state_hash_prev: prev_state.clone(),
            state_hash_next: next_state(10_000 + step * 17 + 1),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: ZERO_HASH.to_string(),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: posts[idx].to_string(),
                spend: spends[idx].to_string(),
                defect: defect_for_step(step).to_string(),
            },
        };
        receipt.chain_digest_next = compute_digest(&receipt);
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = receipt.state_hash_next.clone();
        out.push(receipt);
    }

    out
}

fn write_jsonl(path: &str, receipts: &[MicroReceiptWire]) {
    let mut file = File::create(path).unwrap();
    for receipt in receipts {
        writeln!(file, "{}", serde_json::to_string(receipt).unwrap()).unwrap();
    }
}
