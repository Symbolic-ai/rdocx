# rdocx-layout

`rdocx-layout` turns semantic Word content into positioned pages. It resolves
styles and numbering, shapes text, lays out tables and notes, paginates
sections, and retains source provenance.

## Capabilities

- Word style, numbering, field, and paragraph resolution.
- Line breaking, tables, footnotes, endnotes, sections, and pagination.
- Deterministic bundled fonts plus system, embedded, and caller fonts.
- Page-reference lookup and Word source provenance.
- Shared `oxml-layout` output for downstream backends.

## Use it when

Use `layout_document_deterministic` for reproducible output with bundled fonts.
Applications that start from a DOCX file should normally call the rendering
methods on [`rdocx::Document`](https://docs.rs/rdocx) instead.

## Relationship

This crate converts Word-specific semantic input into the shared positioned
model from `oxml-layout`. PDF and raster backends consume that model. It does
not emit PDF or pixels itself.

## Example

```rust,no_run
use rdocx_layout::{LayoutInput, Result, layout_document_deterministic};

fn page_count(input: &LayoutInput) -> Result<usize> {
    Ok(layout_document_deterministic(input)?.pages.len())
}
```

```toml
[dependencies]
rdocx-layout = "0.13.1"
```
