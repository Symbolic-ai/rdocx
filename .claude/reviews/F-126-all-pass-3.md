# F-126, all aspects, pass 3

**Reviewed**: the complete working-tree diff against `HEAD`, 3 files and 7,992
changed lines, comprising 5,396 additions and 2,596 deletions. Pass 1, pass 2,
and every pass-2 D1 through D7 remediation were checked. `cargo test -p
rpptx-chart` passed all 67 unit tests and 0 doc tests.
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, a large declared category count causes unbounded annotation work
`crates/rpptx-chart/src/lib.rs:486`

The category-axis path loops over every slot from zero through the untrusted
cached `c:ptCount` and can append both a gridline and a tick for each one. The
parser accepts a sparse cache whose declared count is any `u32`, so a one-point
cache with `ptCount="4294967295"` makes labelled rendering attempt billions of
iterations and allocations. The adjacent geometry contract deliberately
compresses sparse zero controls to avoid allocating every slot of an untrusted
count. Annotation projection needs an explicit bounded failure instead of
reintroducing that resource-exhaustion path.

### D2, category-axis number formats do not control numeric tick labels
`crates/rpptx-chart/src/lib.rs:425`

Category labels are projected before the axis loop through `series_categories`,
which formats numeric categories from the cache's `c:formatCode` at
`crates/rpptx-chart/src/lib.rs:876`. The category axis's typed
`axis.number_format` is never consulted. A numeric category cache using
`General` with an axis format of `0.00` therefore emits `1` instead of `1.00`.
An unsupported category-axis format is also silently ignored when the cache
format is supported. The pass-2 test covers only an unsupported cache format,
not the effective axis format.

### D3, explicit bounds suppress in-range bar labels and can retain out-of-range ones
`crates/rpptx-chart/src/lib.rs:1420`

Bar anchors are retained according to the centre of the complete unclipped bar
rectangle, not the data endpoint. With an explicit value domain of 50 through
100, a column ending at 75 starts from the off-plot zero baseline, so its full
rectangle centre is below the plot and its valid label is removed. A column
ending at 125 can have that centre inside the plot even though its data endpoint
is above the explicit maximum, so its label is retained. This contradicts the
documented rule that anchors outside an explicit axis range do not emit labels.

### D4, inside positions leave short bars and thin doughnut rings
`crates/rpptx-chart/src/lib.rs:1274`

`InsideBase` always advances five points from the base and `InsideEnd` always
retreats five points from the end at
`crates/rpptx-chart/src/lib.rs:1280`. Neither displacement is limited by the
segment length. On a four-point bar, the two positions land past the opposite
end. The same failure occurs on a doughnut whose visible radial thickness is
less than five points. The labels follow the segment direction, but they do not
remain inside the rendered family geometry as their declared positions require.

### D5, zero-valued bars cannot emit requested value labels
`crates/rpptx-chart/src/lib.rs:1713`

Zero-width or zero-height bar rectangles are discarded before anchors are
built. `render_data_labels` then skips any value without an anchor at
`crates/rpptx-chart/src/lib.rs:930`. A bar chart that requests value labels thus
omits `0` for every zero-valued point. If every bar is zero, the earlier empty
plot check rejects the otherwise valid cached chart before its axes or labels
can render.

### D6, the public geometry entry point still forces zero into standard line domains
`crates/rpptx-chart/src/lib.rs:222`

`render_geometry` continues to pass default geometry options. The standard line
path consequently falls back to `domain_from_layers` at
`crates/rpptx-chart/src/lib.rs:1746`, whose lower endpoints include zero. A line
with values 1000 and 1010 is still compressed against a 0 through 1010 domain
through this public entry point. The pass-1 remediation fixed the labelled
entry point and added a direct helper test, but it did not fix the public
geometry behavior covered by the HLD statement that standard line domains do
not force zero.

### D7, radar axes and gridlines are emitted as Cartesian annotations
`crates/rpptx-chart/src/lib.rs:439`

The single generic axis loop draws every axis on a rectangular plot edge and
draws category gridlines as vertical or horizontal segments. A radar plot is a
centred radial polygon, so its category annotations need spokes and perimeter
labels, while value gridlines need concentric radial geometry. The current
output places a bottom or side axis and Cartesian grid over the radar marks.
Those annotations do not describe the geometry they label.

### D8, radar values outside explicit bounds wrap across the plot centre
`crates/rpptx-chart/src/lib.rs:2097`

Radar radius uses the oriented normalized value without rejecting or clipping
values outside the explicit axis domain. With a 50 through 100 minimum-to-
maximum scale, a value of 25 produces a negative radius and is drawn on the
opposite side of the centre. Under reversed orientation, values above the
maximum have the same failure. The resulting point can remain inside the
rectangular plot and survive the label-anchor filter, so both geometry and its
label remain visible at an unrelated category direction instead of being
suppressed as out of range.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-2 D1 and D2: percentage aggregation is now lazy per effective label, and
  collection or point number formats control percentage precision.
- Pass-2 D3: bubble sizes now join through preserved logical cache indexes.
- Pass-2 D4: bounded sparse and unlabeled category caches emit ticks and
  gridlines for every logical slot while emitting text only for present labels.
  D1 records the remaining unbounded-count case.
- Pass-2 D5: ordinary horizontal, negative, reversed, pie, and doughnut cases
  now derive label direction from shared family geometry. D3, D4, and D8 record
  the remaining clipping and short-segment cases.
- Pass-2 D6: unsupported numeric cache formats now return contextual projection
  errors. D2 records the separate effective category-axis format failure.
- Pass-2 D7: the point-override test now proves indexed number-format and
  separator application with exact joined text.
- Point overrides and OOXML preservation: ChartML aliases resolve by namespace
  URI, malformed or duplicate modelled values and schema-order violations are
  rejected, foreign lookalikes are ignored, and the original raw `c:dLbl`
  subtrees remain the only serialization source.
- Sparse scatter and bubble layouts: axis domains and label anchors use matched
  logical x and y indexes, and bubble label values use their own logical index
  map.
- Nice-number scaling: the reviewed opposite finite extremes, subnormal values,
  constants, and one-sided explicit bounds produce finite increasing ticks.
- Panics: no reachable indexing, slicing, `unwrap`, or `expect` panic was found
  in the reviewed production paths. D1 is an unbounded resource-exhaustion
  defect rather than a direct Rust panic.
- Structure: no new file, module, trait, generic parameter, dynamic dispatch,
  forwarding wrapper, feature, or dependency was introduced.
- Deterministic shaping and z-order: emitted text uses the caller's
  `FontManager`, and the top-level order remains gridlines, the clipped plot,
  axis lines, ticks, legend swatches, then text.
