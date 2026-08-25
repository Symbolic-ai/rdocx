# S57 sprint review, pass 1

**Reviewed**: `sprint/s57` against
`89d0f28435395d37adcd4231ac185d6208998e82`, 23 files, 2,513 changed lines,
crates: `rdocx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

That end-of-milestone gate does not yet hold. The reviewed S57 calibration at
`docs/hld/12-testing-strategy.md:662` records 18 union pages with only one page
at or above 0.95 SSIM, for 5.56 percent coverage against the 80 percent trend
reference. The shaping work remains assigned to F-198 through F-200 at
`docs/hld/14-development-backlog.md:1835`. This is not an S57 blocker because
S57 establishes the corpus, evidence boundary, and performance baseline before
those S58 stories, as stated at `docs/sprints/CURRENT_SPRINT.md:5`.

The S57 sprint definition of done holds. The exact performance regression
passed locally with 1,000 pages, a 22.32 MiB layout peak, a 1.73 MiB additional
PDF peak, 93,944 layout pages per second, and 83,197 PDF pages per second. The
17-test SSIM suite passed its 16 self-contained tests with the expected
full-corpus evidence test skipped outside a live gate. Six focused corpus and
CI mutation regressions passed. The deterministic hash harness reports 49 of
49 entries unchanged. Diff checking, prose checking, and generated-skill drift
checking also pass. The retained integrated live evidence records the complete
five-document, 18-union-page run at `docs/sprints/AS_BUILT.md:9389`.

## Not found

- **Interaction**: F-197 consumes F-196's exact manifest and verified corpus.
  Its accepted-view oracle preparation matches the facade's default accepted
  render projection. F-201 measures the same deterministic layout and shared
  PDF boundary without changing either production path.
- **Duplication**: no helper was independently added twice in S57. The Word
  harness imports the existing strict PNG decoder and SSIM implementation at
  `scripts/docx_ssim_harness.py:19`.
- **Layering**: no Cargo manifest, lockfile, or `oxml-*` file changed. No reverse
  format dependency entered the workspace graph.
- **Harness**: all three feature records declare an unchanged 49-entry hash
  harness at `docs/sprints/AS_BUILT.md:9313`,
  `docs/sprints/AS_BUILT.md:9350`, and
  `docs/sprints/AS_BUILT.md:9396`. The integrated check reproduces that result.
- **Gate**: corpus mismatch, tool drift, renderer and oracle failure, zero
  output, and missing evidence fail closed. The exact ignored release
  performance test and the filtered Word fidelity job propagate failures into
  CI.
- **Docs**: the integrated delta updates exactly the planned HLD impact set,
  `docs/hld/08-rendering-spec.md`, `docs/hld/12-testing-strategy.md`, and
  `docs/hld/15-build-and-toolchain.md`. The recorded limits, oracle identity,
  normalization policy, and calibration agree with the implementation.
- **Deps**: no dependency was added or changed.
- **Surface**: no production public API was added. The only crate change is the
  approved ignored regression in the existing `rdocx` integration binary.
