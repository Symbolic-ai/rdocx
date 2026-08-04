# F-098a, all aspects, pass 2

**Reviewed**: working implementation diff, 3 files, 185 additions and 5 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML hazards, tests, and structure produced no
findings. Pass 1 D1 is resolved because the fallback regression now exercises
both rectangle and bounds-fallback geometry. The test rectangle remains local
to the shape, all four insets affect the expected coordinates, and oversized
insets retain their origin while clamping both extents to zero.
