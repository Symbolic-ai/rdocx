# F-144, all, pass 2

**Reviewed**: uncommitted `work/f-144-codex` implementation, 15 files and 1,067
changed lines, including pass 1 remediation and the four approved new CLI crate
paths
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the corpus remediation still leaves the clean MSRV job failing

`.github/workflows/ci.yml:280`

The primary test job now fetches the pinned presentation corpus before its
workspace test, but the clean MSRV job also runs
`cargo test --workspace --all-features` and has no fetch step. That job reaches
the same unconditional corpus assertion at
`crates/rpptx-cli/tests/integration.rs:88`, so all 50 decks are missing in its
fresh checkout. The new workflow regression only inspects the primary test job.
Fetch and verify the pinned corpus before every CI workspace test that includes
this gate, and make the structural regression cover those jobs.

### D2, PNG conversion of a valid zero-slide deck succeeds without an artifact

`crates/rpptx-cli/src/commands.rs:147`

A presentation with zero slides is valid and is the repository's bundled
template shape, but `write_converted_pngs` treats an empty page list as a
successful multi-page conversion. The loop writes nothing, the command exits
zero, and no requested output exists. A focused run against
`crates/rpptx/assets/default.pptx` reproduced exit 0 with no stdout and no PNG.
Reject zero-page PNG conversion explicitly, or define and implement an output
artifact for that valid input, then cover the command result and filesystem
postcondition.

## Smells

None.

## Nitpicks

None.

## Pass 1 re-evaluation

- **D1, clean CI corpus setup**: partially resolved. The primary test job now
  fetches before testing, but the MSRV workspace test remains uncovered as
  described above.
- **D2, unbounded raster allocation**: resolved. Convert validates every page
  and render validates every selected page against an 8,000,000 pixel budget
  before rasterization. Both boundary regressions passed.
- **D3, missing plain inspect metadata**: resolved. The plain branch reports
  all eight modeled core-property fields, and the populated metadata command
  regression passed.

## Not found

- **Public replacement semantics**: literal matches remain snapshotted and
  processed in reverse. Same-run and cross-run replacement preserves the first
  run's formatting and the unmatched suffix's formatting. Breaks, fields, and
  alternate content remain boundaries. Groups and table cells are traversed.
- **Package preservation**: the CLI round-trip replacement regression passed
  and retains distinct run properties plus an opaque package part.
- **CLI paths**: exactly the seven approved F-144 commands remain present.
  Shared JSON, output, and one-based range contracts are used, and thumbnail
  and outline remain absent. Apart from zero-page PNG conversion, no command
  dispatch, output naming, LCS, selection, or validation-exit defect was found.
- **Correctness and panics**: derived raster dimensions are checked before
  allocation. Replacement offsets remain valid UTF-8 byte boundaries, and the
  internal `expect` and `unreachable` sites are protected by local invariants.
- **Tests**: all six corpus-independent CLI integration tests passed, including
  both raster limits, deterministic outputs, inspect metadata, replacement,
  diff, and render. The focused facade replacement matrix also passed. The
  corpus gate was not rerun because its clean-CI setup remains defective.
- **Structure and dependencies**: no extra crate, module, test binary, trait,
  wrapper, generic, or forbidden reverse dependency was added.
- **Publication and harness**: release inventory changes remain metadata and
  dry-run preparation only. No tag or publication action is introduced, and no
  rendering baseline or hash-harness behavior changes.
- **Hygiene**: `git diff --check` passed before the review record was written.
