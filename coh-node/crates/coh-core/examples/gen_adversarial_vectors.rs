//! Generate adversarial test vectors for the Coh Validator.
//! Covering: broken digests, state discontinuities, policy violations, numeric malformations.

use coh_core::canon::*;
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MetricsWire, MicroReceiptWire};
use std::fs::File;
use std::io::Write;
use std::path::Path;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new("vectors/adversarial");
    std::fs::create_dir_all(base_dir)?;

    // 1. Valid Chain (Control)
    gen_chain(
        base_dir.join("valid_chain.jsonl"),
        "agent.valid",
        |i, _| i, // No tampering
    )?;

    // 2. Reject Chain Digest (Tampered link at step 1)
    gen_chain(
        base_dir.join("reject_chain_digest.jsonl"),
        "agent.tamper.digest",
        |i, w| {
            if i == 1 {
                w.chain_digest_next = "f".repeat(64);
            }
            i
        },
    )?;

    // 3. Reject State Link (Discontinuous state at step 2)
    gen_chain(
        base_dir.join("reject_state_link.jsonl"),
        "agent.tamper.state",
        |i, w| {
            if i == 2 {
                w.state_hash_prev = "e".repeat(64);
            }
            i
        },
    )?;

    // 4. Reject Policy Violation (Spend > v_pre at step 1)
    gen_chain(
        base_dir.join("reject_policy_violation.jsonl"),
        "agent.tamper.policy",
        |i, w| {
            if i == 1 {
                w.metrics.spend = "9999999".to_string(); // Violation
            }
            i
        },
    )?;

    // 5. Reject Schema (Invalid version)
    gen_chain(
        base_dir.join("reject_schema.jsonl"),
        "agent.tamper.schema",
        |i, w| {
            if i == 0 {
                w.version = "9.9.9".to_string();
            }
            i
        },
    )?;

    // 6. Reject Numeric Parse
    gen_chain(
        base_dir.join("reject_numeric_parse.jsonl"),
        "agent.tamper.numeric",
        |i, w| {
            if i == 1 {
                w.metrics.v_pre = "not-a-number".to_string();
            }
            i
        },
    )?;

    // 7. Reject Overflow (Max u128 + 1)
    gen_chain(
        base_dir.join("reject_overflow.jsonl"),
        "agent.tamper.overflow",
        |i, w| {
            if i == 0 {
                w.metrics.v_pre = "340282366920938463463374607431768211455".to_string(); // u128::MAX
                w.metrics.defect = "1".to_string(); // Triggers overflow in v_pre + defect
            }
            i
        },
    )?;

    println!("Generated adversarial vectors in: {:?}", base_dir);
    Ok(())
}

fn gen_chain<P: AsRef<Path>, F>(path: P, object_id: &str, tamper: F) -> std::io::Result<()>
where
    F: Fn(u64, &mut MicroReceiptWire) -> u64,
{
    let mut file = File::create(path)?;
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = "a".repeat(64);

    for i in 0..3 {
        let next_state = format!("{:064x}", i + 1);
        let mut wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: object_id.to_string(),
            canon_profile_hash: VALID_PROFILE.to_string(),
            policy_hash: "0".repeat(64),
            step_index: i,
            state_hash_prev: prev_state.clone(),
            state_hash_next: next_state.clone(),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: "50".to_string(),
                spend: "50".to_string(),
                defect: "0".to_string(),
            },
        };

        // Apply tampering *conditionally* on whether we need a valid digest first
        // If we want a valid-looking receipt that triggers a LATER link failure, we seal it.
        // If we want a receipt that is internally malformed, we can skip sealing or seal then tamper.

        let digest = compute_digest(&wire);
        wire.chain_digest_next = digest.clone();

        // Apply tampering
        tamper(i, &mut wire);

        // Update loop state (using the version BEFORE tampering for continuity demos)
        prev_digest = digest;
        prev_state = next_state;

        serde_json::to_writer(&file, &wire)?;
        file.write_all(b"\n")?;
    }

    Ok(())
}

fn compute_digest(wire: &MicroReceiptWire) -> String {
    use std::convert::TryFrom;
    // We only unwrap here because we are generating vectors from a known-good starting point.
    // If this fails, the generator logic is broken.
    let r = coh_core::types::MicroReceipt::try_from(wire.clone())
        .expect("Failed to create MicroReceipt in compute_digest");
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(r.chain_digest_prev, &bytes).to_hex()
}
