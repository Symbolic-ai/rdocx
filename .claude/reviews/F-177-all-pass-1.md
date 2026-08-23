# F-177, all, pass 1

**Reviewed**: complete worker diff from base `e7fee7c` across `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `crates/rdocx/src/rtf.rs`, and `crates/rdocx/tests/integration_test.rs`, with 1,109 added lines and 4 removed lines
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Defects

### D1, table-cell paragraph formatting leaks because cell paragraphs are not reset
`crates/rdocx/src/rtf.rs:421`

`write_table` emits each cell paragraph through `write_paragraph_contents`, but that helper writes only the requested paragraph properties at `crates/rdocx/src/rtf.rs:454`. It does not start the table-cell paragraph with `\pard` or another reset. The only separator for a later paragraph in the same cell is `\par` at `crates/rdocx/src/rtf.rs:419`, and the next cell starts immediately after `\cell` at `crates/rdocx/src/rtf.rs:428`. A table whose first cell paragraph is centered and whose next paragraph or next cell is default therefore leaves `\qc` active in the emitted RTF. The F-177 plan explicitly requires formatting resets so state cannot leak between paragraphs, cells, or rows, and this path violates that for table content.

### D2, public paragraph formatting is silently dropped without diagnostics
`crates/rdocx/src/rtf.rs:213`

The scanner for a paragraph reports raw XML, wrappers, revisions, ranges, bookmarks, hyperlinks, and then scans runs at `crates/rdocx/src/rtf.rs:217`, but it never inspects `CT_PPr` for unsupported paragraph properties. The writer only emits alignment, indents, spacing, line spacing, and numbering at `crates/rdocx/src/rtf.rs:467`. That leaves public facade state such as keep-with-next, keep-together, page-break-before, widow control, borders, tabs, outline level, shading, paragraph-mark run properties, and section properties dropped with an empty diagnostic list. Those fields are modelled on `CT_PPr` at `crates/rdocx-oxml/src/properties.rs:116`, and several are directly settable through the public facade, for example `set_keep_with_next_value` at `crates/rdocx/src/paragraph.rs:463`. The milestone and design require every safe lossy export to return a stable diagnostic instead of disappearing silently.

### D3, horizontally merged table cells can serialize as malformed RTF
`crates/rdocx/src/rtf.rs:408`

For each row, the writer derives the RTF cell boundaries from the table grid through `table_column_widths`, which returns every grid column when a grid exists at `crates/rdocx/src/rtf.rs:808`. It then emits one `\cell` per physical row cell at `crates/rdocx/src/rtf.rs:415`. A valid DOCX row with one cell spanning two grid columns, created by the public `grid_span` API at `crates/rdocx/src/table.rs:494`, therefore produces two `\cellx` boundaries but only one `\cell`. The F-176 reader rejects that shape as a row cell-count mismatch, and other consumers see a malformed or changed table. The merge is also not diagnosed during `scan_table`, so the caller receives neither a valid round-trip nor lossy evidence.

## Smells

### S1, writer bounds are checked after allocation and diagnostics are uncapped
`crates/rdocx/src/rtf.rs:148`

The writer checks `MAX_RETAINED_OUTPUT_BYTES` only after the whole RTF byte vector has been built. Large image bytes are hex-expanded by `write_hex_bytes` at `crates/rdocx/src/rtf.rs:793` before that check can fail, so a document can allocate far beyond the intended retained-output bound before returning an error. The diagnostic path also pushes without the reader's `MAX_DIAGNOSTICS` guard at `crates/rdocx/src/rtf.rs:700`. This is not a small-document correctness failure, but it leaves the writer's memory and diagnostic bounds weaker than the bounded reader contract it extends.

## Nitpicks

None. Count: 0.

## Not found

- Header allocation for fonts, colours and simple list tables is deterministic for the scanned body order.
- Escaping of backslashes and braces and signed UTF-16 emission for BMP and supplementary scalar values produced no finding.
- Basic body paragraph run resets, list table emission, PNG and JPEG source-order emission, and truncating image goal dimensions produced no additional finding.
- The additive public API shape matches the approved `RtfWriteResult`, `Document::to_rtf_bytes`, and `Document::save_rtf` plan.
- Existing DOCX save and reopen behaviour for preserved unmodelled body XML was not changed by this diff.
- The focused writer tests are real for the simple covered cases, but they do not cover the table-state, merged-cell, or public paragraph-format diagnostic failures above.

## Checks

- Review only. No remediation attempted.
