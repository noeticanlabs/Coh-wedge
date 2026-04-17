use coh_core::build_slab;
use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{Decision, MicroReceiptWire, SlabReceiptWire};
use coh_core::{verify_chain, verify_micro, verify_slab_envelope};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pythonize::{depythonize, pythonize};
use std::convert::TryFrom;

// --- Exceptions ---

pyo3::create_exception!(coh, CohError, pyo3::exceptions::PyException);
pyo3::create_exception!(coh, CohVerificationError, CohError);
pyo3::create_exception!(coh, CohMalformedError, CohError);

// --- Result Object ---

#[pyclass]
pub struct CohResult {
    #[pyo3(get)]
    pub normalized: Py<PyAny>,
    #[pyo3(get)]
    pub hash: String,
}

// --- Internal Helpers ---

fn parse_polymorphic_input(_py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<MicroReceiptWire> {
    if let Ok(json_str) = input.extract::<String>() {
        serde_json::from_str(&json_str)
            .map_err(|e| CohMalformedError::new_err(format!("JSON parse error: {}", e)))
    } else if input.is_instance_of::<PyDict>() {
        depythonize(&input).map_err(|e| {
            CohMalformedError::new_err(format!("Dictionary to Receipt conversion failed: {}", e))
        })
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Input must be a JSON string or a dictionary",
        ))
    }
}

fn parse_chain_input(_py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<Vec<MicroReceiptWire>> {
    if let Ok(jsonl_str) = input.extract::<String>() {
        jsonl_str
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str(l).map_err(|e| {
                    CohMalformedError::new_err(format!("JSONL line parse error: {}", e))
                })
            })
            .collect()
    } else if let Ok(list) = input.downcast::<PyList>() {
        list.iter()
            .map(|item| {
                depythonize(&item).map_err(|e| {
                    CohMalformedError::new_err(format!(
                        "List item to Receipt conversion failed: {}",
                        e
                    ))
                })
            })
            .collect()
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Input must be a JSONL string or a list of dictionaries",
        ))
    }
}

fn parse_slab_input(_py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<SlabReceiptWire> {
    if let Ok(json_str) = input.extract::<String>() {
        serde_json::from_str(&json_str)
            .map_err(|e| CohMalformedError::new_err(format!("Slab JSON parse error: {}", e)))
    } else if input.is_instance_of::<PyDict>() {
        depythonize(&input).map_err(|e| {
            CohMalformedError::new_err(format!("Dictionary to Slab conversion failed: {}", e))
        })
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Input must be a JSON string or a dictionary representing a Slab",
        ))
    }
}

// --- Public API ---

/// Normalize a single receipt and return its canonical digest.
#[pyfunction]
fn normalize(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<CohResult> {
    let wire = parse_polymorphic_input(py, input)?;
    let r = coh_core::types::MicroReceipt::try_from(wire)
        .map_err(|e| CohMalformedError::new_err(format!("Semantic error: {:?}", e)))?;

    let prehash = to_prehash_view(&r);
    let canon_bytes = to_canonical_json_bytes(&prehash)
        .map_err(|e| CohError::new_err(format!("Canonicalization failed: {:?}", e)))?;

    let digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);
    let normalized_dict = pythonize(py, &prehash)?;

    Ok(CohResult {
        normalized: normalized_dict.into_any().unbind(),
        hash: digest.to_hex(),
    })
}

/// Verify a single micro-receipt. Raises CohVerificationError on failure.
#[pyfunction]
fn verify(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<()> {
    let wire = parse_polymorphic_input(py, input.clone())?;
    let result = verify_micro(wire);

    if result.decision != Decision::Accept {
        let msg = format!("Verification failed: {}", result.message);
        let err = CohVerificationError::new_err(msg);

        let py_err_obj = err.value(py);
        let _ = py_err_obj.setattr("reason", result.message.clone());
        if let Some(h) = result.chain_digest_next {
            let _ = py_err_obj.setattr("hash", h);
        }
        if let Some(idx) = result.step_index {
            let _ = py_err_obj.setattr("step_index", idx);
        }

        return Err(err);
    }

    Ok(())
}

/// Verify a chain of micro-receipts (continuity and individual validity).
/// Raises CohVerificationError on failure (unified with verify() API).
#[pyfunction]
fn verify_chain_api(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<PyObject> {
    let receipts = parse_chain_input(py, input)?;
    let result = verify_chain(receipts);

    // RFAP V1.0 Compliance: Unify error behavior with verify()
    if result.decision != Decision::Accept {
        let msg = format!("Chain verification failed: {}", result.message);
        let err = CohVerificationError::new_err(msg);

        let py_err_obj = err.value(py);
        let _ = py_err_obj.setattr("reason", result.message.clone());
        if let Some(h) = result.final_chain_digest {
            let _ = py_err_obj.setattr("hash", h);
        }
        if let Some(idx) = result.failing_step_index {
            let _ = py_err_obj.setattr("step_index", idx);
        }

        return Err(err);
    }

    pythonize(py, &result)
        .map(|b| b.unbind())
        .map_err(|e| CohError::new_err(format!("Result conversion failed: {}", e)))
}

/// Build a Slab (macro receipt) from a chain of micro-receipts.
#[pyfunction]
fn build_slab_api(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<PyObject> {
    let receipts = parse_chain_input(py, input)?;
    let result = build_slab(receipts);
    pythonize(py, &result)
        .map(|b| b.unbind())
        .map_err(|e| CohError::new_err(format!("Result conversion failed: {}", e)))
}

/// Verify a Slab receipt.
#[pyfunction]
fn verify_slab_api(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<PyObject> {
    let wire = parse_slab_input(py, input)?;
    let result = verify_slab_envelope(wire);
    pythonize(py, &result)
        .map(|b| b.unbind())
        .map_err(|e| CohError::new_err(format!("Result conversion failed: {}", e)))
}

#[pyfunction]
fn hash(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<String> {
    let res = normalize(py, input)?;
    Ok(res.hash)
}

#[pyfunction]
fn compare(py: Python<'_>, a: Bound<'_, PyAny>, b: Bound<'_, PyAny>) -> PyResult<bool> {
    let res_a = normalize(py, a)?;
    let res_b = normalize(py, b)?;
    Ok(res_a.hash == res_b.hash)
}

#[pyfunction]
fn assert_equivalent(py: Python<'_>, a: Bound<'_, PyAny>, b: Bound<'_, PyAny>) -> PyResult<()> {
    if !compare(py, a.clone(), b.clone())? {
        let h_a = hash(py, a)?;
        let h_b = hash(py, b)?;
        return Err(CohVerificationError::new_err(format!(
            "Equivalence check failed: hashes differ ({} vs {})",
            h_a, h_b
        )));
    }
    Ok(())
}

#[pymodule(name = "coh")]
fn coh(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CohResult>()?;
    m.add_function(wrap_pyfunction!(normalize, m)?)?;
    m.add_function(wrap_pyfunction!(verify, m)?)?;
    m.add_function(wrap_pyfunction!(hash, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(assert_equivalent, m)?)?;

    // V1 API Expansion
    m.add_function(wrap_pyfunction!(verify_chain_api, m)?)?;
    m.add_function(wrap_pyfunction!(build_slab_api, m)?)?;
    m.add_function(wrap_pyfunction!(verify_slab_api, m)?)?;

    // Create a mapping for consistent naming if desired
    m.add("verify_chain", m.getattr("verify_chain_api")?)?;
    m.add("build_slab", m.getattr("build_slab_api")?)?;
    m.add("verify_slab", m.getattr("verify_slab_api")?)?;

    // Add exceptions to the module
    let py = m.py();
    m.add("CohError", py.get_type::<CohError>())?;
    m.add(
        "CohVerificationError",
        py.get_type::<CohVerificationError>(),
    )?;
    m.add("CohMalformedError", py.get_type::<CohMalformedError>())?;

    Ok(())
}
