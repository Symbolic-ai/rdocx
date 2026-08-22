# F-X045, all, pass 2

**Reviewed**: remediated working tree diff against `7f317e7`, 4 implementation and HLD files, 1,307 changed lines, comprising 1,218 insertions and 89 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, watermark allowance checks only the raw root local name

`crates/rdocx-layout/src/engine.rs:1663`

The corrected predicate counts every raw root whose local name is `pict`, but
does not require that root to be in the WordprocessingML namespace. A producer
wrapper such as `x:pict` containing a nested supported `w:pict` yields one raw
`pict` root and one projected watermark, so the counts match and the opaque
wrapper passes reuse. Resolve the root prefix through the part's captured
namespace declarations, recognize the conventional `w` binding, and require
the WordprocessingML namespace before allowing the projected raw subtree. Add
a nested foreign-namespace `pict` bypass assertion.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no further findings. The cache remains exact, transactional,
  insertion-ordered, source-rebinding, diagnostic-replaying, and font-trace
  replaying.
- Contract: no further findings. Section revisions and opaque section children
  now bypass reuse, and complete context changes still disable every hit.
- Panics: zero production findings. No new unchecked input indexing or
  fallible-operation assertion was introduced by the remediation.
- OOXML: no further findings. Raw XML remains preserved and serialization order
  is unchanged.
- Tests: no further findings. The two-section resolved-inheritance fixture now
  proves twelve warm variant hits, and structural key capacity now exercises
  oversized bypass.
- Structure: zero findings. The remediation adds only local predicates and
  accounting within the existing engine module.
