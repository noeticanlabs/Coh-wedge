use crate::canon::{to_canonical_json_bytes, to_prehash_view, EXPECTED_CANON_PROFILE_HASH};
use crate::hash::compute_chain_digest;
use crate::types::{MetricsWire, MicroReceipt, MicroReceiptWire, SignatureWire};
use crate::verify_micro::verify_micro;
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

use super::generator::ExtValidationReport;

#[derive(thiserror::Error, Debug)]
pub enum IngestError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("schema: {0}")]
    Schema(String),
}

fn reseal_in_place(wire: &mut MicroReceiptWire) -> Result<(), crate::types::RejectCode> {
    let runtime = MicroReceipt::try_from(wire.clone())?;
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash)?;
    wire.chain_digest_next = compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex();
    Ok(())
}

fn hex64_of(v: u128) -> String {
    format!("{:064x}", v)
}

fn signature_for(step_index: u64, signer: &str) -> SignatureWire {
    SignatureWire {
        signature: format!("sig-log-{:016}", step_index),
        signer: signer.to_string(),
        public_key: None,
        timestamp: 1_700_000_000 + step_index,
    }
}

pub fn ingest_api_jsonl(path: impl AsRef<Path>) -> Result<Vec<MicroReceiptWire>, IngestError> {
    ingest_with_mapper(path, |i, prev_state, prev_digest, v| {
        let tokens_used = v
            .get("tokens_used")
            .or_else(|| v.get("cost_tokens"))
            .and_then(|x| x.as_u64())
            .unwrap_or(1) as u128;
        let success = v
            .get("success")
            .or_else(|| v.get("ok"))
            .and_then(|x| x.as_bool())
            .unwrap_or(true);
        let object_id = v
            .get("object_id")
            .or_else(|| v.get("service"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("api.call.{}", i));

        let v_pre = u128::from_str_radix(&prev_state[32..], 16).unwrap_or(1_000);
        let spend = tokens_used.min(v_pre);
        let v_post = v_pre - spend;

        let mut wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id,
            public_key: None,
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: "0".repeat(64),
            step_index: i as u64,
            step_type: Some("api_log".to_string()),
            signatures: Some(vec![signature_for(i as u64, "api-logger")]),
            state_hash_prev: prev_state.to_string(),
            state_hash_next: hex64_of(v_post),
            chain_digest_prev: prev_digest.to_string(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: "0".to_string(),
                authority: if success { "1" } else { "0" }.to_string(),
            },
        };
        reseal_in_place(&mut wire).ok();
        Ok(wire)
    })
}

pub fn ingest_pipeline_jsonl(path: impl AsRef<Path>) -> Result<Vec<MicroReceiptWire>, IngestError> {
    ingest_with_mapper(path, |i, prev_state, prev_digest, v| {
        let cpu_ms = v.get("cpu_ms").and_then(|x| x.as_u64()).unwrap_or(50) as u128;
        let bytes = v.get("bytes").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
        let success = v
            .get("success")
            .or_else(|| v.get("status"))
            .and_then(|x| x.as_bool())
            .unwrap_or(true);
        let object_id = v
            .get("stage")
            .or_else(|| v.get("action"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("pipeline.stage.{}", i));

        let v_pre = u128::from_str_radix(&prev_state[32..], 16).unwrap_or(10_000);
        // Convert CPU ms + a scaled bytes to a spend unit within v_pre
        let spend_units = cpu_ms + (bytes / 1024);
        let spend = spend_units.min(v_pre);
        let v_post = v_pre - spend;

        let mut wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id,
            public_key: None,
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: "0".repeat(64),
            step_index: i as u64,
            step_type: Some("pipeline_log".to_string()),
            signatures: Some(vec![signature_for(i as u64, "pipeline-logger")]),
            state_hash_prev: prev_state.to_string(),
            state_hash_next: hex64_of(v_post),
            chain_digest_prev: prev_digest.to_string(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: if success { "0" } else { "1" }.to_string(),
                authority: if success { "1" } else { "0" }.to_string(),
            },
        };
        reseal_in_place(&mut wire).ok();
        Ok(wire)
    })
}

pub fn ingest_cicd_jsonl(path: impl AsRef<Path>) -> Result<Vec<MicroReceiptWire>, IngestError> {
    ingest_with_mapper(path, |i, prev_state, prev_digest, v| {
        let duration_s = v
            .get("duration_s")
            .or_else(|| v.get("duration"))
            .and_then(|x| x.as_u64())
            .unwrap_or(30) as u128;
        let retries = v.get("retries").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
        let status_str = v
            .get("status")
            .and_then(|x| x.as_str())
            .unwrap_or("success");
        let success = matches!(status_str, "success" | "ok" | "passed");
        let object_id = v
            .get("job")
            .or_else(|| v.get("step"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("cicd.job.{}", i));

        let v_pre = u128::from_str_radix(&prev_state[32..], 16).unwrap_or(3_600);
        // Spend: wall time + retry penalty
        let spend = (duration_s + retries * 10).min(v_pre);
        let v_post = v_pre - spend;

        let mut wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id,
            public_key: None,
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: "0".repeat(64),
            step_index: i as u64,
            step_type: Some("cicd_log".to_string()),
            signatures: Some(vec![signature_for(i as u64, "cicd-logger")]),
            state_hash_prev: prev_state.to_string(),
            state_hash_next: hex64_of(v_post),
            chain_digest_prev: prev_digest.to_string(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: if success { "0" } else { "1" }.to_string(),
                authority: if success { "1" } else { "0" }.to_string(),
            },
        };
        reseal_in_place(&mut wire).ok();
        Ok(wire)
    })
}

pub fn run_logs_validation(receipts: Vec<MicroReceiptWire>) -> ExtValidationReport {
    let total = receipts.len();
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    for r in receipts {
        let res = verify_micro(r);
        match res.decision {
            crate::types::Decision::Accept => accepted += 1,
            _ => rejected += 1,
        }
    }
    ExtValidationReport {
        total_valid: total,
        total_invalid: 0,
        accepted_valid: accepted,
        rejected_valid: rejected,
        accepted_invalid: 0,
        rejected_invalid: 0,
    }
}

fn ingest_with_mapper<F>(
    path: impl AsRef<Path>,
    mut map: F,
) -> Result<Vec<MicroReceiptWire>, IngestError>
where
    F: FnMut(usize, &str, &str, Value) -> Result<MicroReceiptWire, IngestError>,
{
    let content = fs::read_to_string(path)?;
    let mut out = Vec::new();
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = "0".repeat(64);
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let v: Value = serde_json::from_str(line)?;
        let r = map(i, &prev_state, &prev_digest, v)?;
        // Update prev_* with what builder produced
        prev_digest = r.chain_digest_next.clone();
        prev_state = r.state_hash_next.clone();
        out.push(r);
    }
    Ok(out)
}
