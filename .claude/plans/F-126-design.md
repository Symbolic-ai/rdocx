# F-126, Chart rendering: axes, gridlines and labels

**Status**: completed
**Sprint**: S31
**Size**: L
**Depends on**: F-125, F-098

## Problem

F-125 supplies plot paths but intentionally leaves the reserved plot margins
empty. The typed ChartML model already exposes scaling bounds, axis positions,
gridline presence, tick settings, number formats, data-label flags, and a
legend shell. Without a deterministic scale and text pass, chart geometry has
no readable axes, tick values, category labels, data labels, or legend.

The renderer must use cached values and current text shaping infrastructure.
It must honor explicit scaling bounds while selecting stable human-readable
ticks for ordinary linear axes.

## Spec reference

- `docs/hld/08-rendering-spec.md`, `GlyphRun`, `PathElement`, `Group`, font
  shaping, and deterministic font mode.
- `docs/hld/09-charts-spec.md`, "The ChartML model", typed axes and data
  labels, "Rendering", and scale selection.
- `docs/hld/12-testing-strategy.md`, deterministic layout and raster tests.
- `docs/hld/14-development-backlog.md`, "F-126, Chart rendering: axes,
  gridlines and labels".

## Approach

Extend the F-125 geometry entry point in the existing `rpptx-chart` crate root.
Keep one concrete `FontManager` input because it is the existing shaping
implementation and no second implementation exists:

```rust
pub fn render_chart(
    chart: &CT_Chart,
    bounds: Rect,
    fonts: &mut FontManager,
) -> Result<GroupElement>;
```

Add an internal nice-number calculation that receives the data extent and a
target tick count. It chooses a step from 1, 2, or 5 times a power of ten,
expands unspecified bounds to enclosing step multiples, preserves explicit
`c:scaling` minimum and maximum, includes zero for ordinary bar and area value
axes, and returns increasing finite ticks. The exact 0 through 100 gate yields
0, 20, 40, 60, 80, and 100.

Draw major gridlines first, then plot geometry, then axis lines and ticks, then
legend swatches, then text. Honor deleted axes, axis position, orientation,
explicit bounds, gridline presence, major tick mark, and tick-label visibility.
Format numeric ticks and data labels through the existing `NumberFormat::format_value`
subset, falling back to deterministic General formatting when no format is
declared.

Shape category labels, numeric tick labels, requested series or point data
labels, and legend series names through the passed `FontManager`. Use a fixed
default chart font family and point size when no supported text property
overrides it. Position labels in the reserved F-125 margins and clip geometry
to the plot rectangle. Treat the current opaque legend shell as visible when
present, lay out one swatch and shaped series name per row, and keep unsupported
legend placement details preserved but behaviorally defaulted.

Project individual `c:dLbl` overrides from their preserved raw children into a
private rendering-only value. Resolve ChartML by namespace URI, reject malformed
or duplicate modelled override values, and leave the captured bytes in their
original `CT_DLbls` raw slots as the only serialization source. This adds the
minimum behavior parsing needed for point-level delete, visibility, number
format, separator, and position without exposing a new public model.

## Rejected alternatives

- Emit unshaped strings. The output contract contains glyph runs, and bypassing
  shaping would break font collection and PDF text extraction.
- Put tick selection in `oxml-layout`. Nice-number scaling is chart semantics,
  not a format-neutral layout primitive.
- Add a font-provider trait or generic parameter. Only `FontManager` implements
  shaping today.
- Type the complete legend schema first. The current story needs visible
  legend entries, while unsupported legend children remain preserved for later
  refinement.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `zero_to_one_hundred_axis_uses_expected_ticks` | A 0 to 100 value axis yields exactly 0, 20, 40, 60, 80, and 100 |
| unit | `nice_number_ticks_cover_unpinned_extents` | Positive, negative, mixed, constant, fractional, and large domains produce finite ordered enclosing ticks |
| unit | `explicit_scaling_bounds_and_orientation_are_honored` | Pinned minimum or maximum and reversed orientation map ticks and geometry to exact coordinates |
| unit | `axes_gridlines_and_tick_marks_follow_model_state` | Deleted axes disappear, requested gridlines and tick forms emit strokes, and z-order is stable |
| unit | `labels_and_legend_shape_with_deterministic_fonts` | Category, numeric, data-label, and legend strings become glyph runs with reproducible positions and font ids |
| unit | `category_ticks_cover_unlabelled_and_sparse_logical_slots` | Category ticks cover bounded logical slots, text remains limited to cached labels, and oversized counts fail before expansion |
| unit | `inside_and_outside_label_positions_follow_family_geometry` | Inside and outside positions follow horizontal, negative, reversed, radial, and short-segment geometry |
| unit | `degenerate_bar_and_radar_anchors_preserve_family_direction` | Exact-bound vertical and horizontal bars preserve normal and reversed value-axis direction, while zero-radius radar anchors preserve their spoke direction |
| unit | `labels_outside_explicit_axis_bounds_are_suppressed` | Bar label retention follows the data endpoint, while centre and inside positions follow the clipped visible segment |
| unit | `zero_bars_render_and_emit_requested_value_labels` | Zero-valued bars retain anchors and an all-zero chart remains renderable |
| unit | `standard_line_domain_does_not_force_zero` | Both domain selection and public geometry spread a positive standard line without zero |
| unit | `radar_annotations_use_spokes_perimeter_labels_and_radial_gridlines` | Radar annotations use spokes, perimeter category labels, and concentric value gridlines |
| unit | `radar_tick_label_positions_are_distinct` | High, low, next-to-axis, and hidden radar tick-label positions remain distinct, while high category origins stay within standard and narrow chart bounds |
| unit | `radar_negative_and_mixed_sign_domains_preserve_values` | Default and explicit radar domains preserve all-negative and mixed-sign values without collapsing them to zero |
| unit | `radar_explicit_bounds_suppress_out_of_range_points_and_labels` | Normal and reversed explicit radar bounds suppress out-of-range geometry and labels, while a fully suppressed plot still emits annotations |
| unit | `percentage_label_aggregation_rejects_nonfinite_totals` | Percentage totals are checked only when an effective label requests them |
| unit | `percentage_labels_use_effective_number_format_precision` | Collection and point number formats control exact percentage precision |
| unit | `sparse_bubble_size_labels_resolve_logical_indexes` | Bubble-size fields join labels by preserved logical cache index |
| unit | `unsupported_numeric_category_formats_return_projection_errors` | The category-axis format controls numeric text and unsupported effective formats return contextual errors |
| unit | `point_label_overrides_render_without_changing_preserved_xml` | Aliased point overrides affect only their indexed labels, serialize in exact original order and bytes, and reparse identically |
| unit | `malformed_point_label_overrides_return_contextual_errors` | Duplicate fields, malformed indexes and booleans, invalid enums, and foreign lookalikes are rejected or ignored according to namespace |
| golden | `labelled_chart_raster_is_deterministic` | A chart with axes, gridlines, labels, and legend rasterises identically across repeated deterministic-font runs |

The test gate is: a chart with a 0 to 100 value axis produces the expected tick
set.

## HLD impact

- `docs/hld/09-charts-spec.md`

Document the nice-number rule, explicit-bound behavior, z-order, shaping input,
default label style, data-label projection, and current legend behavior.

## Risk routing

- Layout, pagination, line breaking, text shaping. Read HLD 08. Use
  deterministic font mode for every layout and raster baseline, and record any
  baseline update deliberately.
- Parser behavior. Read HLD 04 and HLD 06. Resolve aliases by namespace URI,
  reject malformed and duplicate modelled values, preserve every `c:dLbl`
  subtree byte for byte in its existing raw slot, and add round-trip and exact
  schema-order coverage alongside the rendering projection tests.

No serialiser, crate graph, published API, binding, feature, new file, external
oracle, or unit-conversion rider applies. All renderer coordinates are already
points.

## Hash harness

Expected unchanged. Native chart annotation is not reached by the Word sample
generator or renderer. All 28 hashes must match.

## Implementation checklist

- [x] Add deterministic nice-number scale and tick selection.
- [x] Render major gridlines, axis lines, and tick marks in stable order.
- [x] Honor axis deletion, position, orientation, explicit bounds, and label
      visibility.
- [x] Shape category, tick, data, and legend labels through `FontManager`.
- [x] Apply supported number formats and deterministic General fallback.
- [x] Project preserved individual point-label overrides without changing
      their serialization source or bytes.
- [x] Add focused scale, coordinate, shaping, and deterministic raster tests to
      the existing crate root.
- [x] Add alias, malformed-value, exact-order, raw-preservation, and
      point-override rendering tests.
- [x] Update exactly HLD 09.
- [x] Run focused checks, routed checks, microscope, and worker preparation.

## Open questions

None. F-125 reserves the plot margins, the existing font manager supplies the
only current shaping implementation, and unsupported legend placement details
remain preserved.
