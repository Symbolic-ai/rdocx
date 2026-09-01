# F-X072, correctness, pass 1

**Reviewed**: claim-base `43cd8a51af75f9ddfd3148236ac59dc095345c94`
through the complete working-tree delta, the approved design plan, cited HLD
sections, progress record, and live Issue 65 contract. The implementation delta
is 1 file with 156 changed lines, 144 additions and 12 deletions.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the required unsafe-prefix regression omits drawings and raw children

`crates/rdocx-layout/src/engine.rs:9492`
`crates/rdocx-layout/src/engine.rs:9502`

The approved `unsafe_prefix_still_disables_later_paragraph_hits` regression is
required to prove that fields, numbering, drawings, raw children, and other
unsupported content remain conservative after note references become safe.
The implemented loop exercises only fields and numbering. It would still pass
if a later change made drawing or raw-child paragraphs cache-safe, or stopped
either class from disabling reads of the cached suffix. Add direct drawing and
raw-child prefix cases that prime a safe suffix, insert the unsafe prefix, and
assert zero suffix hits plus warm and fresh equality. The production predicate
currently rejects raw paragraph XML at
`crates/rdocx-layout/src/engine.rs:2303` and drawings at
`crates/rdocx-layout/src/engine.rs:2335`, so this is a test-gate defect rather
than a confirmed output defect.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Correctness** produced no production finding. Direct body footnote and
  endnote references are admitted only after every pre-existing paragraph and
  run safety check at `crates/rdocx-layout/src/engine.rs:2299`. A cache hit
  requires the fingerprint, complete typed paragraph, exact width, and revision
  view at `crates/rdocx-layout/src/engine.rs:1903`. The reusable context compares
  exact footnote and endnote parts before enabling reads, so note-part mutation
  rebuilds the transaction.
- **Reference identity** produced zero findings. Footnote and endnote IDs are
  retained in `RunContent` and therefore in complete paragraph equality. The
  shared fingerprint tag can collide across IDs, but it is only a prefilter and
  cannot serve a mismatched paragraph.
- **Scope boundaries** produced zero findings. The new helper keeps note-bearing
  header and footer paragraphs conservative at
  `crates/rdocx-layout/src/engine.rs:2402`. It also rejects note references in
  every recursive table paragraph at `crates/rdocx-layout/src/engine.rs:2508`.
  Body restart identity continues to record the ordered note-reference stream
  separately.
- **Contract** produced no finding beyond D1. The implementation changes one
  existing private predicate and adds focused tests in the existing engine test
  module. It adds no facade behavior, dependency, feature, module, file, trait,
  generic, or external oracle.
- **Panics and bounds** produced zero findings. The production change adds no
  indexing, slicing, arithmetic, recursion, allocation bound, unwrap, or expect.
  Existing complete-key equality and the 4,096-entry, 50 MiB paragraph limits
  remain authoritative.
- **OOXML and preservation** produced zero findings. No parser, serializer,
  schema order, namespace, raw subtree, or package path changes. Raw XML remains
  excluded from cache-safe paragraphs.
- **Tests** beyond D1 produced zero findings. The two 700-paragraph reference
  regressions prove 699 warm hits and one rebuild for both streams. Changed-ID
  and changed-note-part cases compare warm output with a fresh deterministic
  engine, and the related-story case proves exact warm and fresh equality.
- **Structure** produced zero findings. `paragraph_has_note_reference` has two
  distinct production consumers and keeps the body, table, and header or footer
  policy visible in the same file. No unnecessary abstraction was added.
- **Focused evidence**: all five note-reference filtered tests passed. Full
  `cargo test -p rdocx-layout --quiet` passed 219 unit tests and one doctest.
  Crate all-target, all-feature Clippy with warnings denied and workspace format
  check passed. Both WASM targets checked successfully. The deterministic hash
  harness matched all 49 entries. The exact release-mode ignored 1,000-page
  regression passed with one test thread. `git diff --check` passed.
