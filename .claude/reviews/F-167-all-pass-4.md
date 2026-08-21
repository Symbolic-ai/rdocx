# F-167, all, pass 4

**Reviewed**: complete working diff, 4 implementation files, 2,895 additions and 6 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, empty prior-property cleanup drops foreign owner attributes
`crates/rdocx/src/revision.rs:583`

Rejection removes a prior `w:pPr` whenever it is empty or contains only
whitespace and no retained children. That test does not account for attributes
on the prior property owner. A captured change whose prior value is
`<w:pPr ext:flag="keep"/>` is therefore treated as absent and discarded on
rejection, even though the raw property inside `w:pPrChange` preserves that
producer attribute. Property rejection must restore the complete
namespace-correct prior value, including unmodelled owner attributes.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-3 numbering remediation was confirmed for add and remove operations
with absent property owners. Empty synthesized current and prior owners are
removed, while owners containing modeled properties or foreign child elements
remain. Foreign same-local elements do not enter Word owner cleanup.

No additional defects were found in modeled field ownership, paragraph and row
marker placement, direct row insertion boundaries, recursive control-owned
rows, nested tables and content controls, schema child order, raw subtree
preservation, accepted and rejected content postconditions, metadata escaping,
revision-id allocation, mutation atomicity, public API exposure, deterministic
LCS tie-breaking, panic safety, or structural discipline.
