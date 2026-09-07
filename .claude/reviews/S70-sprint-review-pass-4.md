# S70 sprint review, pass 4

**Reviewed**: `sprint/s70` against
`59c47fb0877e2b1139f1d2c690bac57e1f8fb138`, 40 files, 6,137 changed
lines, crates: `rdocx`, `rdocx-layout`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Extension decision

This pass explicitly extends the default three-pass bound because `/run-sprint`
requires a sprint review at the scheduled Wave 3 dependency boundary. Pass 3
was clean before F-242 and F-X085 integrated. The extension reviews that new
combined delta and is not a continuation of unresolved findings.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M23 end gate is not due at this dependency-prefix boundary. Its contract
requires generating all five private references from a blank public facade,
while the delivery backlog still assigns that implementation path to F-243
through F-263. This review makes no early claim that the milestone is complete.

The completed-prefix gate holds. The root README distinguishes authoring,
reading, mutation, preservation, and permanent non-goals at `README.md:13` and
maps its bounded product summary to canonical capability IDs at `README.md:37`.
The validator binds those claims, metadata-derived versions, every local link,
and exact official comparison evidence at `scripts/readme_doctests.py:383`,
`scripts/readme_doctests.py:460`, `scripts/readme_doctests.py:507`, and
`scripts/readme_doctests.py:546`. The integrated README runner compiled all 23
Rust examples and verified all 27 inventories and 22 packaged copies.

F-X085 keeps fingerprint filtering before exact identity serialization at
`crates/rdocx-layout/src/engine.rs:1188`, shares one layout-local memo across
all three scans at `crates/rdocx-layout/src/engine.rs:2046`, and moves computed
bytes into publication at `crates/rdocx-layout/src/engine.rs:2474`. Its
715-block at-most-once regression and transient-memory bound pass at
`crates/rdocx-layout/src/engine.rs:11530` and
`crates/rdocx-layout/src/engine.rs:11591`. The combined full gate passed with
all 49 hash entries unchanged.

## Not found

- Interaction: F-X084 note-part filtering and F-X085 identity memoization meet
  at the retained body entry but preserve separate note and exact-body gates at
  `crates/rdocx-layout/src/engine.rs:1219` and
  `crates/rdocx-layout/src/engine.rs:1279`.
- Duplication: the sprint has one public authoring conformance owner, one root
  README validator, and one restart identity memo.
- Layering: no Cargo manifest or lockfile changed, and no shared crate gained a
  format dependency.
- Harness: every completed story records an unchanged 49-entry result, and no
  rendering baseline changed.
- Gate: the completed prefix has executable conformance, README, cache, private
  corpus, and full-workspace evidence. The later M23 result remains unclaimed.
- Docs: the F-242 and F-X085 plan impact lists match their HLD edits, including
  the approved F-X002 three-example correction.
- Dependencies: no dependency or feature changed.
- Surface: F-242 changes documentation and validation only. F-X085 adds one
  private concrete helper and no public API.
