# F-102, Table rendering

**Status**: approved
**Sprint**: S25
**Size**: L
**Depends on**: F-074, F-098

## Problem

The DrawingML table model retains grid dimensions, band flags, spans, merge
continuations, and cell text, but leaves cell fills, borders, margins, and the
table style list opaque at `crates/oxml-drawing/src/table.rs:68`. The resolver
therefore freezes only dimensions, text, and merge flags at
`crates/rpptx-layout/src/context.rs:1274`, while the renderer drops
`ResolvedContent::Table` through its empty-content arm at
`crates/rpptx-render/src/lib.rs:250`.

The backlog gate requires a banded merged table with concrete fills and no
duplicated borders. Direct cell properties alone cannot meet that gate because
the pinned corpus contains tables whose visual state comes from
`ppt/tableStyles.xml`.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Tables" and "Rendering".
- `docs/hld/05-drawingml-model.md`, "Tables".
- `docs/hld/06-presentationml-model.md`, package part inventory and preserved
  PresentationML package parts.
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract".
- `docs/hld/08-rendering-spec.md`, "Text in a shape" and the backend-neutral
  page-frame contract.
- `docs/hld/12-testing-strategy.md`, deterministic render evidence.
- `docs/hld/14-development-backlog.md`, "F-102, Table rendering".

## Approach

Type the rendering subset of table cell properties and table styles inside the
existing `oxml-drawing/src/table.rs` module. Add concrete table-style region,
cell-style, fill, border, margin, and text-style values while preserving every
unsupported attribute and child at its ordered raw boundary. Parse and write
`CT_TableStyleList` with prefix-tolerant reads, fixed `a:` writes, and schema
child order.

Let `ResolveCtx` accept an optional table style list without breaking its
existing constructor. Resolve the table default or explicit style, overlay the
ECMA-defined whole-table, band, edge, and corner regions, then overlay direct
cell properties. Freeze concrete fill, margins, four edge strokes, text style,
spans, and merge ownership into `ResolvedTableCell`. Ignore cell autofit as the
declared v1 fallback and append a stable diagnostic when it is requested.

Add a private `lower_table` path in `rpptx-render`. Compute grid and row offsets,
emit only merge origins, span each origin over its covered rows and columns,
and render one fill and one text block per origin. Build a logical boundary map
for shared edges, apply the documented adjacent-border conflict precedence,
and emit each physical segment once. Reuse the S24 fixed-box text path with the
resolved cell margins and without cell autofit.

## Rejected alternatives

- Render only direct `a:tcPr`. It cannot produce banded tables whose appearance
  comes from `ppt/tableStyles.xml`.
- Draw every stored cell independently. Continuation cells would duplicate
  fills, text, and shared borders.
- Add a separate table renderer module. The current renderer is one concrete
  lowering path, and a new file is not needed.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `table_style_and_cell_properties_preserve_unmodelled_xml_byte_for_byte` | Typed additions retain unsupported table XML at its original boundary |
| unit | `table_style_regions_resolve_in_documented_precedence` | Whole-table, band, edge, corner, and direct-cell overlays follow the approved order |
| regression | `table_cell_autofit_is_ignored_and_records_a_diagnostic` | Cell text remains visible without unsupported autofit |
| integration | `banded_merged_table_renders_correct_fills_without_duplicated_borders` | The backlog gate, including sampled band pixels, visible text, merged bounds, and one physical edge per boundary |
| regression | `merged_continuation_cells_do_not_render_fill_border_or_text_twice` | Only merge origins emit visual content |
| unit | `table_cell_margins_place_text_in_the_fixed_content_box` | Resolved margins feed the shared text layout exactly |

The test gate is a banded table with merged cells renders with correct fills
and no duplicated borders.

## HLD impact

- `docs/hld/05-drawingml-model.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add alternate-prefix, fixed-prefix,
  schema-order, and byte-identical raw-subtree round-trip checks.
- Layout, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Use deterministic fonts for every raster
  assertion and never record a system-font baseline.
- Theme colour, tint, shade, colour mapping: read
  `docs/hld/05-drawingml-model.md`. Reuse the spec-correct `oxml-drawing`
  colour path and do not change the deliberately naive Word adapter.
- Unit conversion: read `docs/hld/01-glossary.md` and the deliberate truncation
  note in `CLAUDE.md`. Reuse the existing EMU-to-points conversion without
  changing constructor semantics.

Extra focused checks are `cargo test -p oxml-drawing`,
`cargo test -p rpptx-layout`, and `cargo test -p rpptx-render` with
deterministic pixel evidence.

## Hash harness

Expected to be unchanged. Table rendering affects only unpublished PowerPoint
development crates.

## Implementation checklist

- [ ] Type the rendering subset of table styles and direct cell properties.
- [ ] Preserve unsupported table XML and schema order on round-trip.
- [ ] Resolve style regions, direct overrides, margins, text style, and merge
  ownership to source-neutral values.
- [ ] Lower merge origins to fills, unique border segments, and fixed-box text.
- [ ] Diagnose and ignore unsupported table-cell autofit.
- [ ] Add deterministic structural and pixel regressions for the backlog gate.

## Open questions

None. The approved scope includes `ppt/tableStyles.xml`, table text styles, and
the ECMA-defined table-region and adjacent-border precedence. Diagonal borders,
effects, 3-D cell properties, and cell autofit remain preserved and diagnosed
but are not rendered in F-102.
