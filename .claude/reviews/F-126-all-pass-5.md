# F-126, all aspects, pass 5

**Reviewed**: the complete working-tree diff against `HEAD`, 3 files and 9,516
changed lines, comprising 6,503 additions and 3,013 deletions. Pass 1 through
pass 4 and every pass-4 D1 through D4 remediation were checked. `cargo test -p
rpptx-chart` passed all 72 unit tests and 0 doc tests. Focused clippy, formatting,
diff, and prose checks also passed.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, degenerate retained anchors lose the family direction
`crates/rpptx-chart/src/lib.rs:1505`

`LabelAnchor::origin` falls back to fixed vertical offsets when `base` and
`end` coincide. The clipped bar construction can create exactly that anchor at
`crates/rpptx-chart/src/lib.rs:1565`. For example, a column whose value equals
the explicit minimum is retained at the plot boundary, but its off-plot zero
baseline clamps to the same point. `InsideBase` then moves eight points below
the plot for a normal vertical axis. With reversed orientation, `OutsideEnd`
moves upward even though the original bar direction points downward. A
horizontal boundary bar also moves vertically instead of along its value axis.
Radar has the same failure at `crates/rpptx-chart/src/lib.rs:1639` when a value
at the domain minimum maps to the centre. Its outside position always moves up
rather than following the point's category spoke. Exact-bound and zero-radius
labels therefore do not follow the clipped bar or radial family geometry.

### D2, high radar category labels can escape the chart bounds
`crates/rpptx-chart/src/lib.rs:684`

The high category position uses `radius + 18` even though F-125 reserves only
12 points above the plot. With the focused 200 by 140 chart bounds, the plot
top is 12, its centre is at y 62, and its radius is 50. The North label origin
therefore becomes y -3. The focused expectation at
`crates/rpptx-chart/src/lib.rs:11584` currently locks in that out-of-chart
coordinate instead of detecting it. A narrow plot can similarly place a right
spoke label beyond the 12-point right margin. `tickLblPos="high"` is distinct,
but its glyph run is not positioned inside the chart's reserved label space.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-4 D1: ordinary in-range bars now derive centre, base, and end from the
  clipped visible segment. D1 records the remaining exact-bound degeneracy.
- Pass-4 D2: a nonempty radar cache whose points are all outside explicit
  bounds now returns an empty clipped plot group while retaining its axes and
  other annotations.
- Pass-4 D3: default radar domains include zero and retain distinct all-negative
  and mixed-sign values. Normal and reversed explicit domains suppress only
  the out-of-range points and labels.
- Pass-4 D4: radar value and category label positions now distinguish high,
  low, next-to-axis, and hidden states. D2 records the separate chart-boundary
  failure in the high category position.
- Scale selection: the 0 through 100 gate, constants, one-sided bounds,
  subnormal values, and opposite finite extremes produce increasing finite
  ticks. Standard line and scatter domains do not force zero.
- Sparse and bounded caches: scatter domains use matched logical points,
  bubble labels join by logical index, and category annotation expansion stops
  above 16,384 slots.
- Point-label projection and OOXML preservation: aliases resolve by namespace
  URI, malformed and duplicate modelled values and schema-order violations are
  rejected, foreign lookalikes are ignored, and raw `c:dLbl` subtrees remain
  the only serialization source.
- Percentage labels: aggregation remains conditional on an effective
  percentage request, overflow is contextual, and effective number formats
  control precision.
- Panics: no reachable production indexing, slicing, `unwrap`, or `expect`
  panic was found in the reviewed rendering and projection paths.
- Structure: no new file, module, trait, generic parameter, dynamic dispatch,
  forwarding wrapper, feature, or dependency was introduced.
- Deterministic shaping and z-order: all text uses the caller's `FontManager`,
  and the top-level order remains gridlines, clipped plot, axis lines, ticks,
  legend swatches, then text.
