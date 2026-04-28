//! Generate AI workflow demo fixtures with proper digests and real signatures.
//!
//! Contract notes:
//! - emits bounded valid-chain fixtures only
//! - emits semi-realistic workflow fixtures
//! - All fixtures are now properly signed with Ed25519

use coh_core::auth::{fixture_signing_key, sign_micro_receipt};
use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire, SignatureWire, AdmissionProfile};
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
    );
    let digest0 = compute_digest(&receipt0);
    receipt0.chain_digest_next = digest0.clone();

    let mut receipt1 = create_receipt(1, &state_hashes[1], &state_hashes[2], "88", "80", "7", "1");
    receipt1.chain_digest_prev = digest0.clone();
    let digest1 = compute_digest(&receipt1);
    receipt1.chain_digest_next = digest1.clone();

    let mut receipt2 = create_receipt(2, &state_hashes[2], &state_hashes[3], "80", "68", "11", "0");
    receipt2.chain_digest_prev = digest1.clone();
    let digest2 = compute_digest(&receipt2);
    receipt2.chain_digest_next = digest2.clone();

    let mut receipt3 = create_receipt(3, &state_hashes[3], &state_hashes[4], "68", "55", "12", "0");
    receipt3.chain_digest_prev = digest2.clone();
    let digest3 = compute_digest(&receipt3);
    receipt3.chain_digest_next = digest3.clone();

    // Valid fixtures
    fs::write(
        "examples/ai_demo/ai_workflow_micro_valid.json",
        serde_json::to_string_pretty(&receipt0).unwrap(),
    )
    .unwrap();
    
    write_jsonl_file(
        "vectors/semi_realistic/ai_workflow_valid.jsonl",
        &[
            receipt0.clone(),
            receipt1.clone(),
            receipt2.clone(),
            receipt3.clone(),
        ],
    );

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

    println!("Generated AI workflow fixtures");
}

fn create_receipt(
    step_index: u64,
    state_hash_prev: &str,
    state_hash_next: &str,
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
) -> MicroReceiptWire {
    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.workflow.demo".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index,
        step_type: Some("workflow".to_string()),
        signatures: None,
        state_hash_prev: state_hash_prev.to_string(),
        state_hash_next: state_hash_next.to_string(),
        chain_digest_prev: ZERO_HASH.to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
            authority: "0".to_string(),
            ..Default::default()
        },
        profile: AdmissionProfile::CoherenceOnlyV1,
        ..Default::default()
    };
    
    wire.signatures = Some(vec![signature_for(step_index)]);
    wire
}

fn compute_digest(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}

fn signature_for(step_index: u64) -> SignatureWire {
    let authority_id = format!("fixture-signer-{}", step_index % 3);
    let signing_key = fixture_signing_key(&authority_id);

    let mock_receipt = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.workflow.demo".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: ZERO_HASH.to_string(),
        step_index,
        step_type: Some("workflow".to_string()),
        signatures: None,
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: ZERO_HASH.to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire::default(),
        profile: AdmissionProfile::CoherenceOnlyV1,
    };

    let signed = sign_micro_receipt(
        mock_receipt,
        &signing_key,
        &authority_id,
        "*",
        1_700_000_000 + step_index,
        None,
        "MICRO_RECEIPT_V1",
    )
    .expect("Failed to sign receipt");

    signed.signatures.unwrap().remove(0)
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
        let mut receipt = create_receipt(
            step,
            &prev_state,
            &next_state(step + 1),
            "100",
            "99",
            "1",
            defect_for_step(step),
        );
        receipt.object_id = object_id.to_string();
        receipt.chain_digest_prev = prev_digest.clone();
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
        let mut receipt = create_receipt(
            step,
            &prev_state,
            &next_state(10_000 + step * 17 + 1),
            "100",
            posts[idx],
            spends[idx],
            defect_for_step(step),
        );
        receipt.object_id = object_id.to_string();
        receipt.step_type = Some(step_types[idx].to_string());
        receipt.chain_digest_prev = prev_digest.clone();
        receipt.chain_digest_next = compute_digest(&receipt);
        
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = receipt.state_hash_next.clone();
        out.push(receipt);
    }

    out
}
