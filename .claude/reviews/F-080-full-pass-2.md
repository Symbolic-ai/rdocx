# F-080, full, pass 2

**Reviewed**: working-tree diff, 10 files, 495 insertions and 15 deletions
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
  each concrete root with deck and part context. The expected and actual
  package construction at `crates/rpptx/tests/integration.rs:77` exercises the
  approved facade-owned and independently serialised roots.
- Contract: corpus discovery at `crates/rpptx/tests/integration.rs:394` requires
  the pinned 50-entry manifest when the corpus gate is required, while the
  package comparison at `crates/rpptx/tests/integration.rs:567` checks content
  types, relationships, part names, counts, and bytes.
- Panics: production theme parsing at
  `crates/oxml-drawing/src/theme.rs:1122` remains fallible. Added panics and
  unwraps under `crates/rpptx/tests/integration.rs` and the focused unit tests
  are assertion failures in test code.
- OOXML: optional `a:fmtScheme/@name` handling at
  `crates/oxml-drawing/src/theme.rs:1122` and
  `crates/oxml-drawing/src/theme.rs:1136` preserves absence without changing
  child order. Canonical `a:blip` output at
  `crates/oxml-drawing/src/fill.rs:630` declares `r` whenever it writes an
  `r:embed` or `r:link` attribute, making the rewritten theme namespace-valid.
- Tests: the absent-name regression at
  `crates/oxml-drawing/src/theme.rs:108`, relationship-prefix regression at
  `crates/oxml-drawing/src/fill.rs:1370`, and corpus gates at
  `crates/rpptx/tests/integration.rs:38` would fail if their respective fixes
  were reverted.
- Structure: the implementation adds no production module, trait, generic,
  wrapper, feature flag, or crate. Corpus logic remains in the one approved
  integration entrypoint.
