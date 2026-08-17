# F-158, correctness, pass 1

**Reviewed**: working-tree diff, 8 files, 560 insertions, 287 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, shared-source regression does not compare formulas or text workbook cells
`crates/rdocx/src/document.rs:3937`

The approved test contract says this regression proves that ChartML formulae
and caches match embedded worksheet cells for every authored series. The chart
helper assertion checks cached names, categories, values, and number formats,
but not their formula ranges. The worksheet loop checks only six numeric data
cells, not the category cells or series headers. A defect that writes a stale
or wrong formula while preserving the cache, or writes categories and headers
to the wrong workbook cells, would pass this test.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML ordering and namespace behavior, and
structure produced no other findings. The public Word mutation is staged
atomically, the shared helper preserves the existing Presentation behavior,
and the test-only typed source keeps the SHA-bound F-157 candidate unchanged.
