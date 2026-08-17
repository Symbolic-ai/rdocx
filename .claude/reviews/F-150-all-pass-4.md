# F-150, all, pass 4

**Reviewed**: full working diff against `e25ef35`, 2 files, 1,404 additions and 2 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, property rejection does not validate the complete prior value
`crates/rdocx/src/revision.rs:539`

`prior_property` accepts the first namespace-correct property child and does
not reject any later element children. A selected `w:rPrChange` containing a
valid `w:rPr` followed by another property element therefore succeeds. Both
acceptance and rejection discard the malformed extra child, and rejection
commits the first prior value. This violates the approved atomic-failure
contract for malformed selected changes. The staged document reparse cannot
detect the problem because the invalid change wrapper has already been
removed.

## Smells

No smells found.

## Nitpicks

No nitpicks found.

## Not found

Pass-3 D1 is fixed by collecting and merging the complete consecutive
paragraph chain while retaining the final paragraph properties. Pass-3 D2 is
fixed by rendering selected descendants for validation before a selected
outer wrapper or contextual owner is removed. Pass-3 D3 is fixed for the
required prior-property namespace and local-name match. Pass-3 D4 is fixed by
accepting lowercase `t` and `z` separators.

No additional findings were found in the eight-method public API shape,
author, id, offset, fractional-second, or inclusive date selection, deleted
text conversion, content-wrapper namespace promotion, contextual row and run
ownership, unmodelled lookalike preservation, mutation commit ordering, cache
invalidation, oracle pinning, panic safety, or structural-rule compliance.
