# F-126, Chart rendering: axes, gridlines and labels

**Status**: approved
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
text. Honor deleted axes, axis position, orientation, explicit bounds,
gridline presence, major tick mark, and tick-label visibility. Format numeric
ticks and data labels through the existing `NumberFormat::format_value`
subset, falling back to deterministic General formatting when no format is
declared.

Shape category labels, numeric tick labels, requested series or point data
labels, and legend series names through the passed `FontManager`. Use a fixed
default chart font family and point size when no supported text property
overrides it. Position labels in the reserved F-125 margins and clip geometry
to the plot rectangle. Treat the current opaque legend shell as visible when
present, lay out one swatch and shaped series name per row, and keep unsupported
legend placement details preserved but behaviorally defaulted.

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

No parser, serialiser, crate graph, published API, binding, feature, new file,
external oracle, or unit-conversion rider applies. All renderer coordinates are
already points.

## Hash harness

Expected unchanged. Native chart annotation is not reached by the Word sample
generator or renderer. All 28 hashes must match.

## Implementation checklist

- [ ] Add deterministic nice-number scale and tick selection.
- [ ] Render major gridlines, axis lines, and tick marks in stable order.
- [ ] Honor axis deletion, position, orientation, explicit bounds, and label
      visibility.
- [ ] Shape category, tick, data, and legend labels through `FontManager`.
- [ ] Apply supported number formats and deterministic General fallback.
- [ ] Add focused scale, coordinate, shaping, and deterministic raster tests to
      the existing crate root.
- [ ] Update exactly HLD 09.
- [ ] Run focused checks, routed checks, microscope, and worker preparation.

## Open questions

None. F-125 reserves the plot margins, the existing font manager supplies the
only current shaping implementation, and unsupported legend placement details
remain preserved.
