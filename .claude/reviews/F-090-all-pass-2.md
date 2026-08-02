# F-090, all aspects, pass 2

**Reviewed**: remediated uncommitted implementation diff, 6 files, 20587 additions and 5 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the source digest and 187 direct definitions are pinned, exact
  source slices prove the identical duplicate, conflicting duplicates fail,
  and 186 sorted lookup keys are emitted deterministically.
- Contract: the implementation carries the full Ecma BSD notice, exposes only
  a private generated lookup, and keeps HLD impact at none.
- Panics: production Rust gains no input-facing panic path. Generator failures
  are explicit command failures with contextual messages.
- OOXML: each emitted value is a complete fixed-prefix `a:custGeom`, source
  child order is retained, and corpus matching requires the DrawingML expanded
  name rather than only a local name.
- Tests: the four named gates cover byte-identical regeneration, corrected
  direct and unique counts, exact duplicate bytes, all 50 corpus decks, and
  known and unknown lookup.
- Structure: the new source, notice, generator, and generated module are the
  exact files explicitly approved by F-090. No trait, generic parameter, crate,
  dependency, or feature flag is added.
