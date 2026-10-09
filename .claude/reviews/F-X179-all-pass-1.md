# F-X179, all, pass 1

**Reviewed**: PR 271 implementation and nine comment acceptance regressions on sprint/s90. Ten modified files, 525 additions and 101 deletions including pre-existing sprint planning changes excluded from feature review.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Nested legacy replies survive root removal

`crates/rdocx/src/comments.rs:1563`

When a removed reply has multiple paragraphs, the traversal records only its
extension paragraph id. A grandchild linked to that reply's first paragraph
remains present after root deletion even though parent lookup recognizes the
link. The added regression
`removing_a_root_removes_nested_replies_linked_to_legacy_first_paragraphs`
fails on the contributed implementation. Add every paragraph id of a removed
descendant to the traversal frontier and extension cleanup set.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in contract, panics, OOXML preservation, test gate or
structure. The contributed gate fails before the implementation and passes
with it. Blank lines, missing final ids, fragment import and exact unsupported
subtree preservation pass. No new public type or source file is introduced.
