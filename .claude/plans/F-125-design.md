# F-125, Chart rendering: geometry

**Status**: approved
**Sprint**: S31
**Size**: L
**Depends on**: F-121, F-093

## Problem

`rpptx-chart` parses and writes the seven supported two-dimensional plot
families, but it produces no backend-neutral drawing elements. As a result,
`rpptx-layout` currently marks every chart graphic frame unsupported and
`rpptx-render` can only draw its fallback rectangle. The typed caches already
contain the values needed for native geometry, but no code maps those values
into chart-local paths.

This story owns the plot geometry only. Axes, gridlines, text labels, legends,
theme colour resolution, preserved-chart fallbacks, and pipeline wiring remain
separate backlog work.

## Spec reference

- `docs/hld/03-architecture.md`, PresentationML crate ownership and dependency
  direction.
- `docs/hld/08-rendering-spec.md`, backend-neutral `PathElement`, `Group`, and
  point-coordinate contracts.
- `docs/hld/09-charts-spec.md`, "The ChartML model", "Rendering", and "What is
  not in v1".
- `docs/hld/12-testing-strategy.md`, deterministic render tests and raster
  comparison.
- `docs/hld/14-development-backlog.md`, "F-125, Chart rendering: geometry".

## Approach

Extend the existing `rpptx-chart` crate root and its existing tests. Add
`oxml-layout` as a normal dependency. Add one concrete geometry entry point:

```rust
pub struct ChartGeometry {
    pub plot_bounds: Rect,
    pub elements: Vec<PositionedElement>,
}

pub fn render_geometry(chart: &CT_Chart, bounds: Rect) -> Result<ChartGeometry>;
```

`bounds` is already in typographic points. The function reserves stable plot
margins that F-126 can populate, computes category and value extents from the
typed caches, and emits chart-local `PathElement` values inside one group.
Reject nonfinite or nonpositive bounds and unsupported opaque or combination
plots with contextual errors.

For bar plots, compute clustered, stacked, and percentage-stacked rectangles
from category slots, gap width, overlap, direction, and a zero-inclusive value
domain. For line and scatter plots, emit one open polyline per series and
marker paths at finite data points. For pie and doughnut plots, convert
positive values to closed cubic wedge paths, with the doughnut inner radius and
first-slice angle applied. For area plots, close each series path to its
baseline. For radar plots, map category angles and values to closed polygonal
paths with markers.

Use deterministic placeholder solid paints indexed by series so geometry can
be raster-tested now. F-127 replaces only the colour selection with chart and
theme resolution. Preserve z-order by plot order, then series order, with
fills before strokes and markers after their owning series path.

## Rejected alternatives

- Add chart primitives to a PDF or PNG backend. Existing path and group output
  already expresses every required geometry and keeps backends unchanged.
- Wire chart package resolution into `rpptx-layout` now. F-129 owns extraction
  of chart relationships into the render input, while this story has a direct
  typed geometry gate.
- Introduce a renderer trait or generic coordinate abstraction. There is one
  current output contract, `oxml-layout`, and no second implementation.
- Resolve final theme colours here. F-127 owns the colour pipeline and can
  replace the temporary series palette without changing geometry.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden, gate | `bar_chart_rasterises_at_computed_positions` | A known clustered bar chart emits exact bar bounds and the tiny-skia raster contains filled pixels in those bounds |
| unit | `bar_geometry_handles_direction_grouping_gap_and_overlap` | Column and horizontal modes plus clustered, stacked, and percentage-stacked calculations match hand-computed coordinates |
| unit | `line_scatter_and_radar_emit_paths_and_markers` | Cached points map to ordered polyline, polygon, and marker commands inside plot bounds |
| unit | `pie_doughnut_and_area_emit_closed_paths` | Wedges, rings, and areas close at exact baselines, angles, and radii |
| negative | `geometry_rejects_invalid_bounds_and_opaque_plots` | Nonfinite bounds, empty renderable data, and unsupported plot choices return contextual errors without panic |
| regression | `geometry_is_backend_neutral_and_deterministic` | Repeated rendering produces equal paths and identical raster bytes in deterministic mode |

The test gate is: a bar chart rasterises with bars at computed positions.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/09-charts-spec.md`

Record the concrete `rpptx-chart` to `oxml-layout` edge, geometry entry point,
plot-area margins, supported family calculations, temporary colour boundary,
and division of work from F-126 through F-129.

## Risk routing

- Layout, pagination, line breaking, text shaping. Read HLD 08. Run every
  raster baseline with deterministic fonts and record any baseline change
  deliberately.
- Crate dependency graph, a new `use` across families. Read HLD 03. Run the
  architecture dependency check and confirm the new edge points from
  `rpptx-chart` to format-neutral `oxml-layout`.

No parser, serialiser, published API, binding, feature, new file, external
oracle, or unit-conversion rider applies. Input and output coordinates are
already points.

## Hash harness

Expected unchanged. The new chart-only geometry entry point is not reached by
the Word sample generator or renderer. All 28 hashes must match.

## Implementation checklist

- [ ] Add the `oxml-layout` dependency and concrete geometry result.
- [ ] Validate bounds, typed plots, and cached render data.
- [ ] Implement bar and line geometry with exact category and value mapping.
- [ ] Implement pie, doughnut, area, scatter, radar, and marker paths.
- [ ] Preserve deterministic order and a replaceable temporary series palette.
- [ ] Add focused coordinate, negative, determinism, and tiny-skia raster tests
      to the existing crate root.
- [ ] Update exactly HLD 03 and HLD 09.
- [ ] Run focused checks, routed checks, microscope, and worker preparation.

## Open questions

None. The existing backend-neutral path model covers every required geometry,
and later stories retain axes, final colours, fallback selection, and pipeline
wiring.
