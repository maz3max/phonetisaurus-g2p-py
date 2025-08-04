#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::exceptions::PyValueError;
use std::path::Path;

// Re-export the main types from main.rs
mod phonetisaurus;
pub use phonetisaurus::*;

#[cfg(feature = "python")]
/// Python class wrapping the Rust PhonetisaurusModel
#[pyclass]
pub struct PyPhonetisaurusModel {
    inner: PhonetisaurusModel,
}

#[cfg(feature = "python")]
/// Python class wrapping the Rust PhonetizationResult
#[pyclass]
#[derive(Clone)]
pub struct PyPhonetizationResult {
    /// Phonemes produced during phonemization
    #[pyo3(get)]
    pub phonemes: String,
    /// Negative log likelihood of phonemes, lower is better
    #[pyo3(get)]
    pub neg_log_score: f32,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyPhonetisaurusModel {
    /// Create a new phonemizer from a phonetisaurus model file path
    #[new]
    fn new(model_path: &str) -> PyResult<Self> {
        let model = PhonetisaurusModel::try_from(Path::new(model_path))
            .map_err(|e| PyValueError::new_err(format!("Failed to load model: {}", e)))?;
        
        Ok(PyPhonetisaurusModel { inner: model })
    }

    /// Create a new phonemizer from model bytes
    #[staticmethod]
    fn from_bytes(model_bytes: &[u8]) -> PyResult<Self> {
        let model = PhonetisaurusModel::try_from(model_bytes)
            .map_err(|e| PyValueError::new_err(format!("Failed to load model from bytes: {}", e)))?;
        
        Ok(PyPhonetisaurusModel { inner: model })
    }

    /// Phonemize a word using the loaded model
    fn phonemize_word(&self, word: &str) -> PyResult<PyPhonetizationResult> {
        let result = self.inner.phonemize_word(word)
            .map_err(|e| PyValueError::new_err(format!("Failed to phonemize word: {}", e)))?;
        
        Ok(PyPhonetizationResult {
            phonemes: result.phonemes,
            neg_log_score: result.neg_log_score,
        })
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl PyPhonetizationResult {
    fn __repr__(&self) -> String {
        format!("PhonetizationResult(phonemes='{}', neg_log_score={})", 
                self.phonemes, self.neg_log_score)
    }
    
    fn __str__(&self) -> String {
        self.phonemes.clone()
    }
}

#[cfg(feature = "python")]
/// Python module for phonetisaurus_g2p_py
#[pymodule]
fn phonetisaurus_g2p_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPhonetisaurusModel>()?;
    m.add_class::<PyPhonetizationResult>()?;
    Ok(())
}
