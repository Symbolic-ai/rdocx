# F-080, full, pass 1

**Reviewed**: working-tree diff, 8 files, 473 insertions and 11 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the seven content-type dispatches at
  `crates/rpptx/tests/integration.rs:466` parse, serialise, reparse, and compare
  the same concrete roots with deck and part context.
- Contract: the expected-package construction at
  `crates/rpptx/tests/integration.rs:77` rewrites every modelled root while the
  facade path owns presentation, slide, and notes-slide writes as approved.
- Panics: production parsing at `crates/oxml-drawing/src/theme.rs:1122` remains
  fallible. Panics and unwraps added under `crates/rpptx/tests/integration.rs`
  are assertion failures in the corpus test harness.
- OOXML: the optional `a:fmtScheme/@name` read and write at
  `crates/oxml-drawing/src/theme.rs:1122` and
  `crates/oxml-drawing/src/theme.rs:1136` preserve attribute absence without
  changing child order or opaque children.
- Tests: the focused regression at `crates/oxml-drawing/src/theme.rs:108` pins
  absent-name preservation, and the corpus gates at
  `crates/rpptx/tests/integration.rs:38` cover structural equality, expected
  package bytes, and the facade read surface.
- Structure: the implementation adds no production module, trait, generic,
  wrapper, or feature flag. All corpus logic stays in the one approved
  integration entrypoint.
