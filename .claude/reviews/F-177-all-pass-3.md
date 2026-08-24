# F-177, all, pass 3

**Reviewed**: cumulative working-tree diff from base `e7fee7c` across `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `crates/rdocx/src/rtf.rs`, and `crates/rdocx/tests/integration_test.rs`, with 2,310 insertions and 4 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, raw table-cell property XML is dropped without a diagnostic
`crates/rdocx/src/rtf.rs:414`

`scan_cell_properties` diagnoses modelled `CT_TcPr` fields from `tcW` through `cnfStyle`, but it never visits `CT_TcPr::extra_xml`. That field is where the DOCX parser preserves unmodelled `w:tcPr` children such as `w:hMerge` or foreign producer properties at `crates/rdocx-oxml/src/table.rs:958`. `write_table` then emits only row defaults, computed cell boundaries, paragraph contents, `\cell`, and `\row` at `crates/rdocx/src/rtf.rs:896`, so those retained property children are lost with no `RtfDiagnostic`. This violates the F-177 exact diagnostic contract at `.claude/plans/F-177-design.md:61`.

### D2, explicit `dxa` cell widths can produce silent fallback or invalid boundaries
`crates/rdocx/src/rtf.rs:1365`

`cell_width_twips` accepts every `dxa` value as supported. `scan_cell_width` returns as soon as any explicit width exists at `crates/rdocx/src/rtf.rs:473`, and `scan_cell_properties` checks only the width type at `crates/rdocx/src/rtf.rs:415`. A parsed DOCX cell with `w:tcW w:type="dxa" w:w="0"` is silently replaced with the 1440 twip fallback at `crates/rdocx/src/rtf.rs:1347`, while a negative value is emitted into the cumulative `\cellx` calculation. Multiple large positive cell widths also saturate the cumulative boundary at `crates/rdocx/src/rtf.rs:907`, which can make later cell boundaries repeat instead of increasing. These are lossy or malformed table exports with no diagnostic.

### D3, list levels and unsupported numbering formats are silently coerced
`crates/rdocx/src/rtf.rs:998`

The writer clamps any paragraph numbering level above 8 to `\ilvl8` when it emits paragraph format controls. Parsed DOCX can carry a larger `numPr/ilvl` in `CT_PPr` even though the public setter rejects it, and `scan_paragraph_properties` only diagnoses the missing-numbering-id case at `crates/rdocx/src/rtf.rs:628`. The same list export path maps an unsupported modelled numbering format such as `ST_NumberFormat::None` at `crates/rdocx-oxml/src/numbering.rs:2075` to decimal via `unwrap_or(ListNumberFormat::Decimal)` at `crates/rdocx/src/rtf.rs:1181`. Both changes alter list semantics and break the round-trip requirement without the required stable lossy diagnostic.

### D4, `save_rtf` is not the approved atomic path writer
`crates/rdocx/src/rtf.rs:89`

The F-177 design resolves the file API as an atomic path writer at `.claude/plans/F-177-design.md:125`, but `save_rtf` serializes and then writes directly to the destination with `std::fs::write` at `crates/rdocx/src/rtf.rs:91`. If the destination already exists and the filesystem write fails after truncation or partial replacement, the old file can be lost even though the method returns an error. The current success-path test at `crates/rdocx/src/lib.rs:148` does not cover that failure mode.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 1 and pass 2 verification

- Pass 1 D1 remains fixed for the audited table-cell paragraph cases. Table-cell paragraphs start with `\pard\intbl` at `crates/rdocx/src/rtf.rs:950`.
- Pass 1 D2 remains fixed for the enumerated `CT_PPr` fields. Paragraph style, auto-spacing, keep flags, page-break-before, widow control, suppress-auto-hyphens, outline level, borders, tabs, shading, paragraph-mark run properties, section properties, numbering revisions, and paragraph property revisions all have diagnostic paths in `scan_paragraph_properties`.
- Pass 1 D3 remains fixed for horizontal grid-span physical cell counts. `table_row_cell_widths` now returns one boundary width per physical row cell at `crates/rdocx/src/rtf.rs:1317`.
- Pass 1 S1 remains fixed for the audited bounds. `BoundedOutput` checks every write at `crates/rdocx/src/rtf.rs:150`, picture hex expansion preflights at `crates/rdocx/src/rtf.rs:1300`, and writer diagnostics are capped at `crates/rdocx/src/rtf.rs:1204`.
- Pass 2 D1 is fixed for modelled table and row properties. `scan_table_properties` and `scan_row_properties` now cover the public and parsed `CT_TblPr` and `CT_TrPr` fields at `crates/rdocx/src/rtf.rs:290` and `crates/rdocx/src/rtf.rs:363`.
- Pass 2 D2 is fixed for `pct` and `auto` cell widths, missing grids, and short grids. The remaining cell-width defect is limited to explicit `dxa` value validity and cumulative boundary overflow.
- Pass 2 D3 is fixed for the audited run-property matrix. Supported direct bold, italic, strike, font, size, colour, shading-backed highlight, caps, small caps, hidden, and vertical alignment remain diagnostic-free, while the unsupported run properties named in pass 2 now have per-property diagnostics at `crates/rdocx/src/rtf.rs:707`.

## Not found

- No additional finding in deterministic font, colour, list table, and list-override header ordering for supported inputs.
- No additional finding in body table, paragraph, list, and image source order for supported body items.
- No additional finding in table-cell paragraph resets, nested-table diagnostics, table-cell content-control diagnostics, or body content-control diagnostics.
- No additional finding in `CT_TblPr`, `CT_TrPr`, `CT_PPr`, or `CT_RPr` duplicate raw, change, or revision diagnostic paths beyond the list and cell-property defects above.
- No false positive found for supported direct paragraph formatting or supported direct run formatting.
- No additional finding in Unicode scalar escaping, signed UTF-16 surrogate emission, brace escaping, backslash escaping, tabs, or line breaks.
- No additional finding in PNG and JPEG picture emission, missing image diagnostics, unsupported image diagnostics, anchored drawing diagnostics, or truncating positive image goal dimensions.
- No additional finding in retained DOCX unmodelled body XML, because `to_rtf_bytes` reads without mutating the `Document` package state.

## Checks

- `python3 scripts/prose_check.py`, passed.
- `git diff --check`, passed.
