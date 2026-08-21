# Changelog

## Unreleased

The next stable rdocx release adopts the shared OOXML crates described below.
It is planned as version 0.8.0. The shared and PowerPoint crate family is
prepared separately at version 0.4.0.

### Migration table

| Previous path or crate | Replacement | Compatibility |
|---|---|---|
| `rdocx::Length` | `oxml_core::Length` | `rdocx::Length` remains an exact re-export |
| `rdocx_oxml::{core_properties, error, raw_xml, units}` | The same modules under `oxml_core` | The `rdocx_oxml` paths remain exact re-exports |
| `rdocx_opc` | `oxml_opc` | `rdocx-opc` is a deprecated exact re-export shim, except for the removed Word-only constructors listed below |
| Word-owned image sniffing, sizing, and media naming | `oxml_media::{resolve, probe, ImageFormat, ImageInfo, NativeSize, MediaNamer}` | These shared APIs are available directly from `oxml-media` |
| `rdocx_layout::bundled_fonts` | `oxml_layout::bundled_fonts` | The old module path is removed |
| `rdocx_layout::font::{FontManager, FontMetrics, ShapedText}` | The same types at the `oxml_layout` root | `rdocx_layout::input::FontFile` and `rdocx_layout::FontFile` remain exact re-exports of `oxml_layout::FontFile` |
| `rdocx_layout::error::{LayoutError, Result}` | `oxml_layout::{LayoutError, Result}` | The types also remain exact re-exports at the `rdocx_layout` root |
| `rdocx_layout::line::{InlineItem, LayoutLine, LineBreakParams, LineItem, TextSegment, break_into_lines}` | The same names at the `oxml_layout` root | The old `rdocx_layout::line` module is removed |
| `rdocx_layout::output::{Color, DocumentMetadata, FieldKind, FontData, FontId, GlyphRun, LayoutResult, OutlineEntry, PageFrame, Point, PositionedElement, Rect}` | The same names at the `oxml_layout` root | Types previously exported at the `rdocx_layout` root remain exact re-exports there |
| Exhaustive `TextSegment` and `GlyphRun` literals | Add `source: Option<SourceSpan>` | Use `None` for generated or unattributed text. Word provenance results supply exact result-local node ids and Unicode-scalar ranges |
| `rdocx_pdf` | `oxml_pdf` | `rdocx-pdf` is a deprecated exact re-export shim |
| `rdocx_pdf::raster::{render_page_to_png, render_all_pages}` | `oxml_pdf::{render_page_to_png, render_all_pages}` | The old nested `raster` path is removed. The functions remain available at the `rdocx_pdf` root through the shim |

`rdocx-oxml` and `rdocx-layout` are retained format-specific crates, not
deprecated shims. `rdocx-oxml` continues to own WordprocessingML types.
`rdocx-layout` continues to own the Word flow engine, paginator, blocks,
tables, style resolver, and Word-to-shared conversion boundary. The `rdocx`,
`rdocx-cli`, and `rdocx-html` crate names are unchanged.

### Shared dependencies

New direct users can select the format-neutral crate that owns each surface:

```toml
[dependencies]
oxml-core = "0.4.0"   # Length, units, XML helpers, document properties
oxml-opc = "0.4.0"    # OPC package, relationships, and content types
oxml-media = "0.4.0"  # Image detection, dimensions, and media naming
oxml-layout = "0.4.0" # Layout output, fonts, and line breaking
oxml-pdf = "0.4.0"    # PDF and PNG rendering backends
```

### Breaking API changes

- `rdocx_opc::OpcPackage::new_docx()` and
  `rdocx_opc::ContentTypes::new_docx()` are removed. Use
  `oxml_opc::OpcPackage::new()` or `OpcPackage::with_main_part(...)`, plus
  `oxml_opc::ContentTypes::minimal()`, and add Word-specific defaults and
  overrides at the application boundary.
- `rdocx::Error::Opc` now contains `oxml_opc::OpcError`, and
  `rdocx::Error::Layout` now contains `oxml_layout::LayoutError`. The
  deprecated OPC shim and retained layout facade re-export those exact shared
  types, but code that spells payload paths in exhaustive matches should use
  the shared paths.
- The public `rdocx_layout::line` module is removed. Its shared replacement
  uses `MediaId` instead of relationship-scoped `embed_id` strings for image
  items. `TextSegment` uses `oxml_layout::Underline` and adds `line_gap`.
  `LayoutLine` adds `line_gap`. `LineBreakParams` replaces Word tab stops,
  alignment, and stringly typed line rules with `TabStop`, `Align`, and
  `LineSpacing`, and adds `wrap`.
- `rdocx_layout::engine::layout_paragraph(...)` and
  `rdocx_layout::table::layout_table(...)`, plus
  `rdocx_layout::paginator::paginate(...)` and
  `rdocx_layout::paginator::paginate_sections(...)`, now take a shared
  `MediaRegistry`. Construct it once from `LayoutInput::images` so relationship
  lookup and pagination use the same collision-resolved IDs, bytes, and
  content types.
- `rdocx_layout::AnchoredContent::Image` replaces `embed_id: String` with
  `media_id: MediaId`.
- `rdocx_layout::ParagraphBlock::jc` replaces `Option<ST_Jc>` with
  `Option<oxml_layout::Align>`.
- `PositionedElement` is non-exhaustive, replaces the optional image
  `embed_id` with `MediaId`, and adds `Path` and `Group` variants. External
  matches must include a wildcard arm.
- `PageFrame` is non-exhaustive and adds `background`. Construct it with
  `PageFrame::new(...)` when a default background is wanted.
- `LayoutResult` is non-exhaustive and adds `diagnostics`. Construct it with
  `LayoutResult::new(...)` when an empty diagnostics list is wanted.
- `oxml_layout::TextSegment` and `oxml_layout::GlyphRun` add the required
  `source: Option<SourceSpan>` field. External exhaustive literals must set it
  to `None` unless they own an exact source range. This source change ships in
  the incubating 0.4.0 family and the stable 0.8.0 family. Word callers can use
  `rdocx_layout::layout_document_with_provenance` or its deterministic variant
  to receive `WordLayoutResult`, resolve result-local nodes to
  `WordSourcePath`, and interpret exclusive character ranges as Unicode scalar
  indices in the recorded revision view.
- The nested `rdocx_pdf::raster` module is removed. Import its two rendering
  functions from the `oxml_pdf` root or from the compatible `rdocx_pdf` root.

### Media behavior and additive API

Word media insertion now detects the image format from its bytes before using
the filename extension. It allocates the next numeric media suffix after the
greatest occupied suffix, so gaps do not overwrite an existing part.

`rdocx::Document::add_picture_auto(image_data, image_filename)` adds an image
at its intrinsic size. It uses declared per-axis DPI when valid and a 72 DPI
fallback otherwise. If dimensions cannot be determined, it returns
`rdocx::Error::UnavailableImageDimensions` before changing the document.

## rpptx-v0.4.0

### Highlights

The complete shared OOXML and PowerPoint family moves together to 0.4.0. This
is the first release to publish `oxml-chart`, making the typed ChartML model,
authoring surface, and renderer available from its format-neutral home.

### Added

- `oxml-chart` now owns shared ChartML parsing, editing, authoring, and render
  geometry. `rpptx-chart` remains an exact compatibility re-export.
- `oxml-layout` glyph runs can carry exact `SourceSpan` provenance through
  shaping and line splitting, with generated or transformed text left
  truthfully unattributed.
- Normal host-font layout reuses a bounded process font snapshot, file-backed
  bytes, and exact-key shaping results. Deterministic and caller-font paths
  remain isolated from that state.

### Fixed

- Bounded OPC reads reject oversized declared ZIP entry counts before the ZIP
  index is constructed, and retain the configured byte and entry ceilings
  throughout package access.
- Deterministic PDF output now writes font, Unicode-map, and image resources in
  stable order, so identical inputs produce identical bytes.

### Compatibility

This is an intentional pre-1.0 source boundary. External exhaustive literals
for `TextSegment` and `GlyphRun` must add `source: None` unless they own an
exact `SourceSpan`. Existing `rpptx-chart` imports remain valid through the
exact re-export, while new direct users should depend on `oxml-chart`.

Normal system-font discovery is now a process-lifetime snapshot. Installing,
removing, or replacing host fonts requires a process restart. Deterministic and
caller-provided font behavior is unchanged. `rpptx-wasm` is prepared at 0.4.0
but remains unpublished on crates.io.

### Contributors

Atul Sharma maintained the release. `@emptinessform` supplied the provenance
and cache reports behind Issues 38 and 39. Pedro Assumpcao
(`@pedroassumpcao`) contributed bounded OPC reads in PR 33 and carried the
entry-limit hardening through PR 34. Jon Stokes (`@jonstokes`) authored the
ZIP entry-admission hardening commit integrated by PR 34.
