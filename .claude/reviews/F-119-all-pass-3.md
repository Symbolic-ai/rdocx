# F-119, all, pass 3

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3
implementation files, 1,849 insertions and 61 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, series children bound through a second ChartML prefix are ignored

`crates/rpptx-chart/src/lib.rs:398`

The series reader resolves a child by comparing its prefix with the root
prefix or with a declaration on that child. It does not use the namespace
bindings already collected from the `c:ser` root. A valid series such as a
`q:ser` root that declares both `q` and `c` as ChartML, then uses `c:idx`,
`c:order`, and `c:val`, preserves those children as opaque and fails with a
missing `c:idx` error. The same prefix-only dispatch is used by the new
reference, cache, wrapper, and point readers. Resolve modeled children by
namespace URI through inherited bindings and add a mixed-alias regression.

### D2, the plot-area projection treats an inherited foreign alias as ChartML

`crates/rpptx-chart/src/lib.rs:1701`

`CT_PlotArea` retains only raw plot bytes, so a namespace declaration inherited
from `c:chartSpace` is absent when `parse_plot_series` examines a plot root.
The projection selects a supported plot from its local name before resolving
that root's namespace, and `chart_root_prefix` accepts an undeclared prefix.
For example, an `x:barChart` whose `xmlns:x="urn:producer"` binding lives on
the chart-space root is treated as ChartML, and an `x:ser` below it can be
returned as a typed series instead of remaining foreign payload. Carry the
ancestor bindings into the plot-area shell and resolve both the plot and its
series children by namespace URI.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 2 cache parse-state smell is fixed. No further contract, panic,
OOXML ordering, cache consistency, preservation, test-gate, public-surface, or
structural findings were found.
