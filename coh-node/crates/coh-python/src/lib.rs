use pyo3::prelude::*;
use coh_core::verify_micro;
use coh_core::types::{MicroReceiptWire};
use coh_core::canon::{to_prehash_view, to_canonical_json_bytes};
use coh_core::hash::compute_chain_digest;
use std::convert::TryFrom;

#[pyfunction]
fn normalize(json_str: &str) -> PyResult<String> {
    let wire: MicroReceiptWire = serde_json::from_str(json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("JSON parse error: {}", e)))?;
    
    let r = coh_core::types::MicroReceipt::try_from(wire)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Semantic error: {:?}", e)))?;
        
    let prehash = to_prehash_view(&r);
    let canon_bytes = to_canonical_json_bytes(&prehash)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Canonicalization error: {:?}", e)))?;
        
    Ok(String::from_utf8(canon_bytes).unwrap())
}

#[pyfunction]
fn compare(a_json: &str, b_json: &str) -> PyResult<bool> {
    let norm_a = normalize(a_json)?;
    let norm_b = normalize(b_json)?;
    Ok(norm_a == norm_b)
}

#[pyfunction]
fn calculate_hash(json_str: &str) -> PyResult<String> {
    let wire: MicroReceiptWire = serde_json::from_str(json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("JSON parse error: {}", e)))?;
    
    let r = coh_core::types::MicroReceipt::try_from(wire.clone())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Semantic error: {:?}", e)))?;
        
    let prehash = to_prehash_view(&r);
    let canon_bytes = to_canonical_json_bytes(&prehash).unwrap();
    let digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);
    
    Ok(digest.to_hex())
}

#[pyfunction]
fn verify(json_str: &str) -> PyResult<String> {
    let wire: MicroReceiptWire = serde_json::from_str(json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("JSON parse error: {}", e)))?;
    
    let result = verify_micro(wire);
    let result_json = serde_json::to_string(&result).unwrap();
    
    Ok(result_json)
}

#[pymodule]
fn coh(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_hash, m)?)?;
    m.add_function(wrap_pyfunction!(verify, m)?)?;
    Ok(())
}
