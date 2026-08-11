use oxml_py_support::{ContentPath, PathSeg};
use pyo3::exceptions::{PyIndexError, PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySlice};
use smallvec::smallvec;

use crate::document::PyDocument;
use crate::stale_to_pyerr;

fn path_indices(path: &ContentPath) -> PyResult<(usize, usize)> {
    let paragraph = path.segs.iter().find_map(|segment| match segment {
        PathSeg::Para(index) => Some(*index),
        _ => None,
    });
    let run = path.segs.iter().find_map(|segment| match segment {
        PathSeg::Run(index) => Some(*index),
        _ => None,
    });
    match (paragraph, run) {
        (Some(paragraph), Some(run)) => Ok((paragraph, run)),
        _ => Err(PyRuntimeError::new_err("run path is incomplete")),
    }
}

fn paragraph_index(path: &ContentPath) -> PyResult<usize> {
    path.segs
        .iter()
        .find_map(|segment| match segment {
            PathSeg::Para(index) => Some(*index),
            _ => None,
        })
        .ok_or_else(|| PyRuntimeError::new_err("run collection path has no paragraph index"))
}

fn normalize_index(index: isize, len: usize) -> PyResult<usize> {
    let normalized = if index < 0 {
        len as isize + index
    } else {
        index
    };
    if normalized < 0 || normalized >= len as isize {
        return Err(PyIndexError::new_err("run index out of range"));
    }
    Ok(normalized as usize)
}

#[pyclass(name = "Run")]
pub struct PyRun {
    document: Py<PyDocument>,
    path: ContentPath,
}

impl PyRun {
    pub(crate) fn new(document: Py<PyDocument>, path: ContentPath) -> Self {
        Self { document, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<(usize, usize)> {
        let document = self.document.borrow(py);
        self.path
            .validate_revision(
                document.revisions.current(),
                "run",
                "Re-fetch it with paragraph.runs[i].",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        path_indices(&self.path)
    }
}

#[pymethods]
impl PyRun {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        let (paragraph_index, run_index) = self.validate(py)?;
        let document = self.document.borrow(py);
        let paragraph = document
            .inner
            .paragraph(paragraph_index)
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        paragraph
            .run(run_index)
            .map(|run| run.text())
            .ok_or_else(|| PyIndexError::new_err("run index out of range"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        let (paragraph_index, run_index) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut paragraph = document
            .inner
            .paragraph_mut(paragraph_index)
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        paragraph
            .run_mut(run_index)
            .ok_or_else(|| PyIndexError::new_err("run index out of range"))?
            .set_text(text);
        Ok(())
    }
}

#[pyclass(name = "RunCollection")]
pub struct PyRunCollection {
    document: Py<PyDocument>,
    paragraph_path: ContentPath,
}

impl PyRunCollection {
    pub(crate) fn new(document: Py<PyDocument>, paragraph_path: ContentPath) -> Self {
        Self {
            document,
            paragraph_path,
        }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        let document = self.document.borrow(py);
        self.paragraph_path
            .validate_revision(
                document.revisions.current(),
                "run collection",
                "Re-fetch it with paragraph.runs.",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        paragraph_index(&self.paragraph_path)
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let paragraph_index = self.validate(py)?;
        self.document
            .borrow(py)
            .inner
            .paragraph(paragraph_index)
            .map(|paragraph| paragraph.run_count())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyRun>> {
        let paragraph_index = self.validate(py)?;
        let path = {
            let document = self.document.borrow(py);
            document.revisions.capture(smallvec![
                PathSeg::Body(0),
                PathSeg::Para(paragraph_index),
                PathSeg::Run(index),
            ])
        };
        Py::new(py, PyRun::new(self.document.clone_ref(py), path))
    }
}

#[pymethods]
impl PyRunCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.len(py)?;
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self.item(py, normalize_index(index, len)?)?.into_any());
        }
        if key.is_instance_of::<PySlice>() {
            let (start, stop, step): (isize, isize, isize) =
                key.call_method1("indices", (len,))?.extract()?;
            let items = PyList::empty(py);
            let mut index = start;
            while if step > 0 { index < stop } else { index > stop } {
                items.append(self.item(py, index as usize)?)?;
                index += step;
            }
            return Ok(items.into_any().unbind());
        }
        Err(PyTypeError::new_err(
            "run indices must be integers or slices",
        ))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyRunIterator>> {
        self.validate(py)?;
        Py::new(
            py,
            PyRunIterator {
                document: self.document.clone_ref(py),
                paragraph_path: self.paragraph_path.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyRunIterator {
    document: Py<PyDocument>,
    paragraph_path: ContentPath,
    index: usize,
}

#[pymethods]
impl PyRunIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyRun>>> {
        let collection =
            PyRunCollection::new(self.document.clone_ref(py), self.paragraph_path.clone());
        let len = collection.len(py)?;
        if self.index >= len {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}
