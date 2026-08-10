# F-126, all aspects, pass 2

**Reviewed**: the complete working-tree diff against `HEAD`, 3 files and 2,680
changed lines, comprising 2,499 additions and 181 deletions. Pass 1 and every
D1 through D10 remediation were checked. `cargo test -p rpptx-chart` passed all
62 unit tests and 0 doc tests.
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, percentage aggregation rejects labels that do not request a percentage
`crates/rpptx-chart/src/lib.rs:914`

The positive-value total is checked for every labelled series before the code
tests `show_percent`. A clustered series containing two finite values near
`f64::MAX` and labels that request only `showVal` therefore fails with a
percentage-total error, even though no percentage is rendered and each value
can be formatted independently. The checked aggregate must be conditional on
an effective collection or point label requesting a percentage.

### D2, percentage labels ignore their declared number format
`crates/rpptx-chart/src/lib.rs:947`

The percentage path always constructs `0%` instead of using the effective
label collection's `number_format`. A `showPercent` label with `0.00%` therefore
renders `33%` for one third instead of `33.33%`. This contradicts the HLD rule
that data-label values use the declared supported `NumberFormat` subset, and it
also makes a point-level number-format override ineffective for percentage
labels.

### D3, sparse bubble-size labels use vector position as the logical cache index
`crates/rpptx-chart/src/lib.rs:951`

The value and category paths retain sparse `c:pt/@idx` values, but bubble-size
lookup calls `size.values.get(logical_index)`. For a cache whose two stored
points have logical indexes 0 and 3, the label for point 3 loses its bubble
size because the dense vector has no element at position 3. Other sparse
layouts can select the wrong stored value. Bubble sizes must be paired through
the cache layout by logical index like the value and category caches.

### D4, category ticks and gridlines disappear when category text is absent or sparse
`crates/rpptx-chart/src/lib.rs:475`

The category-axis loop emits gridlines and tick marks only while iterating the
first series' present category labels. A valid series with values but no
category cache renders its plot against category slots yet emits no category
ticks or requested gridlines. A sparse category cache also omits ticks at its
empty logical slots. Tick and gridline positions must come from the logical
category count. Label emission can remain conditional on a cached label at
that index.

### D5, data-label positions are not interpreted relative to their plot geometry
`crates/rpptx-chart/src/lib.rs:1373`

`InsideBase`, `InsideEnd`, and `OutsideEnd` are reduced to fixed vertical
offsets from the mark centre. On a horizontal bar, `OutsideEnd` moves the label
up instead of beyond the bar endpoint. On negative columns it moves toward the
wrong end, and on pie or doughnut slices it does not move radially. The HLD
claims that the declared position is projected from the same family geometry,
so these values need family, direction, sign, and orientation-aware origins.

### D6, unsupported numeric category formats are silently replaced with raw number text
`crates/rpptx-chart/src/lib.rs:865`

Numeric category labels discard both `NumberFormat::new` and `format_value`
errors and fall back to `f64::to_string`. A valid producer format outside the
implemented subset therefore renders misleading General-like text instead of
returning the contextual projection error required by the documented
`NumberFormat` boundary. The same renderer already propagates this error for
numeric ticks and ordinary data-label values.

### D7, the point-override behavior test does not exercise number format or separator overrides
`crates/rpptx-chart/src/lib.rs:12698`

The indexed override declares `0.0` and `" / "`, but it disables `showVal` and
enables only one output field. The expected text is therefore just `South`.
Removing both number-format and separator application from
`PointLabelOverride::apply_to` leaves this test passing. The approved test plan
requires those two override behaviors to affect only their indexed label, so
the test must request at least two fields including a formatted numeric field
and assert their exact joined text.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D1 through D10: all original triggers have dedicated remediation. No
  recurrence was found for matched sparse scatter domains, shared family
  anchors, skipped unpaired scatter labels, standard-line domains, finite
  extreme and subnormal scales, one-sided explicit bounds, checked percentage
  overflow, private point override parsing, or exact top-level annotation
  ordering.
- Nice-number scaling: the 0 through 100 gate, opposite finite extremes,
  subnormal extents, constants, and one-sided explicit bounds produce finite
  increasing ticks in the reviewed cases.
- Explicit and reversed bounds: the shared axis mappings reach plot geometry
  and label anchors, and anchors outside the plot rectangle are removed.
- Blank handling: `Gap`, `Zero`, and `Span` retain their F-125 geometry
  behavior. Sparse scatter axes use the same rendered pair set as geometry,
  while markers and labels remain restricted to real matched points.
- OOXML and preservation: point overrides resolve ChartML by namespace URI,
  reject duplicate or malformed modelled values and schema-order violations,
  ignore foreign lookalikes, and leave the captured `c:dLbl` bytes as the only
  serialization source.
- Z-order: the implementation and focused test distinguish nonempty
  gridlines, the single clipped plot group, axis lines, tick marks, legend
  swatches, and text in the required order.
- Panics: no reachable production panic was found. The indexed and sliced
  accesses reviewed are guarded by equal-length construction or nonempty
  ranges.
- Structure: no new file, module, trait, generic parameter, dynamic dispatch,
  forwarding wrapper, feature, or dependency was introduced.
- Deterministic shaping: every emitted label and legend string uses the
  caller-supplied `FontManager`, and the raster test carries its font data into
  the layout result.
