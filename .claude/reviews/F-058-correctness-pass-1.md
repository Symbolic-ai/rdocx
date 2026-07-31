# F-058, correctness, pass 1

**Reviewed**: working tree, 2 files and 738 added lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, valid multi-turn sweeps are rejected instead of segmented

`crates/oxml-drawing/src/geometry.rs:524`

The evaluator rejects every finite sweep above one full circle. The approved
contract says to split sweeps into segments of at most 90 degrees and does not
limit the input to one turn. `ST_AdjAngle` can carry an evaluated guide value,
so a 450 degree sweep reaches this branch even though five finite cubic
segments would represent it. Remove the one-turn restriction and retain a
bounded allocation check only outside the schema-representable angle range.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.
