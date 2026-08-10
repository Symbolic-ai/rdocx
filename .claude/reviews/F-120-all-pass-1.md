# F-120, all, pass 1

**Reviewed**: `git diff --working` against claim commit `696d464`, one tracked
file with 1,746 changed lines, comprising 1,716 additions and 30 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, changing the public axis kind can retain invalid type-specific children

`crates/rpptx-chart/src/lib.rs:2119`

`Axis::kind` is publicly mutable, while the type-specific axis tail remains in
private raw preservation state. Parsing a category axis that contains `c:auto`,
then changing `kind` to `AxisKind::Value`, causes `to_xml()` to write a
`c:valAx` root while retaining the category-only `c:auto` child. The result
violates the `CT_ValAx` sequence and can trigger a PowerPoint repair. The writer
needs either an immutable root kind or validation that rejects a kind change
when preserved type-specific content belongs to the parsed root.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness defects were found. Contract coverage, panic safety,
ChartML namespace handling, fixed-prefix output, common child ordering, raw XML
preservation, the focused and corpus tests, and the repository structural rules
produced no additional findings.
