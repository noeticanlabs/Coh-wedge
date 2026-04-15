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
    client
        .post(url)
        .json(payload)
        .send()?
        .error_for_status()?
        .json()
}
