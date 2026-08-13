# F-144, all, pass 5

**Reviewed**: uncommitted `work/f-144-codex` implementation, 15 files and 1,160
changed lines, including all prior remediation and the four approved new CLI
crate paths
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Prior finding re-evaluation

- **Pass 1 D1 and pass 2 D1, clean CI corpus setup**: resolved. Both workspace
  test jobs fetch the verified corpus before testing, and the structural
  regression checks both jobs and step order.
- **Pass 1 D2, unbounded single-page raster allocation**: resolved. Convert
  validates every page and render validates every selected page against the
  pixel budget before rasterization. Both rejection regressions remain green.
- **Pass 1 D3, missing plain inspect metadata**: resolved. Plain inspect emits
  all eight modeled core-property fields, with populated command coverage.
- **Pass 2 D2, zero-slide PNG conversion**: resolved. The valid zero-slide
  input fails before layout or output creation, and the regression checks exit
  status, stdout, and filesystem state.
- **Pass 3 D1, unbounded quadratic LCS matrix**: resolved. Row, column, and cell
  arithmetic is checked before allocation. The 1,002,001-cell case is rejected
  without partial stdout, while accepted LCS output retains its ordering.
- **Pass 4 D1, aggregate PNG retention**: resolved. Convert preflights page
  count and every page dimension, then encodes, writes, and drops one page at a
  time. Single-page output and one-based multi-page suffixes remain unchanged.

## Boundary and sensitivity evidence

- Raising the LCS limit admitted the crafted case and failed its exact
  regression before byte-identical restoration.
- Restoring aggregate PNG rendering failed the 24-slide streaming regression.
  The source was restored byte-identically, and streaming plus existing naming
  and zero-slide regressions reran green.
- Raising the raster limit, removing either CI corpus fetch, bypassing the
  zero-slide guard, forcing validation success, and skipping cross-run
  replacement each retain recorded negative evidence and exact restoration.

## Not found

- **Correctness and contract**: exactly the seven approved F-144 commands are
  implemented through public facades. Inspect, text, deterministic conversion,
  LCS diff, literal replacement, validation exit status, one-based range
  rendering, output defaults, and schema-1 JSON match the approved contract.
  Thumbnail and outline remain absent for F-145.
- **Public replacement semantics**: literal matches are snapshotted and applied
  in reverse without recursive replacement. Same-run and cross-run matches
  retain first-run formatting and unmatched suffix formatting. Breaks, fields,
  and alternate content remain boundaries. Nested groups and tables are
  traversed.
- **Package preservation and OOXML**: typed run edits retain raw run state and
  opaque package parts. No CLI path reads or writes raw PresentationML, changes
  schema child order, or mutates selected alternate-content branches.
- **Panics and resource bounds**: indexing and slicing sites are protected by
  local invariants. DPI, raster dimensions, LCS arithmetic, LCS cell count, and
  aggregate PNG retention are bounded before the relevant allocations.
- **Tests**: all ten CLI integration tests passed with the verified external
  50-deck corpus. The focused facade replacement matrix also passed. Named
  boundary tests cover single-page raster size, zero-slide conversion, LCS
  cells, and streaming multi-slide PNG output.
- **Structure and dependencies**: the implementation uses only the four
  approved new paths, one command module, and one test entrypoint. It adds no
  trait, forwarding wrapper, generic, feature, test binary, or forbidden
  reverse dependency.
- **Publication and harness**: release edits expand eligibility, allowlists,
  workflow ordering, and dry-run preparation only. They create no tag and start
  no publication. No sample generator, rendering default, baseline, or hash
  harness behavior changes.
- **Hygiene**: prose validation, generated-skill drift, and `git diff --check`
  passed after the focused tests.
