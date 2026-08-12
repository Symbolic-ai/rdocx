use oxml_py_support::{ContentPath, PathSeg};
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySlice};

use crate::normalize_index;
use crate::presentation::PyPresentation;
use crate::shape::{shape_mut_at, shape_ref_at};
use crate::validate_path;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyTable>()?;
    module.add_class::<PyColumn>()?;
    module.add_class::<PyColumnCollection>()?;
    module.add_class::<PyCell>()?;
    Ok(())
}

fn row_index(path: &ContentPath) -> Option<usize> {
    path.segs.iter().find_map(|segment| match segment {
        PathSeg::Row(index) => Some(*index),
        _ => None,
    })
}

fn cell_index(path: &ContentPath) -> Option<usize> {
    path.segs.iter().rev().find_map(|segment| match segment {
        PathSeg::Cell(index) => Some(*index),
        _ => None,
    })
}

#[pyclass(name = "Table")]
pub struct PyTable {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyTable {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn dimensions(&self, py: Python<'_>) -> PyResult<(usize, usize)> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "table",
            ".table",
        )?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.table())
            .map(|table| (table.row_count(), table.column_count()))
            .ok_or_else(|| PyValueError::new_err("shape has no table"))
    }
}

#[pymethods]
impl PyTable {
    #[getter]
    fn columns(&self, py: Python<'_>) -> PyResult<Py<PyColumnCollection>> {
        self.dimensions(py)?;
        Py::new(
            py,
            PyColumnCollection {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
            },
        )
    }

    fn cell(&self, py: Python<'_>, row: isize, col: isize) -> PyResult<Py<PyCell>> {
        let (rows, columns) = self.dimensions(py)?;
        let row = normalize_index(row, rows, "row")?;
        let column = normalize_index(col, columns, "cell")?;
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Row(row));
        segments.push(PathSeg::Cell(column));
        let path = self.presentation.borrow(py).revisions.capture(segments);
        Py::new(
            py,
            PyCell {
                presentation: self.presentation.clone_ref(py),
                path,
            },
        )
    }
}

#[pyclass(name = "ColumnCollection")]
pub struct PyColumnCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyColumnCollection {
    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "column collection",
            ".table.columns",
        )?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.table())
            .map(|table| table.column_count())
            .ok_or_else(|| PyValueError::new_err("shape has no table"))
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyColumn>> {
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Cell(index));
        let path = self.presentation.borrow(py).revisions.capture(segments);
        Py::new(
            py,
            PyColumn {
                presentation: self.presentation.clone_ref(py),
                path,
            },
        )
    }
}

#[pymethods]
impl PyColumnCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.len(py)?;
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self
                .item(py, normalize_index(index, len, "column")?)?
                .into_any());
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
            "column indices must be integers or slices",
        ))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyColumnIterator>> {
        self.len(py)?;
        Py::new(
            py,
            PyColumnIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyColumnIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PyColumnIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyColumn>>> {
        let collection = PyColumnCollection {
            presentation: self.presentation.clone_ref(py),
            path: self.path.clone(),
        };
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Column")]
pub struct PyColumn {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

#[pymethods]
impl PyColumn {
    #[getter]
    fn width(&self, py: Python<'_>) -> PyResult<i64> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "column", "")?;
        let column = cell_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("column index is missing"))?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.table())
            .and_then(|table| table.column_width(column))
            .map(|width| width.0)
            .ok_or_else(|| PyIndexError::new_err("column index out of range"))
    }

    #[setter]
    fn set_width(&self, py: Python<'_>, width: i64) -> PyResult<()> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "column", "")?;
        let column = cell_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("column index is missing"))?;
        shape_mut_at(&mut self.presentation.borrow_mut(py).inner, &self.path)
            .and_then(rpptx::ShapeMut::into_table_mut)
            .ok_or_else(|| PyValueError::new_err("shape has no table"))?
            .set_column_width(column, rpptx::Emu(width))
            .map_err(|error| crate::rpptx_to_pyerr(py, error))
    }
}

#[pyclass(name = "Cell")]
pub struct PyCell {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

#[pymethods]
impl PyCell {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "cell", "")?;
        let row =
            row_index(&self.path).ok_or_else(|| PyIndexError::new_err("row index is missing"))?;
        let cell =
            cell_index(&self.path).ok_or_else(|| PyIndexError::new_err("cell index is missing"))?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.table())
            .and_then(|table| table.cell(row, cell))
            .map(|cell| cell.text())
            .ok_or_else(|| PyIndexError::new_err("cell index out of range"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "cell", "")?;
        let row =
            row_index(&self.path).ok_or_else(|| PyIndexError::new_err("row index is missing"))?;
        let cell =
            cell_index(&self.path).ok_or_else(|| PyIndexError::new_err("cell index is missing"))?;
        shape_mut_at(&mut self.presentation.borrow_mut(py).inner, &self.path)
            .and_then(rpptx::ShapeMut::into_table_mut)
            .and_then(|table| table.into_cell_mut(row, cell))
            .map(|mut cell| cell.set_text(value))
            .ok_or_else(|| PyIndexError::new_err("cell index out of range"))
    }
}
