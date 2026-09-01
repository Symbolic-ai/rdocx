# F-X073, correctness, pass 1

**Reviewed**: claim base `ef34d1f31eb9334d993bf333762079682633319f`
through the complete working-tree delta, including untracked state, the
approved revised design plan, progress record, cited HLD sections, risk-routing
riders, and live Issue 66 contract. The delta is 7 files with 711 changed
lines, 547 additions and 164 deletions.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, unsupported typed fields can publish pagination checkpoints
`crates/rdocx-layout/src/engine.rs:2424`
`crates/rdocx-layout/src/engine.rs:3221`
`crates/rdocx-layout/src/engine.rs:5034`
`crates/rdocx-layout/src/engine.rs:11064`

The new restart source predicate admits every `RunContent::Field`. The later
block predicate does not identify a field from its source. It only rejects a
rendered segment whose `field_kind` is present. PAGE, NUMPAGES, and a resolved
PAGEREF receive such a marker, but REF and every unsupported field instruction
receive `None`. A DATE field in an otherwise safe ordinary-prose body therefore
passes both predicates and the recorded paginator can publish checkpoints.
This violates the approved fail-closed field contract and the explicit
field-bearing zero-checkpoint contract at
`docs/hld/08-rendering-spec.md:795` and
`docs/hld/12-testing-strategy.md:541`. The required unsafe-content regression
does not detect the gap because its only field case is PAGE. Classify
field-bearing source directly so every typed field retains any supported
substitution pair but contributes no pagination checkpoint, and add non-PAGE
and unresolved-field cases to the regression.

### D2, bookmark cardinality can disguise foreign raw paragraph content
`crates/rdocx-layout/src/engine.rs:2372`
`crates/rdocx-layout/src/engine.rs:11085`

The source guard treats all paragraph `extra_xml` as represented whenever its
entry count equals the typed bookmark-marker count. It does not prove that each
raw entry is the bookmark raw subtree projected by the corresponding marker.
For example, construct a bookmark through the public paragraph API, replace its
single retained raw entry with `<w:unknown/>`, and leave the typed marker in
place. The counts still match, the otherwise ordinary paragraph becomes
restart-record safe, and the unknown raw child can publish checkpoints even
though raw content is required to fail closed at
`docs/hld/08-rendering-spec.md:777` and
`docs/hld/14-development-backlog.md:3535`. Exact restart identity prevents a
different source from aliasing this source, but it does not represent the
unknown element's pagination effects. Validate one-to-one raw bookmark
ownership and ordering rather than cardinality alone. Extend the raw-child
regression with a same-count foreign raw decoy.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Aggregate accounting** produced zero findings. Candidate admission uses
  checked addition across published and pending paragraph, table, header or
  footer state plus the complete restart candidate at
  `crates/rdocx-layout/src/engine.rs:2170`. It enforces both the 5,216-entry and
  64 MiB limits, rejects overflow, retains every component partition, and
  replaces rather than double-counts the old restart record.
- **Restart size and publication** produced zero findings. Complete candidates
  have retained capacities charged at
  `crates/rdocx-layout/src/engine.rs:3275`, and incomplete body identities fail
  closed through `usize::MAX` at `crates/rdocx-layout/src/engine.rs:1870`.
  Publication remains a successful-layout transaction at
  `crates/rdocx-layout/src/engine.rs:1113`.
- **Ordinary-prose correctness** produced zero findings beyond D1 and D2.
  Multi-line prose, headings, `keepNext`, and `keepLines` no longer fail the
  complete-record predicate at
  `crates/rdocx-layout/src/engine.rs:3239`. Checkpoints are still emitted only
  after a finalized empty-page boundary with drained note and wrap state at
  `crates/rdocx-layout/src/paginator.rs:1197`.
- **Split fallback** produced zero findings. Every paragraph split is recorded
  at `crates/rdocx-layout/src/paginator.rs:2163`. A detected split disables the
  restart record, reruns the complete ordinary paginator from the start, and
  publishes no checkpoint state at
  `crates/rdocx-layout/src/engine.rs:1512`.
- **Exact identity and substitution behavior** produced zero findings beyond
  D1. Restart body entries retain byte-exact serialized paragraph or table
  identity and an independent fingerprint at
  `crates/rdocx-layout/src/engine.rs:722`. Exact context, ordered note
  references, font trace, source mode, and complete body suffix remain required
  before reuse at `crates/rdocx-layout/src/engine.rs:1410`. Supported PAGE,
  NUMPAGES, and PAGEREF substitution inputs remain exact at
  `crates/rdocx-layout/src/engine.rs:1689`.
- **Panics and arithmetic** produced zero findings. The production delta adds
  no untrusted indexing or slicing. Aggregate admission and paragraph
  publication use checked arithmetic, while candidate retained-byte accounting
  saturates and therefore rejects overflow. Existing checkpoint expectations
  remain guarded by the reusable-record predicates.
- **OOXML and preservation** produced zero parser or serializer findings. The
  source delta does not change OOXML parsing, writing, namespace handling,
  schema order, raw replay, relationships, or package parts. D2 concerns the
  cache safety classification of already retained raw XML, not its byte
  preservation.
- **Public contract and structure** produced zero findings. No public API,
  dependency, feature, module, file, trait, generic, binding, or package graph
  changed. The paginator adds one boolean to an existing private recorded-pass
  result, and the accounting helper remains local to its single engine owner.
- **HLD and hash expectations** produced zero findings. The revised plan lists
  exactly HLD 08, 12, 14, and newly required HLD 15, and all four describe the
  aggregate budget and restart boundary. The deterministic hash harness matched
  all 49 entries with no baseline delta.
- **Tests and routed evidence** produced no finding beyond D1 and D2. All six
  plan-named regressions passed. Full `rdocx-layout` passed 225 unit tests and
  one doctest. The `rdocx` regression binary passed 180 tests with one ignored,
  and the crate unit binary passed 326 tests with three ignored. The unrelated
  pinned LibreOffice ODT integration test could not run its `soffice`
  conversion in this sandbox, while the other 129 integration tests passed.
  All-target, all-feature `rdocx-layout` Clippy, both WASM target checks,
  workspace formatting, prose, and diff hygiene passed. The exact ignored
  release-mode 1,000-page performance regression passed with deterministic
  fonts and one test thread.
