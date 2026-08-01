# F-074, DrawingML tables

**Status**: approved
**Sprint**: S17
**Size**: L
**Depends on**: F-064

## Problem

`oxml-drawing` has the completed text model needed by table cells, but it has
no model for `a:tbl`. Consequently `rpptx-oxml` can only retain a table inside
the raw `ShapeTreeChild::GraphicFrame(Vec<u8>)` payload at
`crates/rpptx-oxml/src/shape_tree.rs:23`. Callers cannot inspect grid widths,
rows, cells, merge origins, spans, or the table banding flags required by the
PresentationML scope.

The 50-deck corpus contains both ordinary tables and merged tables. The latter
use a span on the merge-origin `a:tc` and `hMerge` or `vMerge` on continuation
cells, so retaining only a rectangular matrix would lose the OOXML merge
contract.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Tables".
- `docs/hld/05-drawingml-model.md`, "Modules", "Text body", and
  "Preservation".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and "Preservation
  strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-074, DrawingML tables".

## Approach

Add `oxml-drawing/src/table.rs` and export these schema-shaped types:

```rust
pub struct CT_Table {
    pub properties: Option<CT_TableProperties>,
    pub grid: CT_TableGrid,
    pub rows: Vec<CT_TableRow>,
}

pub struct CT_TableGrid {
    pub columns: Vec<Emu>,
}

pub struct CT_TableProperties {
    pub right_to_left: bool,
    pub first_row: bool,
    pub first_column: bool,
    pub last_row: bool,
    pub last_column: bool,
    pub band_rows: bool,
    pub band_columns: bool,
    pub style_id: Option<String>,
}

pub struct CT_TableRow {
    pub height: Emu,
    pub cells: Vec<CT_TableCell>,
}

pub struct CT_TableCell {
    pub text_body: Option<CT_TextBody>,
    pub row_span: u32,
    pub grid_span: u32,
    pub horizontal_merge: bool,
    pub vertical_merge: bool,
}
```

The concrete structs also retain raw attributes and ordered raw children
privately. `row_span` and `grid_span` default to one, while merge flags default
to false. The origin remains the non-continuation cell carrying a span greater
than one. Continuation cells retain `hMerge` and `vMerge` independently, which
also covers two-dimensional merges.

`CT_Table::from_xml` accepts any prefix by local name and validates the
`a:tblPr`, `a:tblGrid`, then `a:tr+` sequence. `to_xml` writes fixed `a:`
prefixes in that order. Cell writers emit `a:txBody` before `a:tcPr`, and all
unmodelled attributes and subtrees are preserved at their schema boundary.
This includes unsupported table and cell styling without attempting to render
it.

The worker adds focused model tests in the new module and extends the existing
`rpptx-oxml/tests/integration.rs` binary to extract every `a:tbl` payload from
the pinned corpus, parse it as `CT_Table`, serialise it, and compare the
reparsed model.

F-074 integrates before F-073. F-073 then consumes `CT_Table` in its
`a:graphicData` dispatch without creating a reverse dependency from
`oxml-drawing` to `rpptx-oxml`.

## Rejected alternatives

- Put the table model in `rpptx-oxml`. `a:tbl` is DrawingML and the architecture
  assigns reusable DrawingML types to `oxml-drawing`.
- Flatten merged cells into a rectangular value matrix. That discards merge
  origins and continuation flags and cannot satisfy the backlog gate.
- Model cell fills, borders, and every table-style child now. F-074 needs their
  XML preserved, while later rendering and facade stories own behavioural use.
- Keep tables opaque until F-073. That would make payload recognition possible
  but would not expose the table structure required by this story.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `merged_table_round_trips_with_merge_origins_intact` | Origin spans, horizontal and vertical continuation flags, row heights, and grid widths survive serialise and reparse |
| unit | `table_properties_preserve_style_and_banding_flags` | Style id, direction, edge flags, and row and column banding retain their values |
| unit | `table_reader_is_prefix_tolerant_and_writer_uses_schema_order` | Alternate input prefixes are accepted and fixed-prefix output orders properties, grid, rows, cell text, and cell properties correctly |
| preservation | `unmodelled_table_and_cell_content_is_preserved_in_place` | Unsupported attributes and subtrees are re-emitted byte for byte at their original schema boundaries |
| round-trip | `every_corpus_drawingml_table_round_trips_structurally` | Every table payload in all 50 pinned decks serialises and reparses to an equal typed model |

The test gate is: a table with merged cells round-trips with merge origins
intact.

## HLD impact

- `docs/hld/05-drawingml-model.md`, add the table module and its merge and
  preservation contract to the current DrawingML model.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Test alternate read prefixes, fixed
  write prefixes, schema child order, and byte-for-byte preservation of each
  unmodelled subtree.
- A new module or file. Read the structural rules in `CLAUDE.md` and obtain
  explicit approval before adding `crates/oxml-drawing/src/table.rs`.

The consolidated sprint gate adds `cargo test -p oxml-drawing`,
`RDOCX_PPTX_CORPUS_REQUIRED=1 cargo test -p rpptx-oxml --test integration
every_corpus_drawingml_table_round_trips_structurally`, and
`cargo tree -p rpptx-oxml --edges normal`.

## Hash harness

Expected to be unchanged. The new model remains inside unpublished PowerPoint
development crates and does not modify the released Word path.

## Implementation checklist

- [ ] Add and export the DrawingML table module and concrete schema types.
- [ ] Parse and write table properties, style id, grid widths, rows, and cells.
- [ ] Retain merge-origin spans and horizontal and vertical continuation flags.
- [ ] Reuse `CT_TextBody` for cell text and preserve unsupported table styling.
- [ ] Add focused schema-order, prefix, merge, banding, and preservation tests.
- [ ] Add the all-corpus table round-trip gate to the existing integration test binary.
- [ ] Update the approved HLD impact file to describe the shipped table model.
- [ ] Confirm all PowerPoint development crates remain version 0.0.0 and unpublished.
- [ ] Confirm all 28 deterministic hashes remain unchanged.

## Open questions

None. The user approved the new DrawingML table module.
