# F-087, correctness, pass 3

**Reviewed**: working-tree diff, 7 files, 1,753 insertions and 12 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

The group affine now follows the documented translate, scale, translate,
rotate, then centre-flip chain. Its regression combines rotation and one flip,
which do not commute. Character and auto-number bullets both retain the
independently inherited font, concrete colour, size, and choice values.

No additional defects were found in resolution correctness, contract coverage,
panic handling, OOXML ordering, corpus traversal, tests, dependency direction,
or source structure.
