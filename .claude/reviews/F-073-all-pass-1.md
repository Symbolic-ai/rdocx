# F-073, all aspects, pass 1

**Reviewed**: working diff, 5 files, 1,015 additions and 10 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, non-element XML inside typed frame containers is discarded

`crates/rpptx-oxml/src/graphic_frame.rs:180`

The frame, `a:graphic`, `a:graphicData`, and non-visual shell readers retain
element children but ignore text, comments, CDATA, and processing instructions.
A producer comment between two modelled frame children, or inside
`p:nvGraphicFramePr`, disappears on serialisation instead of remaining in its
original slot. This violates the approved preservation contract for unsupported
content.

### D2, arbitrary attribute values can manufacture inherited namespaces

`crates/rpptx-oxml/src/graphic_frame.rs:637`

The namespace filter treats every whitespace-separated attribute value as a
possible prefix. If an ordinary attribute such as `name="mc"` occurs in a frame
whose source ancestors do not bind `mc`, the shape-tree writer adds its standard
`mc` binding and the second parse retains that newly introduced binding at the
frame root. The model then changes across serialise and reparse. Only QName
values and the markup-compatibility prefix-list attributes should be inspected
as namespace references.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML ordering, test-gate, or
structural findings were found.
