# F-165, all, pass 2

**Reviewed**: remediated uncommitted worker diff, 8 files, 555 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: all repeated body and row placements validate numbering before
  evaluation, and multi-row iteration retains source order.
- Contract: the behavior remains within `Document::render_template`, with no
  public surface, type, file, dependency, or feature-flag addition.
- Panics: production changes add no input-reachable panic, unchecked indexing,
  slicing, or arithmetic.
- OOXML: table owners stay in place. Cloned rows retain merge properties,
  content controls, and raw row, cell, and cell-property XML in schema order.
- Tests: the exact thirty-row gate fails on claim base because it includes the
  missing numbering-reference rejection. Continuous numbering and round-trip
  preservation have direct reopened-document assertions.
- Structure: the private numbering query has one concrete use and the recursive
  validation mirrors the existing typed content variants without a new trait,
  generic, wrapper type, or module.
