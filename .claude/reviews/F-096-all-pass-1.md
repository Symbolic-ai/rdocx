# F-096, all aspects, pass 1

**Reviewed**: Working-tree diff, 5 tracked files, 1,469 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, counter-rotated picture content does not cover the rotated clip

`crates/rpptx-render/src/lib.rs:313`

Stretch content is counter-rotated while its destination remains the original
local shape rectangle. At any non-quarter-turn angle, the rotated picture clip
has a larger axis-aligned extent than that unrotated image, so corners of the
picture geometry have no image content. The uncropped rectangular case also
skips a picture-shape clip entirely, which lets the unrotated image remain
visible outside the rotated outline. Tile content has the same coverage gap
because rows and columns are generated only over the original local bounds
before the inverse transform at `crates/rpptx-render/src/lib.rs:409`. The
rotation regression at `crates/rpptx-render/src/lib.rs:1803` asserts only the
presence of the inverse transform and cannot detect either visible failure.

## Smells

None.

## Nitpicks

None.

## Not found

Contract, panic safety, OOXML boundary handling, source-scope resolution, crop
arithmetic, tile ordering and bounds, DPI precedence, test reversion strength
outside D1, and structural-rule violations produced no findings. The large
renderer diff remains cohesive private lowering and test code in the existing
module. It introduces no trait, generic, forwarding wrapper, feature flag,
crate, module, or file split.
