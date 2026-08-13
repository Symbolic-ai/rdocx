use oxml_py_support::{ContentPath, PathSeg};
use pyo3::PyClass;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySlice};

use crate::normalize_index;
use crate::presentation::PyPresentation;
use crate::shape::{shape_mut_at, shape_ref_at};
use crate::validate_path;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyTextFrame>()?;
    module.add_class::<PyParagraph>()?;
    module.add_class::<PyParagraphCollection>()?;
    module.add_class::<PyRun>()?;
    module.add_class::<PyRunCollection>()?;
    module.add_class::<PyFont>()?;
    Ok(())
}

fn paragraph_index(path: &ContentPath) -> Option<usize> {
    path.segs.iter().find_map(|segment| match segment {
        PathSeg::Para(index) => Some(*index),
        _ => None,
    })
}

fn run_index(path: &ContentPath) -> Option<usize> {
    path.segs.iter().find_map(|segment| match segment {
        PathSeg::Run(index) => Some(*index),
        _ => None,
    })
}

#[pyclass(name = "TextFrame")]
pub struct PyTextFrame {
    pub(crate) presentation: Py<PyPresentation>,
    pub(crate) path: ContentPath,
}

impl PyTextFrame {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<()> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "text frame",
            ".text_frame",
        )
    }
}

#[pymethods]
impl PyTextFrame {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        self.validate(py)?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .map(|frame| frame.text())
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        shape_mut_at(&mut presentation.inner, &self.path)
            .and_then(rpptx::ShapeMut::into_text_frame)
            .map(|mut frame| frame.set_text(value))
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))?;
        presentation.revisions.bump();
        Ok(())
    }

    #[getter]
    fn paragraphs(&self, py: Python<'_>) -> PyResult<Py<PyParagraphCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyParagraphCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    fn add_paragraph(&self, py: Python<'_>) -> PyResult<Py<PyParagraph>> {
        self.validate(py)?;
        let original_path = self.path.clone();
        let index = {
            let mut presentation = self.presentation.borrow_mut(py);
            let mut frame = shape_mut_at(&mut presentation.inner, &original_path)
                .and_then(rpptx::ShapeMut::into_text_frame)
                .ok_or_else(|| PyValueError::new_err("shape has no text frame"))?;
            let index = frame.paragraph_count();
            frame.add_paragraph();
            index
        };
        let path = {
            let mut presentation = self.presentation.borrow_mut(py);
            presentation.revisions.bump();
            let mut segments = original_path.segs.clone();
            segments.push(PathSeg::Para(index));
            presentation.revisions.capture(segments)
        };
        Py::new(py, PyParagraph::new(self.presentation.clone_ref(py), path))
    }
}

#[pyclass(name = "Paragraph")]
pub struct PyParagraph {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyParagraph {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "paragraph",
            "",
        )?;
        paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))
    }
}

#[pymethods]
impl PyParagraph {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        let index = self.validate(py)?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .and_then(|frame| frame.paragraph(index))
            .map(|paragraph| paragraph.text())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        shape_mut_at(&mut presentation.inner, &self.path)
            .and_then(rpptx::ShapeMut::into_text_frame)
            .and_then(|frame| frame.into_paragraph_mut(index))
            .map(|mut paragraph| paragraph.set_text(value))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        presentation.revisions.bump();
        Ok(())
    }

    #[getter]
    fn level(&self, py: Python<'_>) -> PyResult<u8> {
        let index = self.validate(py)?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .and_then(|frame| frame.paragraph(index))
            .map(|paragraph| paragraph.level())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    #[setter]
    fn set_level(&self, py: Python<'_>, value: u8) -> PyResult<()> {
        let index = self.validate(py)?;
        let changed = shape_mut_at(&mut self.presentation.borrow_mut(py).inner, &self.path)
            .and_then(rpptx::ShapeMut::into_text_frame)
            .and_then(|frame| frame.into_paragraph_mut(index))
            .is_some_and(|mut paragraph| paragraph.set_level(value));
        if changed {
            Ok(())
        } else if value > 8 {
            Err(PyValueError::new_err("paragraph level must be from 0 to 8"))
        } else {
            Err(PyIndexError::new_err("paragraph index out of range"))
        }
    }

    #[getter]
    fn font(&self, py: Python<'_>) -> PyResult<Py<PyFont>> {
        self.validate(py)?;
        Py::new(
            py,
            PyFont {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
            },
        )
    }

    #[getter]
    fn runs(&self, py: Python<'_>) -> PyResult<Py<PyRunCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyRunCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }
}

#[pyclass(name = "ParagraphCollection")]
pub struct PyParagraphCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyParagraphCollection {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let presentation = self.presentation.borrow(py);
        validate_path(
            py,
            &presentation,
            &self.path,
            "paragraph collection",
            ".text_frame.paragraphs",
        )?;
        drop(presentation);
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .map(|frame| frame.paragraph_count())
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyParagraph>> {
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Para(index));
        let path = self.presentation.borrow(py).revisions.capture(segments);
        Py::new(py, PyParagraph::new(self.presentation.clone_ref(py), path))
    }
}

#[pymethods]
impl PyParagraphCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        sequence_item(py, key, self.len(py)?, "paragraph", |index| {
            self.item(py, index)
        })
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyParagraphIterator>> {
        self.len(py)?;
        Py::new(
            py,
            PyParagraphIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyParagraphIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PyParagraphIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyParagraph>>> {
        let collection =
            PyParagraphCollection::new(self.presentation.clone_ref(py), self.path.clone());
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Run")]
pub struct PyRun {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyRun {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }
}

#[pymethods]
impl PyRun {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "run", "")?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let run =
            run_index(&self.path).ok_or_else(|| PyIndexError::new_err("run index is missing"))?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .and_then(|frame| frame.paragraph(paragraph))
            .and_then(|paragraph| paragraph.run(run))
            .map(|run| run.text().to_owned())
            .ok_or_else(|| PyIndexError::new_err("run index out of range"))
    }
}

#[pyclass(name = "RunCollection")]
pub struct PyRunCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyRunCollection {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "run collection",
            ".runs",
        )?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .and_then(|frame| frame.paragraph(paragraph))
            .map(|paragraph| paragraph.run_count())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyRun>> {
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Run(index));
        let path = self.presentation.borrow(py).revisions.capture(segments);
        Py::new(py, PyRun::new(self.presentation.clone_ref(py), path))
    }
}

#[pymethods]
impl PyRunCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        sequence_item(py, key, self.len(py)?, "run", |index| self.item(py, index))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyRunIterator>> {
        self.len(py)?;
        Py::new(
            py,
            PyRunIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyRunIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PyRunIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyRun>>> {
        let collection = PyRunCollection::new(self.presentation.clone_ref(py), self.path.clone());
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Font")]
pub struct PyFont {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

#[pymethods]
impl PyFont {
    #[getter]
    fn bold(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "font",
            ".font",
        )?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        Ok(
            shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
                .and_then(|shape| shape.text_frame())
                .and_then(|frame| frame.paragraph(paragraph))
                .and_then(|paragraph| paragraph.default_run_properties())
                .and_then(|properties| properties.bold),
        )
    }

    #[setter]
    fn set_bold(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "font",
            ".font",
        )?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut paragraph = shape_mut_at(&mut presentation.inner, &self.path)
            .and_then(rpptx::ShapeMut::into_text_frame)
            .and_then(|frame| frame.into_paragraph_mut(paragraph))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        paragraph.default_run_properties_mut().bold = value;
        Ok(())
    }

    #[getter]
    fn size(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "font",
            ".font",
        )?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let size = shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|shape| shape.text_frame())
            .and_then(|frame| frame.paragraph(paragraph))
            .and_then(|paragraph| paragraph.default_run_properties())
            .and_then(|properties| properties.font_size);
        size.map(|centipoints| {
            py.import("rpptx")?
                .getattr("Length")?
                .call1((i64::from(centipoints) * 127,))
                .map(Bound::unbind)
        })
        .transpose()
    }

    #[setter]
    fn set_size(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "font",
            ".font",
        )?;
        let paragraph_index = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let centipoints = value
            .map(|emu| i32::try_from(emu / 127))
            .transpose()
            .map_err(|_| PyValueError::new_err("font size is out of range"))?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut paragraph = shape_mut_at(&mut presentation.inner, &self.path)
            .and_then(rpptx::ShapeMut::into_text_frame)
            .and_then(|frame| frame.into_paragraph_mut(paragraph_index))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        paragraph.default_run_properties_mut().font_size = centipoints;
        Ok(())
    }
}

fn sequence_item<T, F>(
    py: Python<'_>,
    key: &Bound<'_, PyAny>,
    len: usize,
    kind: &str,
    mut item: F,
) -> PyResult<Py<PyAny>>
where
    T: PyClass,
    F: FnMut(usize) -> PyResult<Py<T>>,
{
    if let Ok(index) = key.extract::<isize>() {
        return Ok(item(normalize_index(index, len, kind)?)?.into_any());
    }
    if key.is_instance_of::<PySlice>() {
        let (start, stop, step): (isize, isize, isize) =
            key.call_method1("indices", (len,))?.extract()?;
        let items = PyList::empty(py);
        let mut index = start;
        while if step > 0 { index < stop } else { index > stop } {
            items.append(item(index as usize)?)?;
            index += step;
        }
        return Ok(items.into_any().unbind());
    }
    Err(PyTypeError::new_err(format!(
        "{kind} indices must be integers or slices"
    )))
}
