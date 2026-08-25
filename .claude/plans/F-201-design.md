# F-201, Large document performance

**Status**: approved
**Sprint**: S57
**Size**: L
**Depends on**: none

## Problem

`LayoutResult` retains every page and its positioned elements in
`crates/oxml-layout/src/output.rs:328`, while `Document` also caches the whole
deterministic result in `crates/rdocx/src/document.rs:1912`. Existing engine
tests prove retained-work cache bounds, but no end-to-end test measures a
thousand-page document's peak memory or throughput.

The largest facade pagination regression at
`crates/rdocx/src/document.rs:6688` covers only 20 pages and checks invocation
count rather than memory and time. The sprint therefore lacks numeric evidence
that both pagination and rendering remain bounded at the declared scale.

## Spec reference

- `docs/hld/03-architecture.md`, Word facade, layout ownership, and bounded engine state.
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap" and "Performance".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and "CI job matrix".
- `docs/hld/14-development-backlog.md`, "F-201, Large document performance".

## Approach

Add one ignored release-mode regression to the existing
`crates/rdocx/tests/regression_test.rs` integration binary. Source-build exactly
one thousand one-page paragraphs through `page_break_before`, then separately
measure `Document::layout_deterministic()` and direct PDF rendering from the
returned layout.

A test-only global allocator wrapper in that same binary counts live and peak
heap allocation only while the serial gate is active. The regression asserts
exactly one thousand pages, explicit peak-memory ceilings, explicit pages per
second floors for pagination and PDF rendering, and nonempty output. It runs as
an exact ignored release test with one test thread on the pinned Ubuntu 24.04
CI environment.

Add the exact release command as an unconditional CI step. Extend the existing
workflow regression suite to reject a missing, weakened, non-release, or
parallel invocation. If the first calibrated run fails, revise this design
before changing production code so the actual layout or PDF diff receives its
own risk routing and review.

## Rejected alternatives

- Add a benchmark crate or file. The existing integration binary can own the
  gate without another link target or dependency.
- Measure raster output for every page. PDF exercises the complete fixed-page
  renderer without retaining one encoded bitmap per page.
- Use an external memory package. A test-only allocator counter keeps the gate
  self-contained and measures the process heap directly.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_thousand_page_document_paginates_and_renders_within_the_declared_limits` | The source-built fixture produces 1000 pages within both memory ceilings and throughput floors |
| regression | workflow contract mutation tests | CI uses the exact locked release, ignored, single-thread gate and cannot swallow failure |

The **test gate** is regression. A thousand-page fixture paginates and renders
within the asserted ceiling and floor.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout and pagination. Read `docs/hld/08-rendering-spec.md`. Build the fixture
  with deterministic bundled fonts, keep all output baselines unchanged, and
  run the exact release regression as an additional consolidated check.

## Hash harness

Expected to be unchanged. The story adds a source-built test and measurement
gate without changing product output.

## Implementation checklist

- [ ] Approve PDF as the measured render backend and the calibration policy.
- [ ] Add inactive-by-default peak allocator accounting to the existing test binary.
- [ ] Build and verify the exact thousand-page deterministic fixture.
- [ ] Measure layout and PDF stages separately.
- [ ] Assert reviewed memory ceilings and throughput floors.
- [ ] Add the exact ignored release gate to CI.
- [ ] Add workflow mutation regressions.
- [ ] Run the focused gate, full verification, and hash harness.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved deterministic pagination plus PDF rendering and
conservative reviewed limits calibrated on the pinned Ubuntu 24.04 release
runner.
