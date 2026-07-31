# F-043, correctness, pass 2

**Reviewed**: remediated working diff, 5 tracked files, 665 additions and 51 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the page-local resource test does not distinguish the pages

`crates/oxml-pdf/src/writer.rs:1728`

The test counts two `/Pattern` dictionaries and checks that `/P0` and `/P1`
occur somewhere in the whole PDF. It still passes if both page dictionaries
contain both pattern names, so reverting the page filter at
`crates/oxml-pdf/src/writer.rs:704` would not fail the regression named in the
test plan. Split out the two page pattern dictionaries and assert the first
contains only `/P0` while the second contains only `/P1`.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, or structure findings. The
pass 1 coordinate defect is fixed, and the exact sampled colours now prove the
declared transform direction.
