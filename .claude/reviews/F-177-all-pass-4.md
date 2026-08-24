# F-177, all, pass 4

**Reviewed**: cumulative working-tree diff from base `e7fee7c` across `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `crates/rdocx/src/rtf.rs`, and `crates/rdocx/tests/integration_test.rs`, with 2,637 insertions and 5 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks
**Counts**: defects 1, smells 0, nitpicks 0

## Defects

### D1, `save_rtf` is still not a portable atomic replacement writer
`crates/rdocx/src/rtf.rs:118`
`crates/rdocx/src/rtf.rs:132`

The pass 3 remediation stages in the destination directory and uses exclusive
temporary creation, but the publish step is still plain `std::fs::rename` at
`crates/rdocx/src/rtf.rs:118`. On Windows that API does not replace an existing
destination, so `Document::save_rtf("existing.rtf")` returns an error instead
of atomically replacing the file. The same save path then unconditionally opens
the parent directory as a regular `File` and calls `sync_all` at
`crates/rdocx/src/rtf.rs:132`. On platforms where directory handles are not
opened this way, a new-file save can return an error after the rename has
already succeeded and after the temporary path is gone. That violates the
approved atomic path writer contract and leaves callers with either a failed
ordinary overwrite or an error result for a completed save.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 1, pass 2, and pass 3 verification

- Pass 1 D1 remains fixed. Table-cell paragraphs start with `\pard\intbl` at
  `crates/rdocx/src/rtf.rs:1021`, and each later cell paragraph is emitted
  through the reset helper.
- Pass 1 D2 remains fixed for the audited paragraph property set. The scanner
  covers public and parsed `CT_PPr` fields from style through revisions at
  `crates/rdocx/src/rtf.rs:611`.
- Pass 1 D3 remains fixed for horizontal grid spans. Physical output cells are
  produced by `table_row_cell_widths`, which sums spanned grid columns into one
  width at `crates/rdocx/src/rtf.rs:1400`.
- Pass 1 S1 remains fixed for output and diagnostic bounds. `BoundedOutput`
  checks each write at `crates/rdocx/src/rtf.rs:189`, picture hex expansion is
  preflighted at `crates/rdocx/src/rtf.rs:1383`, and writer diagnostics cap at
  `crates/rdocx/src/rtf.rs:1287`.
- Pass 2 D1 remains fixed for modelled table and row properties. The table and
  row property scanners cover `CT_TblPr` and `CT_TrPr` fields at
  `crates/rdocx/src/rtf.rs:329` and `crates/rdocx/src/rtf.rs:402`.
- Pass 2 D2 remains fixed for unsupported cell-width types, missing table
  grids, and short grids. Positive `dxa` widths are the only explicit widths
  accepted by `cell_width_twips` at `crates/rdocx/src/rtf.rs:1447`.
- Pass 2 D3 remains fixed for the audited run-property matrix. The scanner
  reports unsupported run properties per property at
  `crates/rdocx/src/rtf.rs:775`, while supported direct run formatting remains
  output by `write_run_format` at `crates/rdocx/src/rtf.rs:1102`.
- Pass 3 D1 is fixed. `CT_TcPr::extra_xml` is visited per retained item at
  `crates/rdocx/src/rtf.rs:509`, including duplicate raw positions through a
  stable suffix.
- Pass 3 D2 is fixed for the audited invalid and overflowing width cases.
  Nonpositive explicit `dxa` widths are diagnosed at
  `crates/rdocx/src/rtf.rs:460`, and cumulative `\cellx` overflow fails rather
  than saturating at `crates/rdocx/src/rtf.rs:976`.
- Pass 3 D3 is fixed for the audited list cases. Paragraph levels above 8 are
  diagnosed before list allocation at `crates/rdocx/src/rtf.rs:553`, and
  unsupported numbering formats stay diagnostic-only through
  `crates/rdocx/src/rtf.rs:1254` plus `crates/rdocx/src/rtf.rs:1073`.
- Pass 3 D4 is partially fixed. Serialization now completes before file I/O,
  staging uses same-directory exclusive creation, writes are synced before
  rename, and stale temporaries are cleaned up on pre-rename errors at
  `crates/rdocx/src/rtf.rs:96`. The remaining portability defect is recorded as
  D1 above.

## Not found

- No additional finding in deterministic font, colour, list table, and
  list-override ordering for supported inputs.
- No additional finding in Unicode scalar escaping, signed UTF-16 surrogate
  emission, brace escaping, backslash escaping, tabs, line breaks, or fixed
  fallback characters.
- No additional finding in positive and negative image goal dimension
  truncation, PNG and JPEG picture emission, missing image diagnostics,
  unsupported image diagnostics, or anchored drawing diagnostics.
- No additional finding in body order for supported paragraphs, tables, lists,
  images, and supported siblings next to dropped content.
- No additional finding in exact diagnostic coverage for the audited
  `CT_PPr`, `CT_RPr`, `CT_TblPr`, `CT_TrPr`, `CT_TcPr`, raw table-cell property
  XML, table raw XML, row raw XML, run raw XML, body raw XML, fields, footnotes,
  endnotes, comments, bookmarks, hyperlinks, and content controls.
- No additional finding in the output byte bound, picture hex bound, diagnostic
  cap, checked table grid-span summing, positive `dxa` handling, invalid width
  diagnostics, `\cellx` overflow failure, list level handling, or unsupported
  numbering format suppression.
- No finding in `to_rtf_bytes` mutating DOCX package state. The writer reads
  the typed document and uses package media lookup without calling the DOCX
  flush path.
- No in-scope published workspace exhaustive-match defect was found from the
  F-177 API addition. F-177 adds `RtfWriteResult`, `to_rtf_bytes`, and
  `save_rtf`, while the known `rdocx-py` exhaustive `Error::Rtf` note is tied
  to the earlier RTF error variant rather than this writer API.

## Checks

- `python3 scripts/prose_check.py`, passed.
- `git diff --check`, passed.
