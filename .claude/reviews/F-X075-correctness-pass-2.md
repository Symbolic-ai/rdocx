# F-X075, correctness, pass 2

**Reviewed**: current complete working-tree diff against
`d0318ce0d7d9f7110fa4e03a154255593ea98263`, pass 1, the remediation progress
record, production restart and checkpoint paths, all changed tests, the
release benchmark and evidence, and HLD 08, 12, and 14. The implementation
delta is 7 files with 637 changed lines, 483 additions and 154 deletions.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the SHA check does not authenticate the source tree being benchmarked
`crates/rdocx/tests/regression_test.rs:218`
`.claude/scratch/F-X075-progress.md:137`
`.claude/skills/differential-testing.md:29`

The remediated harness verifies only `git rev-parse HEAD^{commit}`. The progress
record explicitly identifies the current measurement as base commit
`d0318ce0` carrying the uncommitted F-X075 implementation, so that HEAD does
not identify the production code that was compiled and timed. The historical
commits do not contain this new benchmark either, which means those measured
trees also need at least an uncommitted harness injection. The check has no
cleanliness predicate and no hash or allowlist for those dirty changes. An
arbitrary engine modification under any expected HEAD therefore passes the
identity assertion and produces evidence labeled as the pinned build. After
the feature is committed, the `current` label instead rejects the actual
integrated implementation because its hard-coded expected HEAD remains the
base. Bind each measurement to the complete compiled tree, allowing only an
exactly hashed benchmark injection on historical checkouts, and make the
current identity remain valid for the reviewed feature state.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 1 D1 closure** produced zero findings. The equality helper now checks
  every metadata field, exact logical structure, and the complete debug
  representation at `crates/rdocx-layout/src/engine.rs:10743`. The ten
  page-spanning edits use `layout_with_provenance`, compare complete warm and
  fresh layout results, and compare the exact source maps at
  `crates/rdocx-layout/src/engine.rs:11063`.
- **Pass 1 D2 evidence detail** produced zero findings beyond D1. Every timing
  line now reports the requested checkout label and resolved SHA at
  `crates/rdocx/tests/regression_test.rs:303`. The progress record retains all
  four round medians for every size, path, and revision and classifies their
  variance at `.claude/scratch/F-X075-progress.md:142`. D1 is limited to what
  source content that SHA actually authenticates.
- **Checkpoint safety** produced zero findings. The recorded path remains
  available only after the unchanged source and rendered-block predicates at
  `crates/rdocx-layout/src/engine.rs:1390`. A split continuation finalizes its
  page through `finish_page` at
  `crates/rdocx-layout/src/paginator.rs:2194`, while checkpoint creation stays
  confined to drained complete-block boundaries in `finish_page_before` at
  `crates/rdocx-layout/src/paginator.rs:1189`.
- **Restart assembly and publication** produced zero findings. Prefix pages,
  rebuilt pages, and an exact cached tail are assembled under the existing
  checkpoint and suffix conditions at `crates/rdocx-layout/src/engine.rs:1467`.
  Complete candidates retain checked aggregate admission and whole-record
  replacement at `crates/rdocx-layout/src/engine.rs:1852`. No failure remains
  after publication.
- **Warm and fresh behavior** produced zero findings. The 175-paragraph source
  pins 16 pages, four lines per paragraph, one recorded pass, and the exact
  complete-boundary checkpoint vector at
  `crates/rdocx-layout/src/engine.rs:11017`. Ten sourced middle edits pin cache
  counts and a two-page work bound. Late edit, insertion, deletion, and undo
  use the same split-producing source and compare complete layout results at
  `crates/rdocx-layout/src/engine.rs:11103`.
- **Notes, fields, and unsafe exclusions** produced zero findings. The combined
  note-bearing split and PAGE-footer test verifies post-split checkpoint
  placement, every displayed page number, warm-to-fresh equality, and bounded
  work at `crates/rdocx-layout/src/engine.rs:11148`. The preexisting matrices
  continue to cover dirty note continuation, numbering, drawings, raw XML,
  fields, unsafe tables, wraps, backgrounds, and multiple sections at
  `crates/rdocx-layout/src/engine.rs:11394` and
  `crates/rdocx-layout/src/engine.rs:12293`.
- **Benchmark construction and thresholds** produced zero findings beyond D1.
  The ignored test builds both approved source sizes, primes the reusable
  engine, times only ten warm edits, and selects the native or deterministic
  bundled-fallback path at `crates/rdocx/tests/regression_test.rs:248`. Timing
  remains outside normal test thresholds, and the recorded worst-round ratios
  remain inside both approved limits at
  `.claude/scratch/F-X075-progress.md:160`.
- **Test sensitivity** produced zero findings beyond D1. Reverting the
  production change restores the split veto and fails the mandatory retained
  restart state at `crates/rdocx-layout/src/engine.rs:11037`. Metadata is
  nonempty, provenance is enabled, and the exact page, line, cache, work, and
  checkpoint assertions prevent a placeholder fixture from passing.
- **Public surface, dependencies, and structure** produced zero findings. The
  production diff removes only one private flag and its private full-layout
  fallback. No manifest, public API, feature, dependency, crate, module, file,
  trait, or generic changes are present. The approved private boundary remains
  stated at `.claude/plans/F-X075-design.md:47`.
- **Panics and arithmetic** produced zero findings. Production adds no unwrap,
  expect, direct indexing, slicing, unchecked arithmetic, or input-driven
  allocation. New process execution, assertions, and indexing are confined to
  the ignored source-built benchmark and unit fixtures at
  `crates/rdocx/tests/regression_test.rs:222` and
  `crates/rdocx-layout/src/engine.rs:11007`.
- **HLD scope and output stability** produced zero findings. The HLD changes
  remain exactly the three files approved at
  `.claude/plans/F-X075-design.md:94`, and their checkpoint and exactness
  contracts match the implementation. The progress record reports the two
  Word WASM checks and the unchanged 49-entry hash harness at
  `.claude/scratch/F-X075-progress.md:84`.
