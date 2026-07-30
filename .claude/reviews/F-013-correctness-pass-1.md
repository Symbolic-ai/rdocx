# F-013, correctness, pass 1

**Reviewed**: the 15-file working diff for F-013, comprising 979 additions and
10 deletions across the new crate, workspace metadata, approved HLD files, and
the design checklist.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the copied conversions preserve their truncating casts, the
  shared XML helpers are prefix tolerant, and raw subtree capture retains every
  event byte for byte under its regression test.
- Contract: `oxml-core` stays at 0.0.0 with publishing disabled, existing Word
  sources remain in place for F-015 and F-016, and no release workflow changed.
- Panics: no new production `unwrap`, `expect`, unchecked indexing, or unchecked
  arithmetic was introduced.
- OOXML: the copied serializer retains its established child order and fixed
  prefixes, while the new parsing helper matches qualified and unqualified
  names.
- Tests: all 18 moved tests run from the new crate, and four focused additions
  cover the public XML text surface, namespace helpers, and exact raw XML
  preservation.
- Structure: the story explicitly authorizes the new crate and modules. No new
  trait, generic parameter, wrapper, or feature flag was introduced.
