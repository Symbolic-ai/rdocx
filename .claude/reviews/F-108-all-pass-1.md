# F-108, all aspects, pass 1

**Reviewed**: working tree against `ca5d1e1`, 7 files, 931 insertions and 40
deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, shape validation assumes the tree root id is always 1

`crates/rpptx/src/lib.rs:416`

The duplicate-id set is seeded with `1` instead of the parsed
`p:spTree/p:nvGrpSpPr/p:cNvPr/@id`. A producer can assign another valid root
id, and a child carrying that same id then passes validation. Expose the
existing parsed root id through `CT_ShapeTree`, seed the set from that value,
and add a root-to-child collision case.

### D2, XML parts without a relationships collection are never scanned

`crates/rpptx/src/lib.rs:464`

The relationship pass iterates only `package.part_rels.keys()`. If a corrupted
deck removes an entire slide relationships part while the slide XML still
contains `r:embed`, `r:id`, or another relationship attribute, the reference is
never inspected and no `DanglingRelationship` is returned. Iterate the union of
package parts and relationship sources, using an empty relationship collection
for a part with no `.rels`, and make the gate cover this deletion case.

### D3, recursive shape validation can overflow the stack

`crates/rpptx/src/lib.rs:830`

`validate_shape_children` recursively calls itself for every nested group and
selected fallback. The public contract says validation is total and
non-panicking, but an adversarially deep owned tree can exhaust the process
stack rather than return issues. Traverse through an explicit heap-backed work
stack while preserving depth-first order.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, OOXML preservation, schema order, test, or structure
findings were found. The issue categories are emitted in deterministic variant
order, package key traversal is sorted, semantic slide-id and empty-text checks
are correctly deferred from the parsers to the facade, the corpus gate covers
all 50 pinned decks, and no new module, crate, trait, generic, wrapper, or
feature flag was introduced.
