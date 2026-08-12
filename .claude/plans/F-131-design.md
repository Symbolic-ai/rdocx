# F-131, rdocx-py formatting and tables

**Status**: completed
**Sprint**: S33
**Size**: L
**Depends on**: F-130, F-132

## Problem

The binding specification requires write-through `Font` and `ParagraphFormat`
subhandles plus Python tri-state formatting. The current Rust facade collapses
unset bold to false (`crates/rdocx/src/run.rs:317`) and its setter always writes
an explicit boolean (`crates/rdocx/src/run.rs:69`). Paragraph boolean readers
and setters have the same gap (`crates/rdocx/src/paragraph.rs:235`,
`crates/rdocx/src/paragraph.rs:540`). Document tables also lack direct mutable
lookup (`crates/rdocx/src/document.rs:454`), so persistent Python table and
cell paths cannot re-resolve through the public facade.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/10-bindings-spec.md`, "Two supporting decisions" and "Python API shape".
- `docs/hld/13-risks-and-open-questions.md`, "R9, index-path aliasing in the Python bindings".
- `docs/hld/14-development-backlog.md`, "F-131, rdocx-py formatting and tables".

## Approach

Add path-only `PyFont` and `PyParagraphFormat` subhandles. Each getter and
setter re-resolves the owning paragraph or run, so nested assignment writes
through without copying formatting. Add minimal facade `Option<bool>` readers
and tri-state setters for the S33 formatting inventory while preserving the
existing bool helpers. Assigning Python `None` clears direct formatting rather
than writing false.

Add direct total accessors for document tables and cell paragraphs. Implement
lazy table, row, cell, and nested paragraph handles using F-129 path segments.
All successful structural additions or removals bump the document revision.
Use F-132 values for enum and `Length` properties instead of exposing Rust
enums as pyclasses. The sprint wave therefore sequences this work after F-132.

The approved S33 surface is limited to facade-backed properties used by the
documented examples and table API: run font name, size, color, bold, italic,
underline and strike, paragraph alignment, spacing and indentation, table
style and dimensions, row/cell lookup, cell text and vertical alignment. Other
python-docx surfaces remain for the explicit F-135 parity story.

## Rejected alternatives

- Return false for inherited properties. That destroys the required
  `None`, false, and true distinction.
- Copy formatting into Python wrapper state. Nested setters must write through
  to the document.
- Depend directly on `rdocx-oxml`. The public facade remains the mutation
  boundary.
- Add stable IDs in this sprint. The specification reserves them for v0.2.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `unset_run_bold_is_none` | `r.font.bold` is `None` when unset rather than false |
| integration | `run_bool_tristate_round_trips` | `None`, false, and true remain distinct after save and reopen |
| integration | `none_clears_direct_formatting` | Assigning `None` removes the direct boolean value |
| integration | `format_subhandles_become_stale_after_structure_change` | Held font and paragraph-format handles enforce revision checks |
| round-trip | `table_handles_write_through_and_reopen` | Table, row, cell, text, and paragraph mutations survive reopen |
| regression | `facade_table_and_tristate_accessors_are_total` | New Rust accessors preserve old helpers and never panic |

The first integration test is the backlog gate. Focused commands are
`cargo test -p rdocx`, the non-extension binding check, and the approved
maturin plus formatting/table pytest command.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/14-development-backlog.md`

The HLD must record direct table/run accessors, explicit tri-state clearing,
the bounded S33 property inventory, and the real F-132 dependency.

## Risk routing

- WASM or PyO3 bindings. Retain workspace binding exclusions and run the
  existing rdocx WASM check.
- New module or file. Obtain explicit approval for dedicated formatting and
  table modules plus one Python test file.
- Public API of published `rdocx`. Record additive semver impact, run the full
  publication dry run, and enforce the archive size assertion.
- Unit conversion. Delegate every Python `Length` value through F-132 and the
  canonical Rust type. Preserve truncation and run conversion regressions.

## Hash harness

Expected unchanged. Existing bool setters retain their current serialization,
and new clearing behavior is reached only through the new binding surface.

## Implementation checklist

- [x] Add facade tri-state readers and clear-capable setters.
- [x] Add direct document table and cell paragraph accessors.
- [x] Implement path-only font and paragraph-format subhandles.
- [x] Implement lazy table, row, cell, and nested paragraph handles.
- [x] Wire the approved F-132 enum and unit inventory.
- [x] Add Rust and Python integration regressions and run all riders.

## Open questions

None. F-132 sequencing, the bounded S33 surface, additive facade APIs, and the
dedicated binding modules and test file were approved together.
