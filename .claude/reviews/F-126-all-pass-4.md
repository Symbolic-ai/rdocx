# F-126, all aspects, pass 4

**Reviewed**: the complete working-tree diff against `HEAD`, 3 files and 9,330
changed lines, comprising 6,313 additions and 3,017 deletions. Pass 1 through
pass 3 and every pass-3 D1 through D8 remediation were checked. `cargo test -p
rpptx-chart` passed all 70 unit tests and 0 doc tests. Focused clippy, formatting,
diff, and prose checks also passed.
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, retained bar labels can still be positioned outside the visible bar
`crates/rpptx-chart/src/lib.rs:1537`

Pass-3 D3 now retains a bar anchor according to its data endpoint, but the
anchor centre, base, and end still come from the complete unclipped rectangle.
With an explicit domain of 80 through 100 and a column ending at 90, the zero
baseline maps far below the plot. The endpoint passes the retention check, but
the default centre and `InsideBase` origin remain below the plot, so the label
is retained only to be emitted outside the visible chart. The anchor segment
must reflect the clipped visible bar before its centre and inside positions are
derived.

### D2, suppressing every out-of-range radar point makes a valid chart fail
`crates/rpptx-chart/src/lib.rs:158`

Pass-3 D8 skips radar points outside an explicit domain. If every cached point
is outside that domain, `render_radar_geometry` returns no children and this
generic empty-plot check rejects the chart before axes can render. The caches
are not empty, and the documented explicit-bound behavior requires those
points and labels to be suppressed. A radar chart with all values clipped by
50 through 100 should therefore render its annotations without marks instead
of returning `plot has no renderable cached data`.

### D3, negative radar domains are rejected or collapsed to zero
`crates/rpptx-chart/src/lib.rs:2281`

The radar projection still defines renderability through a maximum folded from
zero, so an all-negative finite radar cache is rejected even though its value
axis has a valid negative linear domain. In a mixed-sign domain, the later
`value.max(0.0)` maps every distinct negative value to the same zero-value
radius. This contradicts the approved negative and mixed-sign scale behavior,
and makes plotted marks disagree with the negative tick values emitted by the
same axis.

### D4, radar tick-label high and low positions have no effect
`crates/rpptx-chart/src/lib.rs:618`

The radar annotation path distinguishes only `None` from every visible
`TickLabelPosition`. `High`, `Low`, and `NextTo` therefore emit value labels at
the same origin. Category labels have the same issue at
`crates/rpptx-chart/src/lib.rs:660`. The HLD states that `c:tickLblPos` moves
labels to the high or low side or suppresses them, so valid radar axes do not
honor their model state.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-3 D1: category and radar annotation expansion now rejects logical counts
  above 16,384 before the annotation loops allocate per-slot output.
- Pass-3 D2: the effective category-axis number format now controls numeric
  category tick labels, including contextual errors for unsupported formats.
- Pass-3 D3: bar anchor retention now follows the data endpoint. D1 records the
  remaining origin and visible-segment problem.
- Pass-3 D4: fixed inside displacements are now clamped to the segment midpoint
  for short bars and thin doughnut rings.
- Pass-3 D5: zero-valued bars retain degenerate geometry and label anchors, and
  an all-zero bar chart renders requested zero labels.
- Pass-3 D6: the public geometry entry point now derives a standard-line domain
  from rendered values without forcing zero.
- Pass-3 D7: radar annotations now use category spokes, perimeter labels, and
  concentric value gridlines instead of Cartesian axes.
- Pass-3 D8: normal and reversed explicit radar domains now discard each
  individual out-of-range point and its label. D2 records the all-discarded
  case.
- Point-label projection and OOXML preservation: namespace aliases, malformed
  and duplicate modeled values, schema order, foreign lookalikes, and exact raw
  subtree preservation remain covered and correct in the reviewed diff.
- Percentage and sparse-cache behavior: percentage aggregation remains lazy,
  effective formats control precision, and bubble sizes join by logical index.
- Panics: no reachable indexing, slicing, `unwrap`, or `expect` panic was found
  in the reviewed production paths.
- Structure: no new file, module, trait, generic parameter, dynamic dispatch,
  forwarding wrapper, feature, or dependency was introduced.
- Deterministic shaping and z-order: text uses the caller's `FontManager`, and
  the top-level order remains gridlines, clipped plot, axis lines, ticks,
  legend swatches, then text.
