//! Generate traces for the "Hallucination Breach" demo.
//!
//! audit_honest.jsonl: 50 steps of lawful accounting.
//! audit_hallucinated.jsonl: Step 25 introduces a "value print" (V_post too high).

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire};
use std::convert::TryFrom;
use std::fs::File;
use std::io::Write;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn main() {
    let mut honest_file = File::create("examples/audit_honest.jsonl").unwrap();
    let mut hallucinated_file = File::create("examples/audit_hallucinated.jsonl").unwrap();

    let mut prev_digest_h = "0".repeat(64);
    let mut prev_state_h = format!("{:064x}", 1u64);
    let mut prev_digest_c = "0".repeat(64);
    let mut prev_state_c = format!("{:064x}", 1u64);

    for i in 0..50 {
        let next_state = format!("{:064x}", (i + 2) as u64);

        // Honest step logic
        let v_pre = 1000 - (i * 10);
        let spend = 10;
        let defect = 0;
        let v_post = v_pre - spend + defect;

        let mut receipt = create_receipt(
            i as u64,
            &prev_state_h,
            &next_state,
            &v_pre.to_string(),
            &v_post.to_string(),
            &spend.to_string(),
            &defect.to_string(),
            &prev_digest_h,
        );
        receipt.chain_digest_next = seal(&receipt);
        writeln!(honest_file, "{}", serde_json::to_string(&receipt).unwrap()).unwrap();
        prev_digest_h = receipt.chain_digest_next.clone();
        prev_state_h = next_state.clone();

        // Hallucinated trace logic
        let h_receipt = if i == 25 {
            let corrupt_v_post = v_post + 500;
            let mut r = create_receipt(
                i as u64,
                &prev_state_c,
                &next_state,
                &v_pre.to_string(),
                &corrupt_v_post.to_string(),
                &spend.to_string(),
                &defect.to_string(),
                &prev_digest_c,
            );
            r.chain_digest_next = seal(&r);
            r
        } else if i > 25 {
            let corrupt_offset = 500;
            let v_pre_corrupt = v_pre + corrupt_offset;
            let v_post_corrupt = v_post + corrupt_offset;
            let mut r = create_receipt(
                i as u64,
                &prev_state_c,
                &next_state,
                &v_pre_corrupt.to_string(),
                &v_post_corrupt.to_string(),
                &spend.to_string(),
                &defect.to_string(),
                &prev_digest_c,
            );
            r.chain_digest_next = seal(&r);
            r
        } else {
            receipt.clone()
        };

        writeln!(
            hallucinated_file,
            "{}",
            serde_json::to_string(&h_receipt).unwrap()
        )
        .unwrap();
        prev_digest_c = h_receipt.chain_digest_next.clone();
        prev_state_c = next_state;
    }

    println!("Generated demo fixtures in examples/");
}

#[allow(clippy::too_many_arguments)]
fn create_receipt(
    step: u64,
    prev_state: &str,
    next_state: &str,
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
    prev_digest: &str,
) -> MicroReceiptWire {
    MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.audit.demo".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: None,
        signatures: None,
        state_hash_prev: prev_state.to_string(),
        state_hash_next: next_state.to_string(),
        chain_digest_prev: prev_digest.to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
            authority: "0".to_string(),
        },
    }
}

fn seal(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}
