# F-126, all aspects, pass 1

**Reviewed**: the complete working-tree diff against `HEAD`, 2 files and 1,468
changed lines, comprising 1,425 additions and 43 deletions
**Verdict**: 10 defects, 0 smells, 0 nitpicks

## Defects

### D1, sparse scatter points that are not rendered still change both axis scales
`crates/rpptx-chart/src/lib.rs:356`

`scatter_domains` collects every cached x value and every cached y value
independently. F-125 pairs scatter points only where their logical indexes
match, at `crates/rpptx-chart/src/lib.rs:1348`. A valid sparse series with an x
point at index 2 and no y point at index 2 therefore expands the x axis even
though no geometry uses that point. The resulting ticks and the positions of
the paired points no longer describe the rendered data.

### D2, data labels ignore explicit scale bounds and reversed axes
`crates/rpptx-chart/src/lib.rs:917`

Plot paths receive `GeometryOptions`, but `data_label_origin` recomputes the raw
data domains and calls the unoriented `map_x` and `map_y` helpers. Any explicit
minimum, explicit maximum, or `maxMin` orientation moves the plot geometry and
ticks without moving its data labels. The labels detach from their points, and
labels for values clipped by explicit bounds can remain visible outside the
plot.

### D3, default data-label anchors do not follow the rendered plot geometry
`crates/rpptx-chart/src/lib.rs:940`

Pie and doughnut anchors use the individual value fraction as an x offset and
ignore cumulative slice angles, `firstSliceAng`, and the doughnut radius. Bar
anchors ignore the series index, cluster width, overlap, and stacked
accumulation, so labels for clustered series overlap and stacked labels point
at the unstacked value. Radar labels use category x positions and Cartesian y
mapping instead of the radial point emitted by the plot renderer. These are
wrong even with default axis bounds and orientation.

### D4, a labelled sparse scatter chart errors on an intentionally unpaired point
`crates/rpptx-chart/src/lib.rs:929`

F-125 defines sparse scatter geometry as the intersection of x and y logical
indexes. The label pass instead iterates every y point and returns
`MissingElement` when the same index has no x point. Adding plot-level labels
to a valid sparse chart therefore makes `render_chart` fail, although the
geometry renderer correctly skips that unpaired point.

### D5, ordinary line charts force zero into the value-axis domain
`crates/rpptx-chart/src/lib.rs:332`

The line branch derives its extent through `domain_from_layers`, whose
accumulator starts at zero at `crates/rpptx-chart/src/lib.rs:1927`. A standard
line series spanning 1000 to 1010 consequently receives an axis starting at
zero. The approved plan and updated HLD reserve zero inclusion for ordinary bar
and area value axes, so this compresses ordinary line geometry and labels into
a small part of the plot.

### D6, valid extreme and subnormal finite domains fail nice-number scaling
`crates/rpptx-chart/src/lib.rs:1833`

Rounding an unpinned bound multiplies a floored quotient by the nice step. For
an extent near `-f64::MAX` to `f64::MAX`, the chosen `1e308` step expands the
bounds to about plus or minus `2e308`, then rejects the resulting infinities.
At the other end, `safe_step` can underflow a subnormal finite range to zero.
The contract says large and fractional finite domains produce increasing
finite ticks, but the focused test reaches only `1e12`.

### D7, a valid one-sided explicit bound can make the renderer reject the chart
`crates/rpptx-chart/src/lib.rs:1824`

When only one scaling bound is pinned, the other requested bound is copied
directly from the data extent before ordering is checked. A chart with data 0
to 10 and an explicit minimum of 20 is valid scale state, but this code reports
that 20 is not less than 10 instead of deriving an automatic maximum above the
pinned minimum and clipping the data. The symmetric problem exists for a lone
maximum below the data minimum. The explicit-bound test supplies both bounds,
so neither case is exercised.

### D8, percentage-label aggregation can silently overflow to a false zero
`crates/rpptx-chart/src/lib.rs:874`

The label pass uses an unchecked `sum::<f64>()` for positive values. Two finite
values near `f64::MAX` produce an infinite total, after which each percentage
is formatted from `value / total` as zero percent. F-125 already uses
`checked_geometry_sum` for the equivalent aggregate. The labelled renderer
must return a contextual numeric error rather than emit deterministic but
incorrect text.

### D9, individual point label overrides from the approved contract are ignored
`crates/rpptx-chart/src/lib.rs:859`

The approved approach includes requested point data labels, and F-123 leaves
individual `c:dLbl` payloads preserved until a rendering story needs them. This
loop consults only the series or plot collection-level `CT_DLbls` fields. A
point-level delete, visibility override, number format, separator, or position
has no effect, so the renderer does not implement that part of the approved
contract.

### D10, the model-state test does not prove gridline, tick, or exact z-order behavior
`crates/rpptx-chart/src/lib.rs:10325`

The pre-plot assertion is true for an empty slice, so removing all major
gridlines still passes. The post-plot assertion accepts any path, including an
axis line or legend swatch, so removing every tick mark also passes. It checks
only that some path occurs after the plot, not that the required order is
gridlines, plot, axis lines, ticks, then text. The planned test claims all three
behaviors, and this is the only focused model-state test.

## Smells

None.

## Nitpicks

None.

## Not found

- Panics: the reviewed rendering paths contain no reachable indexing, slicing,
  `unwrap`, or `expect` panic for a validated typed plot. The two `expect`
  calls in data-label placement are guarded by matching plot variants.
- OOXML: this diff does not change parsing or serialization. No namespace,
  schema-order, whitespace, or raw-preservation defect was found.
- Structure: no new file, module, trait, generic parameter, dynamic dispatch,
  forwarding wrapper, feature, or dependency was introduced. The helpers stay
  concrete and local to the existing crate root.
- Deterministic shaping and font collection: all emitted text goes through the
  caller-supplied `FontManager`, and the raster test carries that manager's font
  data into the layout result. No separate font-collection defect was found.
- Legend shell behavior: presence produces one deterministic placeholder
  swatch and shaped series name per row in the documented upper-right default.
  No defect was found within the deliberately opaque placement boundary.
- Clipping and top-level z-order: the plot paths are grouped under the plot
  rectangle clip, and the top-level order is gridlines, clipped plot, path
  annotations, then text. The test weakness above does not change that current
  implementation order.
