# S54 sprint review, pass 1

**Reviewed**: `sprint/s54` at `35899847f834b22ac94eda3169c8e83e0affe721`
against merge base `83633f83f53055fdf54cd212fbbe4d255cbdbef6`, 63 files and
12,195 changed lines, crates: `oxml-cli-support`, `oxml-layout`, `oxml-pdf`,
`rdocx`, `rdocx-cli`, `rdocx-layout`, `rdocx-py`, and `rpptx-cli`
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the RTF round-trip gate does not exercise JPEG preservation

`crates/rdocx/tests/integration_test.rs:177`

The named F-177 gate adds only one PNG before writing and reading the document
back. Its image assertions then check only that one image's count and goal
dimensions at `crates/rdocx/tests/integration_test.rs:253`. The separate JPEG
test stops after matching `\jpegblip` and goal-dimension text in the serialized
RTF at `crates/rdocx/tests/integration_test.rs:275`. It never sends those bytes
through F-176. The documented gate promises normalized round-trip comparison
for both PNG and JPEG images at `docs/hld/12-testing-strategy.md:76`, so a
writer or reader defect specific to JPEG payloads can pass the declared gate.
Extend the named round-trip test with a JPEG and verify the reopened image kind,
payload, and dimensions alongside the PNG.

### B2, streamed PresentationML export accepts an empty page selection

`crates/rpptx-cli/src/commands.rs:320`

For a zero-slide presentation with no explicit range,
`selected_zero_based_slides` returns an empty vector at
`crates/rpptx-cli/src/commands.rs:489`. The TIFF branch delegates that vector to
the shared backend and gets `EmptyPageSelection`, but the streamed PNG and JPEG
branch creates the output directory, iterates zero times, publishes an empty
set, and returns success at `crates/rpptx-cli/src/commands.rs:343`. This bypasses
the F-183 rule that empty selections fail before output publication and makes
format choice change whether the same empty selection is accepted. Reject a
zero-slide or empty selected set before the format branches, and add the CLI
regression for PNG, JPEG, and TIFF with no output directory created.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M18 gate is: "each format round-trips at its declared fidelity level, and
every lossy conversion records a diagnostic naming what it dropped"
(`docs/hld/14-development-backlog.md:1451`).

M18 is intentionally still open. F-178 through F-182 remain pending, so this
sprint cannot establish the end gate for HTML, ODT, EPUB, or SVG. For the S54
slice, the checked RTF reader differential compares the source-built fixture
and generated-DOCX reopen against the pinned Word 16.104 structural record in
`rtf_reader_matches_the_pinned_word_docx_structure`
(`crates/rdocx/tests/integration_test.rs:1592`). The writer performs the named
text, formatting, table, list, and PNG round trip, but B1 leaves the declared
JPEG portion untested. Stable lossy-reader and lossy-writer diagnostics have
exact assertions at `crates/rdocx/tests/integration_test.rs:1638` and
`crates/rdocx/tests/integration_test.rs:1065`.

The shared raster gate decodes PNG, JPEG, and multi-page TIFF for selected
pages 2 and 0 in caller order at `crates/oxml-pdf/src/raster.rs:1182`. B2 shows
that one streamed CLI path bypasses the shared empty-selection validation. The
font-alias regression proves two document-facing aliases share the exact caller
font bytes and retain provenance at
`crates/rdocx/tests/regression_test.rs:5686`. The current hash harness reports
49 of 49 entries unchanged, and the deterministic golden-PNG gate reports 7 of
7 page-one pixel buffers unchanged at 150 DPI.

The S54 slice of the milestone gate therefore does not yet hold because B1 and
B2 leave required behavior outside the effective gates.

## Not found

- `interaction`: apart from B1's reader-writer JPEG boundary and B2's streamed
  CLI bypass, no conflict was found among RTF projection, reusable font aliases,
  deterministic layout, and shared raster encoding.
- `duplication`: the reader and writer share one private RTF module, both CLIs
  share range and staged-output helpers, and no competing sprint-local helper
  was found.
- `layering`: `cargo metadata --no-deps` reports no `oxml-*` dependency on an
  `rdocx-*` or `rpptx-*` crate.
- `harness`: neither baseline file changed. The current 49-entry hash check and
  7-buffer deterministic golden-PNG check both pass.
- `gate`: apart from B1 and B2, the named reader differential, raster option,
  alias identity, malformed-input, diagnostic, bounds, warm-cold, CLI, Python,
  package, WASM, and workspace gates provide evidence for the S54 contract.
- `docs`: every plan-listed HLD file was updated, and the architecture,
  rendering, binding, testing, backlog, and build descriptions agree with the
  implemented ownership boundaries.
- `deps`: `encoding_rs` is the direct RTF code-page consumer in `rdocx`.
  `jpeg-encoder` and `tiff` are direct `oxml-pdf` consumers with default
  features disabled. No unexplained dependency or format-family edge was
  added.
- `surface`: the RTF result types and methods, raster option types and methods,
  alias-aware layout trio, Python image method, and CLI flags belong to the
  approved feature contracts. No unrelated public surface was found.
- `delivery ledgers`: all four plans, current-sprint rows, backlog rows,
  completion entries, tracker rows, and run-state feature records agree on
  completed S54 ownership and dates. The exact final-HEAD verification remains
  a later `/run-sprint` step after the sprint review record lands.
