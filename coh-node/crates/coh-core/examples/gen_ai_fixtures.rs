//! Generate AI workflow demo fixtures with proper digests.
//!
//! Contract notes:
//! - emits bounded valid-chain fixtures only
//! - emits semi-realistic workflow fixtures
//! - does not claim support for >1000-step acceptance because the verifier
//!   currently enforces a depth budget

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire, SignatureWire};
use std::convert::TryFrom;
use std::fs;
use std::io::Write;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn main() {
    fs::create_dir_all("examples/ai_demo").unwrap();
    fs::create_dir_all("vectors/valid").unwrap();
    fs::create_dir_all("vectors/semi_realistic").unwrap();

    let state_hashes = [
        "1".repeat(64),
        "2".repeat(64),
        "3".repeat(64),
        "4".repeat(64),
        "5".repeat(64),
    ];

    let mut receipt0 = create_receipt(
        0,
        &state_hashes[0],
        &state_hashes[1],
        "100",
        "88",
        "12",
        "0",
        "0",
    );
    let digest0 = compute_digest(&receipt0);
    receipt0.chain_digest_next = digest0.clone();

    let mut receipt1 = create_receipt(
        1,
        &state_hashes[1],
        &state_hashes[2],
        "88",
        "80",
        "7",
        "1",
        "0",
    );
    receipt1.chain_digest_prev = digest0.clone();
    let digest1 = compute_digest(&receipt1);
    receipt1.chain_digest_next = digest1.clone();

    let mut receipt2 = create_receipt(
        2,
        &state_hashes[2],
        &state_hashes[3],
        "80",
        "68",
        "11",
        "0",
        "0",
    );
    receipt2.chain_digest_prev = digest1.clone();
    let digest2 = compute_digest(&receipt2);
    receipt2.chain_digest_next = digest2.clone();

    let mut receipt3 = create_receipt(
        3,
        &state_hashes[3],
        &state_hashes[4],
        "68",
        "55",
        "12",
        "0",
        "0",
    );
    receipt3.chain_digest_prev = digest2.clone();
    let digest3 = compute_digest(&receipt3);
    receipt3.chain_digest_next = digest3.clone();

    // State snapshots
    fs::write(
        "examples/ai_demo/state_0.json",
        serde_json::to_string_pretty(&serde_json::json!({
            "task_id": "AGENT-2026-0001",
            "status": "TASK_RECEIVED",
            "workspace_ready": false,
            "tool_calls": 0,
            "response_ready": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        "examples/ai_demo/state_1.json",
        serde_json::to_string_pretty(&serde_json::json!({
            "task_id": "AGENT-2026-0001",
            "status": "PLAN_CREATED",
            "workspace_ready": true,
            "tool_calls": 0,
            "response_ready": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        "examples/ai_demo/state_2.json",
        serde_json::to_string_pretty(&serde_json::json!({
            "task_id": "AGENT-2026-0001",
            "status": "TOOL_CALLED",
            "workspace_ready": true,
            "tool_calls": 1,
            "response_ready": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        "examples/ai_demo/state_3.json",
        serde_json::to_string_pretty(&serde_json::json!({
            "task_id": "AGENT-2026-0001",
            "status": "TOOL_RESULT_APPLIED",
            "workspace_ready": true,
            "tool_calls": 1,
            "response_ready": false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        "examples/ai_demo/state_4.json",
        serde_json::to_string_pretty(&serde_json::json!({
            "task_id": "AGENT-2026-0001",
            "status": "FINAL_RESPONSE_EMITTED",
            "workspace_ready": true,
            "tool_calls": 1,
            "response_ready": true
        }))
        .unwrap(),
    )
    .unwrap();

    // Valid fixtures
    fs::write(
        "examples/ai_demo/ai_workflow_micro_valid.json",
        serde_json::to_string_pretty(&receipt0).unwrap(),
    )
    .unwrap();
    fs::write(
        "examples/ai_demo/ai_workflow_chain_valid.jsonl",
        to_jsonl(&[
            receipt0.clone(),
            receipt1.clone(),
            receipt2.clone(),
            receipt3.clone(),
        ]),
    )
    .unwrap();

    fs::write(
        "vectors/semi_realistic/ai_workflow_valid.jsonl",
        to_jsonl(&[
            receipt0.clone(),
            receipt1.clone(),
            receipt2.clone(),
            receipt3.clone(),
        ]),
    )
    .unwrap();

    // Invalid digest fixture
    let mut bad_digest = receipt0.clone();
    bad_digest.chain_digest_next = "0".repeat(64);
    fs::write(
        "examples/ai_demo/ai_workflow_micro_invalid_digest.json",
        serde_json::to_string_pretty(&bad_digest).unwrap(),
    )
    .unwrap();

    // Invalid state-link fixture: recompute downstream digests so failure is state linkage, not digest mismatch.
    let mut bad_state_step2 = receipt2.clone();
    bad_state_step2.state_hash_prev = "f".repeat(64);
    bad_state_step2.chain_digest_next = compute_digest(&bad_state_step2);

    let mut bad_state_step3 = receipt3.clone();
    bad_state_step3.chain_digest_prev = bad_state_step2.chain_digest_next.clone();
    bad_state_step3.chain_digest_next = compute_digest(&bad_state_step3);

    fs::write(
        "examples/ai_demo/ai_workflow_chain_invalid_state_link.jsonl",
        to_jsonl(&[
            receipt0.clone(),
            receipt1.clone(),
            bad_state_step2.clone(),
            bad_state_step3.clone(),
        ]),
    )
    .unwrap();

    // Invalid step-index fixture: recompute digest for changed final step.
    let mut bad_step = receipt3.clone();
    bad_step.step_index = 4;
    bad_step.chain_digest_next = compute_digest(&bad_step);
    fs::write(
        "examples/ai_demo/ai_workflow_chain_invalid_step_index.jsonl",
        to_jsonl(&[
            receipt0.clone(),
            receipt1.clone(),
            receipt2.clone(),
            bad_step.clone(),
        ]),
    )
    .unwrap();

    // Bounded valid fixtures for acceptance-path testing.
    write_jsonl_file(
        "vectors/valid/valid_chain_10.jsonl",
        &build_valid_chain("agent.valid.10", 10),
    );
    write_jsonl_file(
        "vectors/valid/valid_chain_100.jsonl",
        &build_valid_chain("agent.valid.100", 100),
    );
    write_jsonl_file(
        "vectors/valid/valid_chain_1000.jsonl",
        &build_valid_chain("agent.valid.1000", 1000),
    );

    // Semi-realistic mixed-distribution fixture.
    write_jsonl_file(
        "vectors/semi_realistic/ai_workflow_noisy.jsonl",
        &build_semi_realistic_chain("agent.workflow.noisy", 24),
    );

    println!("Generated AI workflow fixtures in examples/ai_demo");
    println!("Generated bounded valid fixtures in vectors/valid");
    println!("Generated semi-realistic fixtures in vectors/semi_realistic");
    println!("STEP0_DIGEST={}", digest0);
    println!("STEP1_DIGEST={}", digest1);
    println!("STEP2_DIGEST={}", digest2);
    println!("STEP3_DIGEST={}", digest3);
}

fn create_receipt(
    step_index: u64,
    state_hash_prev: &str,
    state_hash_next: &str,
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
    authority: &str,
) -> MicroReceiptWire {
    MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.workflow.demo".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index,
        step_type: Some("workflow".to_string()),
        signatures: Some(vec![signature_for(step_index)]),
        state_hash_prev: state_hash_prev.to_string(),
        state_hash_next: state_hash_next.to_string(),
        chain_digest_prev: ZERO_HASH.to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
            authority: authority.to_string(),
        },
    }
}

fn compute_digest(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}

fn signature_for(step_index: u64) -> SignatureWire {
    SignatureWire {
        signature: format!("sig-{:016x}", step_index),
        signer: format!("fixture-signer-{}", step_index % 4),
        timestamp: 1_700_000_000 + step_index,
    }
}

fn to_jsonl(receipts: &[MicroReceiptWire]) -> String {
    let mut out = String::new();
    for receipt in receipts {
        out.push_str(&serde_json::to_string(receipt).unwrap());
        out.push('\n');
    }
    out
}

fn write_jsonl_file(path: &str, receipts: &[MicroReceiptWire]) {
    let mut file = fs::File::create(path).unwrap();
    for receipt in receipts {
        writeln!(file, "{}", serde_json::to_string(receipt).unwrap()).unwrap();
    }
}

fn defect_for_step(step: u64) -> &'static str {
    match step % 3 {
        0 => "2",
        1 => "1",
        _ => "0",
    }
}

fn next_state(seed: u64) -> String {
    format!("{:064x}", 0x100000_u64 + seed)
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
                authority: "0".to_string(),
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
                authority: "0".to_string(),
            },
        };
        receipt.chain_digest_next = compute_digest(&receipt);
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = receipt.state_hash_next.clone();
        out.push(receipt);
    }

    out
}
