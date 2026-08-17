# F-152, all, pass 1

**Reviewed**: working-tree diff, 12 files, 1,403 additions and 53 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, self-closing ordinary content remains opaque
`crates/rdocx-oxml/src/content_control.rs:582`

Every self-closing child of `w:sdtContent` is stored as raw XML. A valid empty
paragraph such as `w:p` therefore survives serialization but is absent from
paragraph traversal. This violates the contract that ordinary paragraphs
inside controls remain traversable.

### D2, paragraph boundary controls can move inside a preceding hyperlink
`crates/rdocx-oxml/src/text.rs:621`

The writer emits the boundary before closing the hyperlink that covered the
previous run. A run-level control that followed the hyperlink as a paragraph
sibling is consequently emitted as a child of `w:hyperlink`, changing its
placement and producing a different tree.

### D3, paragraph controls are reordered across comment anchors
`crates/rdocx-oxml/src/text.rs:757`

Controls and comment anchors record only how many raw children preceded them.
When both occur at the same run and raw boundary, the writer always emits the
control first. An input anchor followed by a control is therefore reordered,
which breaks exact parent-position preservation.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in contract, panics, OOXML namespace handling, test
isolation, or structure. The approved implementation adds no trait, generic,
dependency, feature flag, crate, or extra module.
