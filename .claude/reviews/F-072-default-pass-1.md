# F-072, default, pass 1

**Reviewed**: working diff, 7 implementation and test files, 1034 insertions and 21 deletions including the untracked 642-line picture module
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, unrelated alternate content satisfies the required blip-fill choice

`crates/rpptx-oxml/src/picture.rs:279`

Any `mc:AlternateContent` immediately after `p:nvPicPr` sets
`has_alternate_blip` without checking that the preserved subtree contains a
PresentationML `p:blipFill`. A picture containing an unrelated compatibility
choice followed by `p:spPr` is therefore accepted with `blip_fill = None`, even
though the design requires either a direct fill or an opaque alternate
blip-fill choice. Validate the presence and namespace of the alternate
`p:blipFill` while keeping the entire compatibility subtree opaque.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML preservation, test-gate, or
structural findings were found.
