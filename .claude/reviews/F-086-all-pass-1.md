# F-086, all aspects, pass 1

**Reviewed**: working-tree diff, 4 files, 849 changed lines
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, the visibility test does not prove the controls are independent

`crates/rpptx-layout/src/context.rs:805`

The fixture sets both the layout and slide `showMasterSp` values to zero in the
same case. The test therefore passes if either control suppresses both inherited
passes, or if the two controls are accidentally swapped. The approved test plan
requires the layout value to suppress only the master pass and the slide value
to suppress only the layout pass.

## Smells

### S1, background fallback order is only tested with a slide background

`crates/rpptx-layout/src/context.rs:773`

The only background assertion provides a slide background. It does not pin the
layout, master, and theme fallback cases or prove that the selected view retains
the context's per-master colour map. A later fallback-order regression would
leave the named draw-order test green.

## Nitpicks

None.

## Not found

No correctness defects were found in the implemented parser, serializer,
background selector, recursive shape-tree walk, placeholder matching, or latent
policy. No contract, panic, OOXML preservation, or structural findings were
found beyond the test-coverage items above.
