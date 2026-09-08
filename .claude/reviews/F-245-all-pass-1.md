# F-245, all, pass 1

**Reviewed**: working-tree diff, 17 files, 1,576 additions and 40 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, removing a font can move or discard retained root children

`crates/rdocx-oxml/src/font_table.rs:244`

`FontTable::remove_font` removes the vector entry without rebasing
`raw_children` boundaries. A foreign child between two font records remains at
the old boundary and moves after the surviving second record when the first is
removed. A foreign child after the final removed record retains a boundary that
the shorter serialization loop never emits, so it disappears. This violates
the required unmodeled-child preservation during the public remove operation.

### D2, modeled font properties discard valid producer representation

`crates/rdocx-oxml/src/font_table.rs:303`

Explicit start and end forms of `w:altName`, `w:family`, `w:pitch`, and the four
embedded-font elements are retained as raw XML rather than projected into the
typed fields. The empty-element path at line 320 models those values but drops
unknown attributes on the three descriptive properties. A subsequent typed
edit can therefore miss a valid existing value or discard producer attributes,
contrary to the prefix-tolerant typed-read and producer-attribute preservation
contract.

### D3, shared embedded-font relationships are not reference safe

`crates/rdocx/src/document.rs:8697`

Replacement always overwrites the existing relationship target, even when a
second font record or face references the same relationship or another font
relationship targets the same part. Removal at line 8840 deletes the selected
relationship without checking whether another font-table element uses its id.
The first case changes an unrelated face's bytes. The second leaves the other
element dangling and can delete its only part. The removal check counts only
remaining relationships, not remaining font-table references.

### D4, the differential gate does not consult an external oracle

`crates/rdocx/tests/integration_test.rs:413`

The required differential test only compares an authored document against a
second document that selects the repository's bundled Carlito face. The Word
and LibreOffice values are string equality checks against unrelated existing
constants. No Word-produced package, parsed tree, LibreOffice output, or
recorded external observation contributes to the assertion, so the test cannot
detect a shared mistake in theme or embedded-font behavior and does not prove
the design plan's external-oracle contract.

## Smells

None.

## Nitpicks

None.

## Not found

No additional panic-safety or arithmetic defects were found in the new public
input paths. No new trait, generic parameter, feature flag, crate, forwarding
wrapper, or unjustified module was introduced. Relationship and content-type
creation is staged before publication, and the legacy tint and shade helper is
unchanged.
