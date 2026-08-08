# F-109, all, pass 1

**Reviewed**: uncommitted working diff against `HEAD`, 11 files, 707 additions and 14 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, inserting an absent group transform emits it after preserved group properties
`crates/rpptx-oxml/src/shape_tree.rs:1051`

`group_transform_mut` inserts the transform without moving raw children from
boundary 0. `GroupProperties::write_xml` emits boundary 0 before `a:xfrm`,
while the parser at line 1584 rejects any `a:xfrm` that follows another
`p:grpSpPr` child. A valid group with no transform and a preserved fill,
effect, or other group property therefore becomes schema-invalid after
`set_position`, `set_size`, or `set_rotation` creates its transform.

### D2, the nested-group test cannot prove sibling order or shape-id preservation
`crates/rpptx/tests/integration.rs:188`

The approved test plan requires the recursive mutation test to show that
sibling order and ids do not change. This fixture has exactly one group child,
and the assertions check neither its id nor any sibling id. The test still
passes if recursive mutation renumbers that child, and a one-child collection
cannot detect reordering.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, OOXML, or test findings. No panics on
untrusted input and no structure-rule violations were found. Prefix handling,
name escaping, fixed-prefix output, `AlternateContent` immutability, finite
adjustment rejection, and the direct dependency direction were checked.
