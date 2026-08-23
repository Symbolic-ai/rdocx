# F-173, Tagged PDF structure tree

**Status**: completed
**Sprint**: S53
**Size**: L
**Depends on**: none

## Problem

`LayoutResult` carries pages, fonts, metadata, outlines, and diagnostics at
`crates/oxml-layout/src/output.rs:343`, but it has no semantic document tree.
Word layout retains heading level only long enough to create an outline at
`crates/rdocx-layout/src/paginator.rs:416`. Table header state exists at
`crates/rdocx-layout/src/table.rs:42`, while list nesting and image alternate
text never reach the backend-neutral output.

The PDF catalog at `crates/oxml-pdf/src/writer.rs:478` links only pages and
outlines. Content streams contain visible drawing operators without marked
content, `/StructTreeRoot`, `/ParentTree`, heading, list, table, or figure
semantics. The visual PDF can therefore be correct while assistive technology
sees no usable document structure.

## Spec reference

- ISO 32000-1, sections 14.7 and 14.8, marked content and logical structure.
- ISO 14289-1, PDF/UA-1 logical structure requirements.
- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/08-rendering-spec.md`, "The PDF backend", "Tables", and "The
  renderer's input".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and "WASM".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness", and
  "The golden-PNG gate".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and
  "Packaging".

## Approach

Add backend-neutral semantic types beside `LayoutResult` in
`crates/oxml-layout/src/output.rs`: `DocumentStructure`, `StructureNode`,
`StructureRole`, and a stable `StructureId`. Add a non-drawing
`PositionedElement::MarkedContent` container that carries one structure id and
children. The existing non-exhaustive enum remains source compatible for
callers that already use a wildcard. Raster and geometry walkers recurse
through the container without changing pixels.

Carry Word semantics through layout instead of reconstructing them from glyph
positions. Paragraph blocks retain heading and list roles, table blocks retain
row and header-cell structure, and drawings retain alternate text. Pagination
wraps the exact emitted leaf elements in structure containers and builds one
document-order tree with heading levels, nested `L` and `LI` nodes, `Table`,
`TR`, `TH`, `TD`, and `Figure` nodes. Decorative drawing and border operations
are artifacts rather than structure children.

Extend `oxml-pdf` to allocate deterministic structure element references,
page-local MCIDs, `/StructParents`, the parent number tree, role maps where
needed, and `/MarkInfo`. Emit `BDC` and `EMC` around marked leaves while
leaving visible operators and resources unchanged. Images use `/Alt` from the
source description. Empty metrics-only text carriers produce no marked
content.

Validate one deterministic PDF with veraPDF 1.30.2 using the `ua1` profile.
Keep the existing PDF API and every binding unchanged. Untagged Presentation
layouts continue to render normally with `structure: None`.

## Rejected alternatives

- Inferring headings, lists, and tables from final coordinates loses source
  roles and cannot distinguish layout coincidence from semantics.
- Encoding Word-only types in `oxml-pdf` would reverse the format-neutral
  dependency boundary.
- Treating the outline tree as the structure tree omits list, table, figure,
  alternate-text, and content ownership information.
- Adding invisible text for structure would change extraction and risk visible
  output drift.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `marked_content_is_backend_neutral_and_non_drawing` | Walkers and raster output recurse through semantic containers without geometry or pixel changes. |
| regression | `tagged_pdf_preserves_heading_and_nested_list_structure` | `/StructTreeRoot`, parent tree, MCIDs, `H1` through `H6`, `L`, `LI`, and document order match the Word source. |
| regression | `tagged_pdf_marks_table_headers_and_cells` | Rows become `TR`, repeated header cells remain `TH`, data cells are `TD`, and content belongs to exactly one structure node. |
| regression | `tagged_pdf_carries_figure_alternate_text` | Informative images have `/Alt` and decorative paint is marked as artifact. |
| golden | `tagging_preserves_visible_pdf_and_raster_output` | Deterministic PNGs are pixel-identical and PDF page resources stay unchanged while only marked content and structural objects move. |
| differential | `tagged_pdf_passes_verapdf_pdf_ua_1` | veraPDF 1.30.2 `ua1` reports the constructed semantic fixture conforming. |

The test gate is **regression**. A rendered PDF carries a structure tree whose
heading and list nesting matches the source document.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout and pagination: re-read `docs/hld/08-rendering-spec.md`. Use bundled
  deterministic fonts for every baseline and require unchanged decoded PNGs.
- Public API of published crates: document the additive pre-1.0 semantic
  carriers. Run package dry runs for `oxml-layout`, `oxml-pdf`,
  `rdocx-layout`, `rdocx`, `rpptx-render`, and `rpptx`, with archive-size
  checks.
- WASM bindings: run both WASM checks and keep semantic carriers free of
  host-only dependencies.
- External oracle comparison: pin veraPDF 1.30.2, record the `ua1` profile and
  exact output, and retain the generated fixture for audit.

## Hash harness

Expected reviewed delta: for each of the seven generated samples,
`pdf/pages` and `pdf/bytes` change because content streams gain marked-content
operators and the file gains structure objects. `pdf/resources`, page-one PNG,
and every OOXML part remain unchanged. This is 14 changed entries and no added
or removed entries.

## Implementation checklist

- [x] Add the minimal semantic tree and marked-content carrier to
      `oxml-layout`.
- [x] Carry headings, list nesting, table roles, headers, and image alternate
      text through Word layout and pagination.
- [x] Mark decorative operators as artifacts.
- [x] Emit deterministic MCIDs, structure elements, parent tree, and catalog
      entries.
- [x] Preserve Presentation rendering when no structure tree exists.
- [x] Add structural, extraction, raster, PDF, warm-cold, WASM, packaging, and
      veraPDF checks.
- [x] Review the exact 14-entry hash delta as its own behavioural commit.

## Open questions

None. The new internal file `crates/oxml-pdf/src/structure.rs` is approved so
the structure-tree allocator and serializer stay out of the 2,400 line writer.
