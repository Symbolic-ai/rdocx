# F-177, all, pass 5

**Reviewed**: cumulative working tree diff from base `e7fee7c` across `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `crates/rdocx/src/rtf.rs`, and `crates/rdocx/tests/integration_test.rs`, with 2,638 insertions and 17 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks
**Counts**: defects 0, smells 0, nitpicks 0

## Defects

None. Count: 0.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Prior finding verification

- Pass 1 D1 remains fixed. Table-cell paragraphs start with `\pard\intbl` before paragraph formatting at `crates/rdocx/src/rtf.rs:1015`, and table cells route each paragraph through that helper at `crates/rdocx/src/rtf.rs:982`.
- Pass 1 D2 remains fixed. Paragraph property diagnostics cover unsupported public and parsed `CT_PPr` fields from style through property revisions at `crates/rdocx/src/rtf.rs:605`.
- Pass 1 D3 remains fixed. `table_row_cell_widths` emits one width per physical row cell, sums grid spans with checked addition, and falls back only after the diagnostic scan has run at `crates/rdocx/src/rtf.rs:1394`.
- Pass 1 S1 remains fixed. `BoundedOutput` checks every write before appending at `crates/rdocx/src/rtf.rs:183`, picture hex expansion preflights the doubled size at `crates/rdocx/src/rtf.rs:1377`, and writer diagnostics stop at `MAX_DIAGNOSTICS` at `crates/rdocx/src/rtf.rs:1281`.
- Pass 2 D1 remains fixed. Table and row property scanners cover the modelled `CT_TblPr` and `CT_TrPr` fields at `crates/rdocx/src/rtf.rs:323` and `crates/rdocx/src/rtf.rs:396`.
- Pass 2 D2 remains fixed. Cell widths diagnose unsupported or invalid explicit widths, missing grids, and short grids at `crates/rdocx/src/rtf.rs:447` and `crates/rdocx/src/rtf.rs:522`.
- Pass 2 D3 remains fixed. Run property diagnostics are per property for unsupported fonts, complex-script variants, underline simplification, double strike, theme colour, keyword highlight, spacing, width scale, position, and revisions at `crates/rdocx/src/rtf.rs:769`.
- Pass 3 D1 remains fixed. Retained raw table-cell property XML is visited per item with duplicate-position suffixing at `crates/rdocx/src/rtf.rs:503`.
- Pass 3 D2 remains fixed. Positive `dxa` is the only accepted explicit cell width at `crates/rdocx/src/rtf.rs:1441`, and cumulative `\cellx` overflow fails through checked addition at `crates/rdocx/src/rtf.rs:968`.
- Pass 3 D3 remains fixed. Numbering levels above 8 are diagnosed and not emitted at `crates/rdocx/src/rtf.rs:547`, while unsupported numbering formats get diagnostic-only list levels at `crates/rdocx/src/rtf.rs:1248` and are suppressed during paragraph emission at `crates/rdocx/src/rtf.rs:1067`.
- Pass 3 D4 and pass 4 D1 are fixed. `save_rtf` serializes before file I/O at `crates/rdocx/src/rtf.rs:89`, stages with exclusive create in the destination directory at `crates/rdocx/src/rtf.rs:96`, syncs the temporary file before publish at `crates/rdocx/src/rtf.rs:115`, calls the shared `document::replace_file` helper at `crates/rdocx/src/rtf.rs:117`, and only removes the temporary path on an error at `crates/rdocx/src/rtf.rs:118`.

## Replacement and platform audit

- The shared helper is now crate-visible and no longer gated by `agile-encryption`, so the RTF writer can call it in normal builds at `crates/rdocx/src/document.rs:4447` and `crates/rdocx/src/document.rs:4452`.
- The non-Windows helper keeps the existing same-directory `std::fs::rename` behavior at `crates/rdocx/src/document.rs:4447`.
- The Windows helper uses `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` and `MOVEFILE_WRITE_THROUGH` at `crates/rdocx/src/document.rs:4456` and passes those flags to the platform call at `crates/rdocx/src/document.rs:4475`.
- No post-rename false error path remains in `save_rtf`. After the helper returns, the function returns that result directly and does not open or sync the parent directory at `crates/rdocx/src/rtf.rs:117`.
- Temporary name collisions are bounded and fail closed after 128 occupied names, and the existing destination is preserved by the regression at `crates/rdocx/src/lib.rs:169`.
- Encrypted save still uses the same staged write and shared helper path at `crates/rdocx/src/document.rs:4413` and `crates/rdocx/src/document.rs:4435`. The feature test covers failed encrypted save preserving the old destination and successful replacement at `crates/rdocx/src/document.rs:5137`.

## Contract audit

- The public writer result and facade methods match the design API at `.claude/plans/F-177-design.md:46` and are exposed through `rdocx` at `crates/rdocx/src/rtf.rs:44`, `crates/rdocx/src/rtf.rs:83`, and `crates/rdocx/src/lib.rs:62`.
- Header tables are emitted before body content through the scan-then-write order at `crates/rdocx/src/rtf.rs:227`, and the stable header regression asserts font, colour, list, and body ordering at `crates/rdocx/tests/integration_test.rs:113`.
- Formatting resets, Unicode escaping, list identity, package image lookup, and truncating image goal dimensions match the design requirements at `.claude/plans/F-177-design.md:38`. The implementation sites are `crates/rdocx/src/rtf.rs:996`, `crates/rdocx/src/rtf.rs:1074`, `crates/rdocx/src/rtf.rs:1171`, `crates/rdocx/src/rtf.rs:1232`, and `crates/rdocx/src/rtf.rs:1357`.
- Unsupported body, table, row, cell, paragraph, run, image, field, note, comment, bookmark, hyperlink, and raw XML cases produce stable diagnostics while supported siblings continue at `crates/rdocx/src/rtf.rs:249`, `crates/rdocx/src/rtf.rs:264`, `crates/rdocx/src/rtf.rs:546`, and `crates/rdocx/src/rtf.rs:717`.
- The round-trip gate in the backlog requires text, formatting, tables, lists, and images to survive write-read at `docs/hld/14-development-backlog.md:1467`. The implemented gate exercises that path at `crates/rdocx/tests/integration_test.rs:149`.
- The RTF writer reads from the typed document and package media lookup without flushing or mutating the DOCX package. The unmodelled XML preservation regression proves the before and after DOCX XML stays identical at `crates/rdocx/tests/integration_test.rs:1122`.

## Not found

- No remaining shared `replace_file` cfg or visibility defect.
- No remaining Windows replacement defect.
- No remaining non-Windows behavior defect.
- No remaining post-rename false error defect.
- No remaining temporary cleanup or collision defect.
- No remaining encrypted-save regression.
- No remaining wasm compile defect.
- No remaining complete-writer correctness defect in the audited RTF subset.
- No remaining diagnostic coverage or bounds defect in the audited modelled fields.
- No remaining round-trip gate defect.

## Checks

- `python3 scripts/prose_check.py`, passed.
- `git diff --check`, passed.
- `cargo test -p rdocx save_rtf_`, passed.
- `cargo test -p rdocx rtf_writer_`, passed.
- `cargo test -p rdocx --test integration_test rtf`, passed with 26 passed, 1 ignored.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`, passed.
- `cargo test -p rdocx --features agile-encryption native_encrypted_save_round_trips_without_live_package_mutation`, passed.
