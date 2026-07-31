# F-054, all aspects, pass 1

**Reviewed**: uncommitted worker diff, 4 files, 336 additions and 15 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, colour parsing does not define the approved concrete error

`crates/oxml-drawing/src/color.rs:6`

The approved design requires malformed RGB input to return a concrete error
defined in `color.rs`. The implementation instead imports and returns the
shared `oxml_core::OxmlError` for the entire colour API. Callers therefore
cannot identify an F-054 colour validation error through the promised local
error contract. Define a concrete colour error in this module and keep the
dependency set limited to `oxml-core` and `quick-xml`.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the four choice arms select the intended values and fixed tags.
- Panics: production code has no panic path on input bytes.
- OOXML: local-name matching accepts arbitrary prefixes, writing fixes the
  prefix to `a:`, and raw child subtrees retain their document order and bytes.
- Tests: all four forms, an optional system fallback, malformed RGB, and raw
  child preservation have direct coverage.
- Structure: the new module is authorised, remains concrete, and adds no
  traits, generic data types, wrappers, or feature flags.
