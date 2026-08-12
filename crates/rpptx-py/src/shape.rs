use std::path::PathBuf;

use oxml_py_support::{ContentPath, PathSeg};
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySlice};

use crate::normalize_index;
use crate::presentation::PyPresentation;
use crate::rpptx_to_pyerr;
use crate::table::PyTable;
use crate::text::PyTextFrame;
use crate::validate_path;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyShape>()?;
    module.add_class::<PyShapeCollection>()?;
    module.add_class::<PyPlaceholderCollection>()?;
    Ok(())
}

fn slide_index(path: &ContentPath) -> PyResult<usize> {
    path.segs
        .iter()
        .find_map(|segment| match segment {
            PathSeg::Slide(index) => Some(*index),
            _ => None,
        })
        .ok_or_else(|| PyIndexError::new_err("slide index is missing"))
}

fn shape_indices(path: &ContentPath) -> impl Iterator<Item = usize> + '_ {
    path.segs.iter().filter_map(|segment| match segment {
        PathSeg::Shape(index) => Some(*index),
        _ => None,
    })
}

pub(crate) fn shape_ref_at<'a>(
    presentation: &'a rpptx::Presentation,
    path: &ContentPath,
) -> Option<rpptx::ShapeRef<'a>> {
    let slide = presentation.slide(slide_index(path).ok()?)?;
    let mut indices = shape_indices(path);
    let mut shape = slide.shape(indices.next()?)?;
    for index in indices {
        shape = shape.child(index)?;
    }
    Some(shape)
}

pub(crate) fn shape_mut_at<'a>(
    presentation: &'a mut rpptx::Presentation,
    path: &ContentPath,
) -> Option<rpptx::ShapeMut<'a>> {
    let slide = presentation.slide_mut(slide_index(path).ok()?)?;
    let mut indices = shape_indices(path);
    let mut shape = slide.into_shape_mut(indices.next()?)?;
    for index in indices {
        shape = shape.into_child_mut(index)?;
    }
    Some(shape)
}

#[pyclass(name = "Shape")]
pub struct PyShape {
    pub(crate) presentation: Py<PyPresentation>,
    pub(crate) path: ContentPath,
}

impl PyShape {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<()> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "shape", "")
    }
}

#[pymethods]
impl PyShape {
    #[getter]
    fn shapes(&self, py: Python<'_>) -> PyResult<Py<PyShapeCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyShapeCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        shape_ref_at(&presentation.inner, &self.path)
            .and_then(|shape| shape.text())
            .ok_or_else(|| PyValueError::new_err("shape has no text"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        shape_mut_at(&mut presentation.inner, &self.path)
            .ok_or_else(|| PyIndexError::new_err("shape index out of range"))?
            .set_text(text)
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        presentation.revisions.bump();
        Ok(())
    }

    #[getter]
    fn has_text_frame(&self, py: Python<'_>) -> PyResult<bool> {
        self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        Ok(shape_ref_at(&presentation.inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .is_some())
    }

    #[getter]
    fn text_frame(&self, py: Python<'_>) -> PyResult<Py<PyTextFrame>> {
        self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        if shape_ref_at(&presentation.inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .is_none()
        {
            return Err(PyValueError::new_err("shape has no text frame"));
        }
        drop(presentation);
        Py::new(
            py,
            PyTextFrame::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    #[getter]
    fn has_table(&self, py: Python<'_>) -> PyResult<bool> {
        self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        Ok(shape_ref_at(&presentation.inner, &self.path)
            .and_then(|shape| shape.table())
            .is_some())
    }

    #[getter]
    fn table(&self, py: Python<'_>) -> PyResult<Py<PyTable>> {
        self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        if shape_ref_at(&presentation.inner, &self.path)
            .and_then(|shape| shape.table())
            .is_none()
        {
            return Err(PyValueError::new_err("shape has no table"));
        }
        drop(presentation);
        Py::new(
            py,
            PyTable::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }
}

#[pyclass(name = "ShapeCollection")]
pub struct PyShapeCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyShapeCollection {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "shape collection", ".shapes")?;
        slide_index(&self.path)
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let slide = self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        if self
            .path
            .segs
            .iter()
            .any(|segment| matches!(segment, PathSeg::Shape(_)))
        {
            return Ok(shape_ref_at(&presentation.inner, &self.path)
                .map_or(0, |shape| shape.child_count()));
        }
        Ok(presentation
            .inner
            .slide(slide)
            .map_or(0, |slide| slide.shapes().len()))
    }

    fn require_slide_root(&self) -> PyResult<()> {
        if self
            .path
            .segs
            .iter()
            .any(|segment| matches!(segment, PathSeg::Shape(_)))
        {
            return Err(PyValueError::new_err(
                "nested shape collections are read-only",
            ));
        }
        Ok(())
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyShape>> {
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Shape(index));
        let path = self.presentation.borrow(py).revisions.capture(segments);
        Py::new(py, PyShape::new(self.presentation.clone_ref(py), path))
    }

    fn capture_added(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyShape>> {
        let mut presentation = self.presentation.borrow_mut(py);
        presentation.revisions.bump();
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Shape(index));
        let path = presentation.revisions.capture(segments);
        drop(presentation);
        Py::new(py, PyShape::new(self.presentation.clone_ref(py), path))
    }
}

#[pymethods]
impl PyShapeCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.len(py)?;
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self
                .item(py, normalize_index(index, len, "shape")?)?
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
            "shape indices must be integers or slices",
        ))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyShapeIterator>> {
        self.validate(py)?;
        Py::new(
            py,
            PyShapeIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }

    #[getter]
    fn title(&self, py: Python<'_>) -> PyResult<Option<Py<PyShape>>> {
        let slide_index = self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        let Some(slide) = presentation.inner.slide(slide_index) else {
            return Ok(None);
        };
        let Some(title) = slide.title() else {
            return Ok(None);
        };
        let index = slide
            .shapes()
            .position(|shape| shape.placeholder_idx() == title.placeholder_idx())
            .expect("title is an immediate slide shape");
        drop(presentation);
        self.item(py, index).map(Some)
    }

    #[getter]
    fn placeholders(&self, py: Python<'_>) -> PyResult<Py<PyPlaceholderCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyPlaceholderCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    fn add_textbox(
        &mut self,
        py: Python<'_>,
        left: i64,
        top: i64,
        width: i64,
        height: i64,
    ) -> PyResult<Py<PyShape>> {
        self.require_slide_root()?;
        let slide_index = self.validate(py)?;
        let index = self.len(py)?;
        self.presentation
            .borrow_mut(py)
            .inner
            .slide_mut(slide_index)
            .expect("validated slide")
            .add_textbox(
                rpptx::Emu(left),
                rpptx::Emu(top),
                rpptx::Emu(width),
                rpptx::Emu(height),
            )
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        self.capture_added(py, index)
    }

    fn add_shape(
        &mut self,
        py: Python<'_>,
        shape_type: i32,
        left: i64,
        top: i64,
        width: i64,
        height: i64,
    ) -> PyResult<Py<PyShape>> {
        self.require_slide_root()?;
        let preset = match shape_type {
            51 => "homePlate",
            52 => "chevron",
            _ => return Err(PyValueError::new_err("unsupported MSO_SHAPE value")),
        };
        let slide_index = self.validate(py)?;
        let index = self.len(py)?;
        self.presentation
            .borrow_mut(py)
            .inner
            .slide_mut(slide_index)
            .expect("validated slide")
            .add_shape(
                preset,
                rpptx::Emu(left),
                rpptx::Emu(top),
                rpptx::Emu(width),
                rpptx::Emu(height),
            )
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        self.capture_added(py, index)
    }

    #[allow(clippy::too_many_arguments)]
    fn add_table(
        &mut self,
        py: Python<'_>,
        rows: usize,
        cols: usize,
        left: i64,
        top: i64,
        width: i64,
        height: i64,
    ) -> PyResult<Py<PyShape>> {
        self.require_slide_root()?;
        let slide_index = self.validate(py)?;
        let index = self.len(py)?;
        self.presentation
            .borrow_mut(py)
            .inner
            .slide_mut(slide_index)
            .expect("validated slide")
            .add_table(
                rows,
                cols,
                rpptx::Emu(left),
                rpptx::Emu(top),
                rpptx::Emu(width),
                rpptx::Emu(height),
            )
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        self.capture_added(py, index)
    }

    #[pyo3(signature = (image_file, left, top, width = None, height = None))]
    fn add_picture(
        &mut self,
        py: Python<'_>,
        image_file: PathBuf,
        left: i64,
        top: i64,
        width: Option<i64>,
        height: Option<i64>,
    ) -> PyResult<Py<PyShape>> {
        self.require_slide_root()?;
        let slide_index = self.validate(py)?;
        let index = self.len(py)?;
        let bytes = std::fs::read(&image_file)?;
        let filename = image_file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image.png");
        self.presentation
            .borrow_mut(py)
            .inner
            .add_picture(
                slide_index,
                &bytes,
                filename,
                rpptx::Emu(left),
                rpptx::Emu(top),
                width.map(rpptx::Emu),
                height.map(rpptx::Emu),
            )
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        self.capture_added(py, index)
    }
}

#[pyclass]
struct PyShapeIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PyShapeIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyShape>>> {
        let collection = PyShapeCollection::new(self.presentation.clone_ref(py), self.path.clone());
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "PlaceholderCollection")]
pub struct PyPlaceholderCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyPlaceholderCollection {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }
}

#[pymethods]
impl PyPlaceholderCollection {
    fn __getitem__(&self, py: Python<'_>, placeholder_idx: u32) -> PyResult<Py<PyShape>> {
        let presentation = self.presentation.borrow(py);
        validate_path(
            py,
            &presentation,
            &self.path,
            "placeholder collection",
            ".placeholders",
        )?;
        let slide_index = slide_index(&self.path)?;
        let slide = presentation
            .inner
            .slide(slide_index)
            .ok_or_else(|| PyIndexError::new_err("slide index out of range"))?;
        let placeholder = slide
            .placeholder(placeholder_idx)
            .ok_or_else(|| PyIndexError::new_err("placeholder index out of range"))?;
        let shape_index = slide
            .shapes()
            .position(|shape| shape.placeholder_idx() == placeholder.placeholder_idx())
            .expect("placeholder is an immediate slide shape");
        drop(presentation);
        let collection = PyShapeCollection::new(self.presentation.clone_ref(py), self.path.clone());
        collection.item(py, shape_index)
    }
}
