//! Python bindings for the rdocx facade.

mod document;
mod paragraph;
mod run;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use oxml_py_support::StaleElementError;

use document::PyDocument;
use paragraph::{PyParagraph, PyParagraphCollection};
use run::{PyRun, PyRunCollection};

pub(crate) fn stale_to_pyerr(py: Python<'_>, error: StaleElementError) -> PyErr {
    let exception_type = py
        .import("rdocx")
        .and_then(|module| module.getattr("StaleElementError"))
        .and_then(|class| class.cast_into::<PyType>().map_err(Into::into));

    match exception_type {
        Ok(class) => PyErr::from_type(class, (error.to_string(),)),
        Err(_) => PyRuntimeError::new_err(error.to_string()),
    }
}

#[pymodule]
fn _rdocx(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyDocument>()?;
    module.add_class::<PyParagraph>()?;
    module.add_class::<PyParagraphCollection>()?;
    module.add_class::<PyRun>()?;
    module.add_class::<PyRunCollection>()?;
    Ok(())
}
