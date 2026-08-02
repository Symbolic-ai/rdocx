# F-087, correctness, pass 2

**Reviewed**: working-tree diff, 7 files, 1,709 insertions and 12 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, group flips and rotation compose in the wrong order

`crates/rpptx-layout/src/context.rs:1388`

`group_affine` applies the centre flip to the base mapping and then applies the
rotation. The HLD contract applies the group rotation before the centre flip.
The two operations do not commute for a group that combines a nonzero rotation
with one flip, so its accumulated affine places grouped leaves on the wrong
side of the rotation axis. The focused nested-group regression covers scale and
translation only and therefore cannot detect this case.

### D2, auto-number bullets discard effective font and colour

`crates/rpptx-layout/src/context.rs:899`

Bullet font, colour, size, and choice are independent inherited properties.
The resolver computes the effective font and concrete colour, but only stores
them on the character-bullet branch. `ResolvedBullet::AutoNumber` receives the
size and drops the font and colour, so the frozen renderer contract cannot
reproduce an auto-number bullet whose bullet styling differs from its text.

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in path-coordinate scaling, table text-body
resolution, paragraph spacing, gradient fallback diagnostics, package traversal,
panic handling, OOXML child order, dependency direction, or source structure.
