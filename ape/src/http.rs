use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ExecuteVerifiedRequest {
    pub receipt: coh_core::types::MicroReceiptWire,
    pub action: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SidecarResponse {
    pub request_id: String,
    pub coh_version: String,
    pub status: String,
    pub data: Option<Value>,
    pub error: Option<Value>,
}

pub fn execute_verified(
    base_url: &str,
    payload: &ExecuteVerifiedRequest,
) -> Result<SidecarResponse, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}/v1/execute-verified", base_url.trim_end_matches('/'));
    let response = client
        .post(url)
        .json(payload)
        .send()?
        .error_for_status()?
        .json();

    // Log the receipt for debugging
    println!(
        "[execute_verified] Sent receipt: {:?}",
        payload.receipt.step_index
    );

    response
}

/// Save valid receipts to JSONL for dashboard
pub fn save_valid_receipts_to_jsonl(
    receipts: &[coh_core::types::MicroReceiptWire],
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create(path)?;
    for receipt in receipts {
        let json = serde_json::to_string(receipt)?;
        writeln!(file, "{}", json)?;
    }
    println!(
        "[execute_verified] Saved {} receipts to {:?}",
        receipts.len(),
        path
    );
    Ok(())
}
