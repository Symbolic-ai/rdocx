# F-167, all, pass 3

**Reviewed**: complete working diff, 4 implementation files, 2,738 additions and 6 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, adding or removing numbering fails on an absent property owner
`crates/rdocx/src/comparison.rs:614`
`crates/rdocx/src/comparison.rs:1343`

When the original paragraph has no `w:pPr`, a numbering change records an
empty `w:pPr` as the prior value. Rejecting that change therefore produces a
typed empty property owner. `paragraph_signature` maps the original absence to
`None` but maps the restored empty owner to `Some((None, None))`, so the reject
postcondition fails even though both represent the same unnumbered list
structure. Removing numbering has the symmetric acceptance failure when the
edited paragraph has no property owner. This blocks list addition and removal,
which are within the approved exact list-structure scope.

## Smells

None.

## Nitpicks

None.

## Not found

All pass-2 remediations were confirmed. Unchanged modeled fields use paragraph
serialization ownership, field edits reject atomically, paragraph markers
target direct paragraph-mark properties, row markers target direct outer row
properties, whole-table marking includes control-owned rows, and control block
replacement retains raw whitespace on its original side. Direct row prepend
and append placement remains within the table shell and respects control and
raw boundaries.

No additional defects were found in revision namespace cleanup, recursive row
ownership, schema child order, nested table and content-control traversal,
unmodelled XML preservation, accepted and rejected content postconditions,
metadata escaping, revision-id allocation, mutation atomicity, public API
exposure, deterministic LCS tie-breaking, panic safety, or structural
discipline.
