use std::path::PathBuf;

use oxml_py_support::{PathSeg, RevisionCounter};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyList};
use smallvec::smallvec;

use crate::paragraph::{PyParagraph, PyParagraphCollection};
use crate::rdocx_to_pyerr;
use crate::table::{PyTable, PyTableCollection};

#[pyclass(name = "Document")]
pub struct PyDocument {
    pub(crate) inner: rdocx::Document,
    pub(crate) revisions: RevisionCounter,
}

impl PyDocument {
    fn from_document(inner: rdocx::Document) -> Self {
        Self {
            inner,
            revisions: RevisionCounter::new(),
        }
    }
}

#[pymethods]
impl PyDocument {
    #[new]
    #[pyo3(signature = (path = None))]
    fn new(path: Option<PathBuf>, py: Python<'_>) -> PyResult<Self> {
        match path {
            Some(path) => rdocx::Document::open(path)
                .map(Self::from_document)
                .map_err(|error| rdocx_to_pyerr(py, error)),
            None => Ok(Self::from_document(rdocx::Document::new())),
        }
    }

    #[staticmethod]
    fn open(path: PathBuf, py: Python<'_>) -> PyResult<Self> {
        rdocx::Document::open(path)
            .map(Self::from_document)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[staticmethod]
    fn from_bytes(bytes: &[u8], py: Python<'_>) -> PyResult<Self> {
        rdocx::Document::from_bytes(bytes)
            .map(Self::from_document)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    fn save(&mut self, path: PathBuf, py: Python<'_>) -> PyResult<()> {
        self.inner
            .save(path)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[pyo3(name = "to_bytes")]
    fn serialize<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        py.detach(|| self.inner.to_bytes())
            .map(|bytes| PyBytes::new(py, &bytes))
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    fn to_pdf<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        py.detach(|| self.inner.to_pdf())
            .map(|bytes| PyBytes::new(py, &bytes))
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[pyo3(signature = (page_index, dpi = 150.0))]
    fn render_page_to_png<'py>(
        &self,
        py: Python<'py>,
        page_index: usize,
        dpi: f64,
    ) -> PyResult<Option<Bound<'py, PyBytes>>> {
        py.detach(|| self.inner.render_page_to_png(page_index, dpi))
            .map(|bytes| bytes.map(|bytes| PyBytes::new(py, &bytes)))
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[pyo3(signature = (dpi = 150.0))]
    fn render_all_pages<'py>(&self, py: Python<'py>, dpi: f64) -> PyResult<Bound<'py, PyList>> {
        let pages = py
            .detach(|| self.inner.render_all_pages(dpi))
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        PyList::new(py, pages.iter().map(|page| PyBytes::new(py, page)))
    }

    #[pyo3(signature = (*, dpi = 150.0, format = "png", quality = 90, transparent = false, pages = None))]
    fn render_pages(
        &self,
        py: Python<'_>,
        dpi: f64,
        format: &str,
        quality: u8,
        transparent: bool,
        pages: Option<Vec<usize>>,
    ) -> PyResult<Py<PyAny>> {
        let rendered = py
            .detach(|| {
                let format = parse_raster_format(format, quality, transparent)?;
                let selected = match pages {
                    Some(pages) => pages,
                    None => (0..self.inner.layout()?.layout.pages.len()).collect(),
                };
                self.inner
                    .render_pages(&selected, rdocx::RasterOptions { dpi, format })
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        match rendered {
            rdocx::RasterOutput::SeparatePages(pages) => {
                let list = PyList::new(py, pages.iter().map(|page| PyBytes::new(py, page)))?;
                Ok(list.into_any().unbind())
            }
            rdocx::RasterOutput::MultiPageTiff(tiff) => {
                Ok(PyBytes::new(py, &tiff).into_any().unbind())
            }
        }
    }

    #[getter]
    fn paragraphs(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PyParagraphCollection>> {
        Py::new(py, PyParagraphCollection::new(slf))
    }

    #[getter]
    fn tables(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PyTableCollection>> {
        Py::new(py, PyTableCollection::new(slf))
    }

    fn add_paragraph(slf: Py<Self>, py: Python<'_>, text: &str) -> PyResult<Py<PyParagraph>> {
        let (index, path) = {
            let mut document = slf.borrow_mut(py);
            let index = document.inner.paragraph_count();
            document.inner.add_paragraph(text);
            document.revisions.bump();
            let path = document
                .revisions
                .capture(smallvec![PathSeg::Body(0), PathSeg::Para(index)]);
            (index, path)
        };
        debug_assert!(matches!(path.segs.last(), Some(PathSeg::Para(i)) if *i == index));
        Py::new(py, PyParagraph::new(slf, path))
    }

    #[pyo3(signature = (rows, cols))]
    fn add_table(slf: Py<Self>, py: Python<'_>, rows: usize, cols: usize) -> PyResult<Py<PyTable>> {
        let (index, path) = {
            let mut document = slf.borrow_mut(py);
            let index = document.inner.table_count();
            document.inner.add_table(rows, cols);
            document.revisions.bump();
            let path = document.revisions.capture(smallvec![PathSeg::Body(index)]);
            (index, path)
        };
        debug_assert!(matches!(path.segs.last(), Some(PathSeg::Body(i)) if *i == index));
        Py::new(py, PyTable::new(slf, path))
    }

    fn remove_content(&mut self, index: usize) -> bool {
        let removed = self.inner.remove_content(index);
        if removed {
            self.revisions.bump();
        }
        removed
    }
}

fn parse_raster_format(
    format: &str,
    quality: u8,
    transparent: bool,
) -> rdocx::Result<rdocx::RasterFormat> {
    match format {
        "png" => Ok(rdocx::RasterFormat::Png {
            transparent_background: transparent,
        }),
        "jpg" | "jpeg" => Ok(rdocx::RasterFormat::Jpeg { quality }),
        "tif" | "tiff" => Ok(rdocx::RasterFormat::Tiff),
        other => Err(rdocx::Error::Other(format!(
            "unknown raster format {other:?}, expected png, jpeg, or tiff"
        ))),
    }
}
