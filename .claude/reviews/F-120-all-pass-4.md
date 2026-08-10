# F-120, all, pass 4

**Reviewed**: `git diff --working` against claim commit `696d464`, one tracked
file with 1,860 changed lines, comprising 1,830 additions and 30 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, axis equality compares lexical spelling instead of normalized ids

`crates/rpptx-chart/src/lib.rs:3224`

`axis_id_markup_eq()` compares the emitted lexical identifier strings after
the public `AxisId` values have already compared equal. Two parsed axes whose
ids use valid alternative spellings such as `1` and `01` therefore compare
unequal even though both normalize to the same identifier. This contradicts
the approved contract that preserves an unchanged producer spelling for output
while using one normalized identifier domain for equality and pairing. The
scalar preservation markup can participate in equality, but the private
`lexical` spelling must not override equal normalized ids.

## Smells

None.

## Nitpicks

None.

## Not found

All prior findings are remediated. Parsed axes reject unsafe root relabelling,
constructed and reparsed axes compare structurally equal, and fresh axes permit
safe public kind, id, cross-axis, and scaling edits through serialize-reparse.
No other correctness, contract, panic-safety, OOXML namespace, schema-order,
raw-preservation, graph-validation, test, or structural finding was found.
