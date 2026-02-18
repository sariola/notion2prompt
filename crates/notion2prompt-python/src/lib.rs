//! Python bindings for notion2prompt via PyO3.
//!
//! Exposes the three-stage pipeline (fetch → compose → deliver) as Python
//! async/sync functions, plus a high-level one-shot `fetch_and_render`.

use pyo3::prelude::*;

mod pipeline;
mod types;

/// The main Python module: `notion2prompt._notion2prompt`
#[pymodule]
fn _notion2prompt(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // High-level async pipeline
    m.add_function(wrap_pyfunction!(pipeline::fetch_and_render, m)?)?;
    m.add_function(wrap_pyfunction!(pipeline::fetch_content, m)?)?;
    m.add_function(wrap_pyfunction!(pipeline::render_content, m)?)?;

    // Types
    m.add_class::<types::PyPipelineConfig>()?;
    m.add_class::<types::PyNotionContent>()?;

    Ok(())
}
