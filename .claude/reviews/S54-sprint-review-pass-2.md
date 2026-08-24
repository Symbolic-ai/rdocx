# S54 sprint review, pass 2

**Reviewed**: `sprint/s54` at `9d2beb9d6470da169c4f545e2f5b43f11b8cfe9c`
against merge base `83633f83f53055fdf54cd212fbbe4d255cbdbef6`, 64 files and
12,350 changed lines, crates: `oxml-cli-support`, `oxml-layout`, `oxml-pdf`,
`rdocx`, `rdocx-cli`, `rdocx-layout`, `rdocx-py`, and `rpptx-cli`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass 1 remediation

- **B1 is resolved.** The named F-177 round-trip gate now adds a JPEG beside
  the PNG at `crates/rdocx/tests/integration_test.rs:183`, then verifies both
  reopened payloads and both goal-dimension pairs at
  `crates/rdocx/tests/integration_test.rs:260`. The exact named test passes.
- **B2 is resolved.** The shared PresentationML slide selector rejects an
  empty selection before either raster output branch at
  `crates/rpptx-cli/src/commands.rs:489`. The CLI regression exercises PNG,
  JPEG, and TIFF and verifies that no output directory is created at
  `crates/rpptx-cli/tests/integration.rs:473`. The exact named test passes.

## Milestone gate

The M18 gate is: "each format round-trips at its declared fidelity level, and
every lossy conversion records a diagnostic naming what it dropped"
(`docs/hld/14-development-backlog.md:1451`).

M18 remains intentionally open because F-178 through F-182 are pending. The
S54 slice now holds. The RTF reader differential passes against the pinned Word
16.104 structural record in
`rtf_reader_matches_the_pinned_word_docx_structure`
(`crates/rdocx/tests/integration_test.rs:1608`). The corrected writer gate
passes its text, formatting, table, list, PNG, and JPEG write-read comparison
in `rtf_writer_round_trip_preserves_supported_document_content`
(`crates/rdocx/tests/integration_test.rs:149`). Exact lossy diagnostic coverage
remains in `rtf_writer_reports_each_lossy_item_without_dropping_supported_siblings`
(`crates/rdocx/tests/integration_test.rs:1081`).

The shared raster regression passes for PNG, JPEG, multi-page TIFF, exact page
order, dimensions, and pixels in
`image_export_options_produce_the_declared_formats_and_exact_pages`
(`crates/oxml-pdf/src/raster.rs:1182`). The empty PresentationML selection
regression now passes for every declared raster format before publication
(`crates/rpptx-cli/tests/integration.rs:473`). The caller-font alias regression
passes exact byte, diagnostic, and provenance identity in
`document_facing_aliases_share_one_caller_font`
(`crates/rdocx/tests/regression_test.rs:5686`). The deterministic checks report
49 of 49 hash entries unchanged and 7 of 7 page-one golden pixel buffers
unchanged at 150 DPI.

## Not found

- `interaction`: B1 and B2 are fixed. No remaining conflict was found among
  the shared RTF reader and writer, caller-font aliases, reusable layout state,
  raster validation, Python bindings, or the two CLI output paths.
- `duplication`: reader and writer still share one private RTF module. Both
  CLIs use the shared range and staged-output helpers, and no competing
  sprint-local helper was found.
- `layering`: the `oxml-*` manifests contain no dependency on an `rdocx-*` or
  `rpptx-*` crate.
- `harness`: neither baseline file changed. The current hash and golden-PNG
  checks both pass with unchanged results.
- `gate`: the named differential, round-trip, raster, empty-selection, and
  font-alias regressions provide direct evidence for the completed S54 slice.
- `docs`: every plan-listed HLD file changed, and the architecture, rendering,
  bindings, testing, backlog, and packaging descriptions agree with the
  implemented ownership and fidelity boundaries.
- `deps`: `encoding_rs` is consumed by the RTF parser. `jpeg-encoder` and
  `tiff` are consumed by the shared raster backend with default features off.
  The CLI workspace edges serve their declared shared layout and raster paths.
- `surface`: every added RTF, raster, alias-aware layout, Python, and CLI entry
  point is required by an approved S54 story. No unrelated public surface was
  found.
- `delivery ledgers`: all four design plans, current-sprint rows, backlog rows,
  completion records, and tracker rows agree that the S54 stories are done and
  unowned.
