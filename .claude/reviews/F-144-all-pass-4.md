# F-144, all, pass 4

**Reviewed**: uncommitted `work/f-144-codex` implementation, 15 files and 1,132
changed lines, including all prior remediation and the four approved new CLI
crate paths
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, PNG conversion retains every encoded slide in memory

`crates/rpptx-cli/src/commands.rs:135`

The per-page pixel checks bound one raster allocation, but convert then calls
`render_all_pages`, which encodes and collects every slide into a
`Vec<Vec<u8>>` before writing the first output. A valid deck can reference one
shared high-entropy image from hundreds of slides, keeping the PPTX relatively
small while producing hundreds of near-budget PNG buffers. Their aggregate can
exhaust memory even though every page passes the 8,000,000 pixel check. Render
and write one page at a time after the complete preflight, or enforce a checked
aggregate budget before encoding, and add a multi-slide boundary regression
that proves encoded pages are not accumulated without a bound.

## Smells

None.

## Nitpicks

None.

## Prior finding re-evaluation

- **Pass 1 D1 and pass 2 D1, clean CI corpus setup**: resolved. Both workspace
  test jobs fetch the verified corpus first, and the structural regression
  checks both jobs and their step order.
- **Pass 1 D2, unbounded single-page raster allocation**: resolved. Convert and
  render reject page dimensions above the pixel budget before rasterization.
  Aggregate PNG retention remains the distinct D1 above.
- **Pass 1 D3, missing plain inspect metadata**: resolved. All eight modeled
  fields are emitted and covered with populated metadata.
- **Pass 2 D2, zero-slide PNG conversion**: resolved. The command fails before
  layout or output creation and the regression checks exit status, stdout, and
  filesystem state.
- **Pass 3 D1, unbounded quadratic LCS matrix**: resolved. Row, column, and cell
  arithmetic is checked before allocation, and matrices above 1,000,000 cells
  are rejected without partial stdout. The 1,002,001-cell regression crosses
  the boundary, while the ordinary diff regression retains accepted behavior.

## Boundary and sensitivity evidence

- The 1,000-slide repeated-text case crosses the LCS boundary by 2,001 cells.
  Raising the limit to 1,100,000 admitted the case and failed the regression,
  after which the source was recorded as restored byte-identically.
- Raising the raster limit admitted both oversized convert and render cases and
  failed their regressions. Restored tests pass at the intended boundary.
- Removing the MSRV fetch and bypassing the zero-slide guard each failed its
  exact regression before byte-identical restoration.
- Validation failure and cross-run replacement each retain their recorded
  negative mutation evidence.

## Not found

- **Public replacement semantics**: reverse snapshot mutation remains correct
  for same-run and cross-run matches, including UTF-8 byte boundaries. Breaks,
  fields, and alternate content remain boundaries. Groups and table cells are
  traversed.
- **Package preservation and OOXML**: replacement retains first-run formatting,
  unmatched suffix formatting, raw run state, and opaque package parts. No CLI
  path reads or writes raw PresentationML directly.
- **Other CLI correctness**: exactly the seven approved F-144 commands remain.
  Shared JSON, output, and one-based range contracts are used. Apart from PNG
  aggregate memory, no fresh command dispatch, output naming, diff ordering,
  replacement, validation-exit, or render-selection defect was found.
- **Panics**: indexing and slicing sites are protected by local loop or match
  invariants. LCS arithmetic and per-page raster dimensions are checked before
  allocation.
- **Tests**: all nine CLI integration tests passed with the verified external
  50-deck corpus. The focused facade replacement matrix also passed.
- **Structure and dependencies**: no extra crate, module, test binary, trait,
  wrapper, generic, feature, or forbidden reverse dependency was added.
- **Publication and harness**: release changes remain eligibility, allowlist,
  workflow, and dry-run preparation only. No tag or publication action is
  introduced, and no rendering baseline or hash-harness behavior changes.
- **Hygiene**: `git diff --check` passed before this review record was written.
