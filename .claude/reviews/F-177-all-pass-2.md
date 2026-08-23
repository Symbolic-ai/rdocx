# F-177, all, pass 2

**Reviewed**: cumulative working-tree diff from base `e7fee7c` across `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `crates/rdocx/src/rtf.rs`, and `crates/rdocx/tests/integration_test.rs`, with 1,593 insertions and 4 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, table and row properties are still silently dropped
`crates/rdocx/src/rtf.rs:231`

`scan_table` reports only table raw XML, row raw XML, cell properties, and cell content. It never inspects `table.properties` or `row.properties`, even though those model public and parsed formatting fields such as table style, width, alignment, borders, margins, layout, indent, shading, table look, table property changes, row height, header-row state, row alignment, cant-split, row conditional style, and row revisions at `crates/rdocx-oxml/src/table.rs:359` and `crates/rdocx-oxml/src/table.rs:739`. `write_table` then serializes only `\trowd`, computed cell boundaries, paragraph contents, `\cell`, and `\row` at `crates/rdocx/src/rtf.rs:636`, so a document using public table APIs such as `Table::set_style`, `Table::set_width`, `Table::set_alignment`, `Table::set_borders`, `Table::set_cell_margins`, or `Table::set_layout_fixed` at `crates/rdocx/src/table.rs:56` loses those properties with an empty diagnostic list. The same is true for public row height, header, and cant-split setters at `crates/rdocx/src/table.rs:275`. The F-177 contract requires each unsupported or lossy source item to emit one stable diagnostic instead of disappearing silently.

### D2, cell width is neither exported nor diagnosed
`crates/rdocx/src/rtf.rs:278`

The pass 1 cell-property remediation diagnoses vertical merge, borders, shading, vertical alignment, no-wrap, text direction, and conditional style, but it skips `CT_TcPr::width` at `crates/rdocx-oxml/src/table.rs:940`. The writer does not otherwise preserve that field. Row boundaries are derived from the table grid or from a fixed 1440 twip fallback at `crates/rdocx/src/rtf.rs:1053`, so a cell width set through `Cell::set_width` at `crates/rdocx/src/table.rs:464`, or read from a DOCX table with a missing or short grid, can be replaced by fallback RTF boundaries with no diagnostic. This leaves the caller with neither a faithful table width round-trip nor the lossy evidence required by the design.

### D3, run formatting diagnostics are not exhaustive or per property
`crates/rdocx/src/rtf.rs:508`

`scan_run` groups several unsupported run properties into one generic run-level diagnostic, and it omits other lossy fields entirely. Public underline styles are collapsed to plain `\ul` for every non-none value at `crates/rdocx/src/rtf.rs:779`, so `Run::set_underline_style` at `crates/rdocx/src/run.rs:158` loses the exact underline style without a diagnostic. Public double strike and text position are modelled at `crates/rdocx-oxml/src/properties.rs:762` and `crates/rdocx-oxml/src/properties.rs:784`, settable at `crates/rdocx/src/run.rs:292` and `crates/rdocx/src/run.rs:369`, and not emitted by `write_run_format` at `crates/rdocx/src/rtf.rs:777`, but they are also absent from the scanner's diagnostic predicate. Other modelled fields such as complex-script bold, italic, and size are dropped the same way. The cases that are in the predicate, for example style, theme colour, spacing, width scale, and revisions, produce one combined message at `crates/rdocx/src/rtf.rs:518` rather than one exact item and location. That violates the design requirement that each unsupported or lossy source item emits one stable diagnostic.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 1 verification

- Pass 1 D1 is fixed for the audited cases. Table-cell paragraphs now begin with `\pard\intbl` at `crates/rdocx/src/rtf.rs:684`, and the added regression covers multi-paragraph and next-cell resets.
- Pass 1 D2 is fixed for the enumerated `CT_PPr` fields. The remaining diagnostics gap is table, row, cell width, and run formatting, recorded above.
- Pass 1 D3 is fixed for physical horizontal cell counts. `table_row_cell_widths` now returns one width per physical row cell at `crates/rdocx/src/rtf.rs:1053`.
- Pass 1 S1 is fixed for the audited output bound. `BoundedOutput` checks every write at `crates/rdocx/src/rtf.rs:150`, picture hex preflights expansion at `crates/rdocx/src/rtf.rs:1036`, and writer diagnostics cap at `crates/rdocx/src/rtf.rs:940`.

## Not found

- No remaining finding in basic table-cell paragraph resets across multiple paragraphs, next cells, nested-table drops, or list paragraphs.
- No remaining finding in the tested grid-span physical cell count case.
- No finding in deterministic font, colour, and list header ordering for the supported subset.
- No finding in Unicode scalar escaping, signed UTF-16 emission, or brace and backslash escaping.
- No finding in PNG and JPEG picture emission or truncating goal-dimension conversion.
- No finding in the additive public API names `RtfWriteResult`, `Document::to_rtf_bytes`, and `Document::save_rtf`.
- No source edit or remediation was attempted.

## Checks

- `python3 scripts/prose_check.py`, passed.
- `git diff --check`, passed.
