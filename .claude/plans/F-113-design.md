# F-113, Table facade

**Status**: approved
**Sprint**: S28
**Size**: L
**Depends on**: F-074, F-109

## Problem

DrawingML tables are already parsed and written by `CT_Table`, including grid
widths, rows, cells, merge spans, continuation flags, banding, fill, margins,
and preserved XML. The presentation facade can only flatten table text. It
cannot construct a table or expose behavior-bearing table and cell handles.

The story must provide `add_table`, cell text and text-frame mutation, cell
fill and margins, merge and split, banding, and column widths. The exact gate
is: merge then split restores the original grid.

## Spec reference

- `docs/hld/01-glossary.md`, "Units".
- `docs/hld/02-scope-and-non-goals.md`, "Tables".
- `docs/hld/03-architecture.md`, dependency direction.
- `docs/hld/04-opc-and-packaging.md`, preservation rules.
- `docs/hld/05-drawingml-model.md`, table and text models.
- `docs/hld/06-presentationml-model.md`, public facade and graphic frames.
- `docs/hld/14-development-backlog.md`, "F-113, Table facade".

## Approach

Add validated constructors to the existing table and graphic-frame modules:

```rust
impl CT_Table {
    pub fn new(rows: usize, columns: usize, width: Emu, height: Emu) -> Result<Self>;
}

impl CT_GraphicFrame {
    pub fn new_table(
        id: u32,
        name: &str,
        transform: CT_Transform2D,
        table: CT_Table,
    ) -> Result<Self>;
}
```

Reject zero counts, counts that cannot be represented, and non-positive
dimensions before allocation. Construct an explicit rectangular grid. Divide
the requested dimensions with the repository's truncating integer rule and
pin remainder behavior in tests. Every new cell contains a minimal text body
with one paragraph and default cell properties.

Add `SlideMut::add_table` beside the existing shape constructors. It allocates
a tree-wide shape id, uses the deterministic name `Table {id}`, appends at the
top z-order, and returns `ShapeMut`. Add `ShapeRef::table` and
`ShapeMut::table_mut`, backed by concrete borrowed `TableRef`, `TableMut`,
`TableCellRef`, and `TableCellMut` handles in the existing facade file.

The handles expose row and column counts, total indexed cell access, grid
widths, first-row, last-row, first-column, last-column, horizontal-banding,
and vertical-banding flags. Changing a grid width uses a checked sum and keeps
the graphic-frame width synchronized.

Cell handles expose text, a mutable text frame, fill, four optional margins,
merge origin and continuation state, span height and width, `merge_to`, and
`split`. `merge_to` accepts opposite corners in either order. It validates the
whole rectangle before mutation, moves non-empty typed paragraphs into the
top-left origin in row-major order, leaves one empty paragraph in each source
cell, and writes the DrawingML span and continuation pattern. Add one narrow
paragraph-transfer behavior to the existing text module so paragraph and run
formatting move without exposing raw storage.

`split` is valid only on a merge origin. It clears the spans and continuation
flags in that origin's validated rectangle. It restores the original explicit
cell grid and geometry. It does not redistribute text moved during merge,
matching python-pptx 1.0.2 semantics.

Use one contextual table-mutation error in the existing facade error enum.
Every fallible operation validates or stages all work before changing state.
No new module, file, trait, generic, feature, or dependency is introduced.

## Rejected alternatives

- Expose `&mut CT_Table`. That permits incoherent span and continuation state.
- Delete continuation cells. DrawingML retains every grid cell explicitly.
- Copy only plain text during merge. That loses paragraphs, runs, fields,
  bullets, and formatting.
- Change a grid width without the graphic-frame extent. That leaves conflicting
  table and shape bounds.
- Add collection wrappers, builders, or a table module. Existing files already
  own the behavior and no second abstraction is justified.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `merge_then_split_restores_the_original_grid` | Save and reopen restores span 1, clears continuation flags, and retains all rows, columns, widths, heights, and cells |
| unit | `two_dimensional_merge_encodes_origins_and_continuations` | Exact `rowSpan`, `gridSpan`, `hMerge`, and `vMerge` pattern |
| unit | `merge_moves_formatted_content_to_origin_in_row_major_order` | Paragraph and run formatting move in deterministic order and split does not redistribute content |
| integration | `add_table_round_trips_cells_formatting_banding_and_widths` | Text frames, fill, margins, flags, and edited widths survive save and reopen |
| negative | `table_mutation_rejects_invalid_ranges_without_partial_changes` | Invalid sizes, coordinates, overlap, split, and overflow leave bytes unchanged |
| preservation | `table_mutation_preserves_unmodelled_xml_and_schema_order` | Unsupported XML is byte-identical and output follows table schema order |
| differential | `add_table_matches_pinned_python_pptx_table_semantics` | python-pptx 1.0.2 agrees on dimensions, defaults, content migration, merge queries, and split behavior |

## HLD impact

- `docs/hld/05-drawingml-model.md`
- `docs/hld/06-presentationml-model.md`

Document table construction, borrowed handles, formatting and banding,
column-width synchronization, merge encoding, content migration, split
semantics, and preservation.

## Risk routing

- Unit conversion and `Emu`: use the truncating conversions defined in the
  glossary and `CLAUDE.md`. Assert exact quotient, remainder, and checked-sum
  results. Expected hash delta is none.
- Parser or serialiser: add prefix-tolerant parsing, fixed-prefix writing,
  schema-order, and byte-preservation checks required by HLD 04 and HLD 06.
- Dependency graph and cross-family use: inspect `cargo tree -p rpptx --edges
  normal`. The facade already depends on `oxml-drawing` and `rpptx-oxml`, so
  this adds no edge.
- External oracle: pin python-pptx 1.0.2 and compare object-model structure,
  not XML bytes, following the differential-testing skill.

The affected crates are unpublished at version 0.0.0. No published API,
layout, rendering, new file, module, trait, generic, feature, or dependency
rider applies.

## Hash harness

Expected unchanged. This story changes unpublished PowerPoint construction and
mutation paths only. All 28 deterministic hashes must match.

## Implementation checklist

- [ ] Add validated table and graphic-frame constructors in existing modules.
- [ ] Add narrow typed-paragraph transfer behavior.
- [ ] Add `add_table` and borrowed table and cell handles.
- [ ] Add text-frame, fill, margins, flags, and width mutation.
- [ ] Implement checked merge and merge-origin-only split.
- [ ] Add the gate, negative, preservation, round-trip, and differential tests
  to existing test binaries.
- [ ] Update exactly HLD 05 and HLD 06.
- [ ] Run focused checks, risk riders, `/verify --full`, and the hash harness.

## Open questions

None. The approved design adopts python-pptx 1.0.2 content-migration and split
semantics and pins truncating dimension distribution in tests.
