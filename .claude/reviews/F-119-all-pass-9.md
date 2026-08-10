# F-119, all, pass 9

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3 tracked
files, 2,208 insertions and 60 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 8 series-slot defect is fixed. Schema-defined marker, inversion,
picture, explosion, point, label, trendline, and error-bar payloads remain
after newly inserted `c:tx` and `c:spPr` and before newly inserted `c:cat`.
Schema-final shape, smoothing, and extension payloads remain around edited
`c:val` and `c:bubbleSize` fields in their stable slots. Existing raw payload
order is retained within each slot.

The slot routing covers the category-based area, bar, line, pie, radar, stock,
and surface series families projected by F-119. Foreign elements whose local
names resemble ChartML series children are not assigned ChartML schema slots.
They retain the boundary established by surrounding modelled children.

All earlier namespace, foreign-shape, duplicate-wrapper, sparse-cache,
cache-tail, public opaque-wrapper, parse-state, and structural remediations
remain intact. No additional correctness, contract, panic, OOXML namespace,
schema-order, preservation, cache-consistency, test-gate, public-surface, or
structural findings were found.
