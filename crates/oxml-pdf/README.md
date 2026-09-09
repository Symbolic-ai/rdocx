# oxml-pdf

Render format-neutral `LayoutResult` pages to PDF, archival PDF/A, or raster
images.

## Capabilities

- PDF output from positioned page elements.
- PDF/A-2b and PDF/A-3b preflight and rendering.
- Font embedding and subsetting, metadata, outlines, links, and structure.
- Selected-page PNG and JPEG plus multi-page TIFF output.

## Use it when

Use this crate when a custom OOXML frontend already produces shared layout frames. Use `rdocx::Document::to_pdf` or `rpptx::Presentation::to_pdf_deterministic` for normal document conversion.

## Relationship

This is the shared successor to the deprecated `rdocx-pdf` shim. It is an
output backend and does not open OOXML packages or lay out DOCX and PPTX
content. Raster page indices are zero-based.

## Example

```rust,no_run
use oxml_layout::LayoutResult;
use oxml_pdf::render_to_pdf;

let layout = LayoutResult::new(Vec::new(), Vec::new(), None, Vec::new());
let pdf = render_to_pdf(&layout);
assert!(pdf.starts_with(b"%PDF-"));
```

Add `oxml-pdf = "0.11.0"` and `oxml-layout = "0.11.0"` to your dependencies. See the [renderer API](https://docs.rs/oxml-pdf) for the accepted layout model.
