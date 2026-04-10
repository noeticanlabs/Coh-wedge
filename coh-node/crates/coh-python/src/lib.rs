use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{Decision, MicroReceiptWire};
use coh_core::verify_micro;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::{depythonize, pythonize};
use std::convert::TryFrom;

// --- Exceptions ---

pyo3::create_exception!(coh, CohError, pyo3::exceptions::PyException);
pyo3::create_exception!(coh, CohVerificationError, CohError);
pyo3::create_exception!(coh, CohMalformedError, CohError);

// --- Result Object ---

#[pyclass]
#[derive(Clone)]
pub struct CohResult {
    #[pyo3(get)]
    pub normalized: PyObject,
    #[pyo3(get)]
    pub hash: String,
}

// --- Internal Helpers ---

fn parse_polymorphic_input(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<MicroReceiptWire> {
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

// --- Public API ---

#[pyfunction]
fn normalize(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<CohResult> {
    let wire = parse_polymorphic_input(py, input)?;
    let r = coh_core::types::MicroReceipt::try_from(wire)
        .map_err(|e| CohMalformedError::new_err(format!("Semantic error: {:?}", e)))?;

    let prehash = to_prehash_view(&r);
    let canon_bytes = to_canonical_json_bytes(&prehash).unwrap();
    let digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);

    let normalized_dict = pythonize(py, &prehash)?;

    Ok(CohResult {
        normalized: normalized_dict,
        hash: digest.to_hex(),
    })
}

#[pyfunction]
fn verify(py: Python<'_>, input: Bound<'_, PyAny>) -> PyResult<()> {
    let wire = parse_polymorphic_input(py, input.clone())?;
    let result = verify_micro(wire);

    if result.decision != Decision::Accept {
        let msg = format!("Verification failed: {}", result.message);
        let err = CohVerificationError::new_err(msg);

        // Add metadata attributes to the exception instance
        let py_err_obj = err.clone_ref(py).into_pyobject(py)?;
        let _ = py_err_obj.setattr("reason", result.message);
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
    if !compare(py, a, b)? {
        let h_a = hash(py, a.clone())?;
        let h_b = hash(py, b.clone())?;
        return Err(CohVerificationError::new_err(format!(
            "Equivalence check failed: hashes differ ({} vs {})",
            h_a, h_b
        )));
    }
    Ok(())
}

#[pymodule]
fn coh(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CohResult>()?;
    m.add_function(wrap_pyfunction!(normalize, m)?)?;
    m.add_function(wrap_pyfunction!(verify, m)?)?;
    m.add_function(wrap_pyfunction!(hash, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(assert_equivalent, m)?)?;

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
