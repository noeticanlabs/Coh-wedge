use crate::fixtures::FixtureError;
use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MicroReceipt, MicroReceiptWire};
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_ai_demo_micro() -> Result<MicroReceiptWire, FixtureError> {
    let path = repo_path(&[
        "coh-node",
        "examples",
        "ai_demo",
        "ai_workflow_micro_valid.json",
    ]);
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn load_ai_demo_chain() -> Result<Vec<MicroReceiptWire>, FixtureError> {
    let path = repo_path(&[
        "coh-node",
        "examples",
        "ai_demo",
        "ai_workflow_chain_valid.jsonl",
    ]);
    load_jsonl_micro(path)
}

pub fn load_dashboard_valid_chain() -> Result<Vec<MicroReceiptWire>, FixtureError> {
    let path = repo_path(&["coh-dashboard", "public", "demo", "valid_chain.jsonl"]);
    load_jsonl_micro(path)
}

pub fn generate_runtime_ai_micro() -> Result<MicroReceiptWire, FixtureError> {
    let mut chain = generate_runtime_ai_chain(1)?;
    Ok(chain.remove(0))
}

pub fn generate_runtime_ai_chain(steps: usize) -> Result<Vec<MicroReceiptWire>, FixtureError> {
    let patterns = [
        ("100", "88", "12", "0", "0"),
        ("88", "80", "7", "1", "0"),
        ("80", "68", "11", "0", "0"),
        ("68", "55", "12", "0", "0"),
    ];

    let mut chain = Vec::with_capacity(steps);
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = format!("{:064x}", 1u64);

    for i in 0..steps {
        let next_state = format!("{:064x}", (i + 2) as u64);
        let (v_pre, v_post, spend, defect, authority) = patterns[i % patterns.len()];
        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: format!("agent.workflow.demo.{}", i / 4),
            canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
                .to_string(),
            policy_hash: "0".repeat(64),
            step_index: i as u64,
            step_type: Some("realdata".to_string()),
            signatures: Some(vec![]),
            state_hash_prev: prev_state.clone(),
            state_hash_next: next_state.clone(),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: "0".repeat(64),
            metrics: coh_core::types::MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: defect.to_string(),
                authority: authority.to_string(),
            },
        };
        receipt.chain_digest_next = seal(&receipt)?;
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = next_state;
        chain.push(receipt);
    }

    Ok(chain)
}

pub fn ensure_output_dir() -> Result<PathBuf, FixtureError> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn write_output_json<T: serde::Serialize>(
    name: &str,
    value: &T,
) -> Result<PathBuf, FixtureError> {
    let dir = ensure_output_dir()?;
    let path = dir.join(name);
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(&path, bytes)?;
    Ok(path)
}

fn repo_path(parts: &[&str]) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    for part in parts {
        p.push(part);
    }
    p
}

fn load_jsonl_micro(path: impl AsRef<Path>) -> Result<Vec<MicroReceiptWire>, FixtureError> {
    let content = fs::read_to_string(path)?;
    let mut receipts = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        receipts.push(serde_json::from_str(line)?);
    }
    Ok(receipts)
}

fn seal(receipt: &MicroReceiptWire) -> Result<String, FixtureError> {
    let runtime = MicroReceipt::try_from(receipt.clone())
        .map_err(|e| FixtureError::NotFound(format!("runtime conversion failed: {:?}", e)))?;
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash)
        .map_err(|e| FixtureError::NotFound(format!("canonicalization failed: {:?}", e)))?;
    Ok(compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex())
}
