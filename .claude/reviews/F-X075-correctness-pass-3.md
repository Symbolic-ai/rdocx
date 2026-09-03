# F-X075, correctness, pass 3

**Reviewed**: current complete working-tree diff against
`d0318ce0d7d9f7110fa4e03a154255593ea98263`, pass 2, current progress,
source-content manifest construction and sensitivity evidence, production
restart and checkpoint paths, all changed tests, the release measurements, and
HLD 08, 12, and 14. The implementation delta is 7 files with 817 changed
lines, 661 additions and 156 deletions.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, self-pin normalization hides same-line harness mutations
`crates/rdocx/tests/regression_test.rs:331`
`.claude/scratch/F-X075-progress.md:211`
`.claude/plans/F-X075-design.md:107`

The exact-harness manifest replaces the entire source line whenever its
trimmed prefix matches the self-pin declaration. It does not replace only the
recursive hash literal or require the declaration to have its exact expected
shape. Appending a comment, attribute, or another valid Rust item after the
self-pin semicolon therefore leaves the computed harness manifest unchanged.
For example, a second module-level constant on that line is compiled but is
discarded completely by the normalizer. The recorded negative harness mutation
used a separate marked-harness comment, so it did not exercise this bypass.
This falls short of the approved exact injected-harness identity. Normalize
only the single hash value in one exact declaration, preserve every other byte
on that line, assert exactly one replacement, and add same-line suffix and
duplicate-candidate sensitivity cases.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 2 source-content closure** produced zero findings beyond D1. The
  production manifest sorts every tracked workspace manifest and crate path,
  hashes current working bytes, and includes both object identity and path at
  `crates/rdocx/tests/regression_test.rs:290`. Ordinary untracked crate source
  is rejected before manifest construction at
  `crates/rdocx/tests/regression_test.rs:271`.
- **Current and historical identity** produced zero findings beyond D1. The
  current label is content-bound and therefore survives committing the same
  reviewed bytes. Both historical labels additionally require exact immutable
  HEADs before source comparison at
  `crates/rdocx/tests/regression_test.rs:353`. The regression source outside
  the marked injection is independently hashed at
  `crates/rdocx/tests/regression_test.rs:317`.
- **Manifest sensitivity** produced zero findings beyond D1. A production
  source mutation and an ordinary marked-harness mutation both reached and
  failed the pre-timing manifest predicate, with the changed identities
  recorded at `.claude/scratch/F-X075-progress.md:211`. D1 is confined to the
  overbroad self-pin normalization boundary those cases did not mutate.
- **Per-round timing evidence** produced zero findings. All four round medians
  for each size, path, and revision are retained, the middle-median aggregates
  and ratios recompute correctly, and worst-round comparisons remain inside
  both approved limits at `.claude/scratch/F-X075-progress.md:218`. Each
  measurement prints checkout, HEAD, all three manifests, fixture size, mode,
  pages, median, and sorted samples at
  `crates/rdocx/tests/regression_test.rs:396` and
  `crates/rdocx/tests/regression_test.rs:469`.
- **Complete result and provenance exactness** produced zero findings. The
  equality helper covers every metadata field, logical structure, and the
  complete debug representation at
  `crates/rdocx-layout/src/engine.rs:10743`. Ten sourced page-spanning edits
  compare the exact warm and fresh result plus result-local source maps at
  `crates/rdocx-layout/src/engine.rs:11063`.
- **Checkpoint safety** produced zero findings. Recorded pagination remains
  behind the unchanged source and rendered-block safety predicates at
  `crates/rdocx-layout/src/engine.rs:1390`. Split continuations finish only
  through `finish_page` at `crates/rdocx-layout/src/paginator.rs:2194`, while
  checkpoints remain confined to clean, drained complete-block boundaries at
  `crates/rdocx-layout/src/paginator.rs:1189`.
- **Restart assembly, bounds, and publication** produced zero findings. Prefix,
  rebuilt, and exact tail pages retain the existing checkpoint and context
  conditions at `crates/rdocx-layout/src/engine.rs:1467`. Candidate accounting
  and whole-record replacement remain checked and transactional at
  `crates/rdocx-layout/src/engine.rs:1852`.
- **Functional regression sensitivity** produced zero findings. The exact
  175-paragraph fixture pins 16 pages, four lines per paragraph, one recorded
  pass, retained state, and every complete-boundary checkpoint at
  `crates/rdocx-layout/src/engine.rs:11017`. Reinstating the removed split veto
  fails that retained-state assertion. Late edit, insert, delete, undo, note,
  PAGE footer, and bounded warm work have direct source-built coverage.
- **Unsafe exclusions** produced zero findings. Numbering, drawings, raw XML,
  fields, unsafe tables, wraps, note-bearing tables, backgrounds, multiple
  sections, and dirty note continuation remain rejected or checkpoint-free at
  `crates/rdocx-layout/src/engine.rs:11394`,
  `crates/rdocx-layout/src/engine.rs:11786`, and
  `crates/rdocx-layout/src/engine.rs:12293`.
- **Public surface, dependencies, and structure** produced zero findings. The
  production change removes one private recorded-pass flag and its private
  fallback branch. No manifest, public API, dependency, feature, crate, module,
  file, trait, or generic is added. The approved private boundary remains at
  `.claude/plans/F-X075-design.md:47`.
- **Panics and arithmetic** produced zero findings. Production adds no unwrap,
  expect, direct indexing, slicing, unchecked arithmetic, or input-driven
  allocation. New command execution and assertions are confined to the ignored
  benchmark, where malformed repository state fails before timing at
  `crates/rdocx/tests/regression_test.rs:218`.
- **HLD scope and output stability** produced zero findings. The HLD diff
  remains exactly the approved 08, 12, and 14 files at
  `.claude/plans/F-X075-design.md:94`. HLD 12 accurately describes the
  content-bound current identity and separately pinned historical commits at
  `docs/hld/12-testing-strategy.md:579`. Current progress records an unchanged
  49-entry hash harness and both consuming Word WASM checks at
  `.claude/scratch/F-X075-progress.md:84`.
