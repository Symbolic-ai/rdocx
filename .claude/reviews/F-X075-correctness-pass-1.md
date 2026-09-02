# F-X075, correctness, pass 1

**Reviewed**: complete working-tree diff against
`d0318ce0d7d9f7110fa4e03a154255593ea98263`, including the approved plan,
progress record, checkpoint and restart implementation, changed regressions,
the ignored release benchmark, routed instructions, and HLD 08, 12, and 14.
The delta is 7 files with 573 changed lines, 422 additions and 151 deletions.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the page-spanning exactness gate does not compare the complete result
`crates/rdocx-layout/src/engine.rs:10743`
`crates/rdocx-layout/src/engine.rs:11043`
`docs/hld/12-testing-strategy.md:553`

The new page-spanning warm tests use `Engine::layout` and the shared
`assert_layout_results_equal` helper. That helper compares pages, fonts,
diagnostics, and outlines, but omits both `LayoutResult::metadata` and
`LayoutResult::structure`. The new cases also never run
`layout_with_provenance`, so they cannot compare the result-local source map
for the split-paragraph path that now publishes and reuses restart state. A
regression in logical structure or source rebinding can therefore pass every
new warm-to-fresh assertion even though the HLD requires complete equality,
including structure and provenance. Add a page-spanning sourced warm case and
compare every `LayoutResult` field plus the provenance map exactly.

### D2, the historical performance comparison is not pinned by its harness
`crates/rdocx/tests/regression_test.rs:217`
`.claude/scratch/F-X075-progress.md:47`
`.claude/plans/F-X075-design.md:104`

The ignored benchmark validates only paragraph count and layout mode. It does
not identify or reject the Git revision being measured, and its output carries
no checkout identity. The v0.11.1 and `0582da0` identities exist only as prose
in the progress record. This permits an invocation from the wrong historical
checkout to produce indistinguishable evidence, contrary to the routed rule
that an external comparison pin belongs in the test harness. The progress
record also keeps only the final median of four per-round medians, not the four
inputs or a variance classification, so the reported aggregate cannot be
independently recomputed. Bind each run to an expected exact HEAD, include that
identity in its output, and retain the per-round medians and variance
classification outside the published crate.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Checkpoint safety** produced zero findings. Split continuations finalize
  pages only through `finish_page` at
  `crates/rdocx-layout/src/paginator.rs:2194`. Checkpoints remain confined to
  `finish_page_before`, after notes, wraps, and resolved state are empty, at
  `crates/rdocx-layout/src/paginator.rs:1189`. The exact 175-paragraph fixture
  pins the first post-split complete boundary and all later boundaries at
  `crates/rdocx-layout/src/engine.rs:11021`.
- **Restart correctness and bounds** produced zero findings beyond D1. The
  existing source and rendered-block safety predicates still gate recorded
  pagination at `crates/rdocx-layout/src/engine.rs:1390`. Candidate size uses
  the existing aggregate admission check and publishes or clears the whole
  restart record at `crates/rdocx-layout/src/engine.rs:1852`.
- **Warm and fresh behavior** produced zero production findings. The ten
  deterministic middle edits assert 174 cache hits, one rebuild, at most two
  page invocations, a two-page rebuilt range, and page, font, diagnostic, and
  outline equality at `crates/rdocx-layout/src/engine.rs:11048`. Late edit,
  insert, delete, and undo exercise the same split-producing source at
  `crates/rdocx-layout/src/engine.rs:11078`. D1 is the remaining completeness
  gap in those comparisons.
- **Notes and fields** produced zero findings beyond D1. The combined
  note-bearing split and displayed PAGE-footer case verifies the first retained
  boundary is after the split paragraph, checks every displayed page number,
  and compares warm output to fresh output at
  `crates/rdocx-layout/src/engine.rs:11123`. The preexisting long-footnote test
  still rejects its dirty continuation boundary at
  `crates/rdocx-layout/src/engine.rs:11786`.
- **Unsafe exclusions** produced zero findings. Numbering, fields, drawings,
  raw XML, and multilingual state remain covered by the no-checkpoint matrix at
  `crates/rdocx-layout/src/engine.rs:11369`. Unsafe tables, floating drawings,
  note-bearing tables, multiple sections, backgrounds, and field checkpoints
  remain covered at `crates/rdocx-layout/src/engine.rs:12268`. The diff removes
  only the obsolete blanket rejection of every split paragraph.
- **Benchmark construction** produced zero findings beyond D2. The ignored
  test builds both approved source sizes in code, primes the reusable engine,
  times ten warm edits outside mutation work, supports native and deterministic
  bundled-fallback paths, and imposes no normal-test wall-clock threshold at
  `crates/rdocx/tests/regression_test.rs:215`.
- **Regression sensitivity** produced zero findings beyond D1 and D2. Reverting
  the production change restores the split veto and makes the new fixture fail
  its required retained restart state at
  `crates/rdocx-layout/src/engine.rs:11017`. The exact page count, four-line
  paragraph shape, invocation count, cache counts, rebuilt-range bounds, and
  checkpoint vector prevent a false pass through a nonrepresentative fixture.
- **Public surface, dependencies, and structure** produced zero findings. The
  production delta removes one private recorded-pass flag and one private
  fallback branch. No manifest, public facade, dependency, feature, crate,
  module, file, trait, or generic changes are present. The plan preserves that
  boundary at `.claude/plans/F-X075-design.md:47`.
- **Panics and arithmetic** produced zero findings. The production delta adds no
  unwrap, expect, indexing, slicing, unchecked arithmetic, or new input-driven
  allocation. New panics and direct indexing are confined to source-built test
  fixtures at `crates/rdocx-layout/src/engine.rs:10987`.
- **HLD scope and output stability** produced zero findings. The HLD diff is
  exactly the three files approved at `.claude/plans/F-X075-design.md:93` and
  consistently narrows the old split exclusion. The progress evidence records
  both Word WASM checks and an unchanged 49-entry hash harness at
  `.claude/scratch/F-X075-progress.md:84`.
