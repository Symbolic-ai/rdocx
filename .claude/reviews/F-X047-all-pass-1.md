# F-X047, all aspects, pass 1

**Reviewed**: Working-tree diff, 8 files, 374 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the PDF assertion does not prove that the carrier is invisible

`crates/rdocx/tests/regression_test.rs:5249`

The PDF comparison renders the ordinary and provenance results. Both results
contain the new empty segment, so the assertion still passes if the PDF backend
emits new operators for that segment. Compare a result containing the carrier
with the existing `without_empty` result, as the raster assertion does.

### D2, the resolved-metrics test checks identity and size but not metrics

`crates/rdocx/tests/regression_test.rs:5213`

The assertion observes font family and point size only. It would pass if the
empty segment carried zero, stale, or unrelated ascent and descent values. The
approved test contract requires direct and style defaults to choose the caret
font and its metrics. Inspect the paragraph block and compare the segment
metrics with the resolved font metrics.

### D3, paragraph-mark decoration can create backend-visible elements

`crates/rdocx-layout/src/engine.rs:3572`

An otherwise empty paragraph whose paragraph mark has highlight, underline,
strike, or double strike copies those properties to the carrier. Pagination
then emits filled rectangles or line elements even though the carrier has no
glyphs and zero width. This violates the backend-invisibility contract and can
change PDF bytes. Keep font-selection properties for metric resolution, but do
not attach paint decorations to the empty semantic carrier. Add a formatted
empty-paragraph case to the backend assertion.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML preservation,
tests, or structure.
