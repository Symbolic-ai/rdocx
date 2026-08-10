# F-127, Chart colour resolution

**Status**: approved
**Sprint**: S32
**Size**: M
**Depends on**: F-125, F-055

## Problem

`rpptx-chart` currently assigns every series from a fixed six-colour placeholder
palette in `crates/rpptx-chart/src/lib.rs:3320`. The labelled renderer at
`crates/rpptx-chart/src/lib.rs:152` has no theme or colour-map input even though
each typed series already exposes direct `c:spPr` at
`crates/rpptx-chart/src/lib.rs:3577`. Charts therefore ignore direct series
styling and cannot follow the effective presentation theme.

The final chart palette must use the existing spec-correct DrawingML colour
pipeline. It must not alter the deliberately naive Word-only tint and shade
function or create a second colour implementation.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Colour, the part everyone gets wrong" and
  "Do not touch the Word path".
- `docs/hld/09-charts-spec.md`, "The ChartML model" and "Rendering".
- `docs/hld/14-development-backlog.md`, "F-127, Chart colour resolution".

## Approach

Change the two chart geometry entry points to receive the effective
`CT_OfficeStyleSheet` and `ColorMap` alongside their existing inputs. Build the
theme lookup from the concrete colour scheme and resolve all final colours with
`oxml_drawing::color::resolve_color`.

For each typed series, select the first supported direct solid colour from its
`c:spPr` fill or line fill. A direct `a:noFill` remains transparent. A present
unsupported direct paint returns a contextual chart projection error rather
than silently changing precedence. When no direct colour is present, cycle
through accent1 to accent6 by series order and resolve the selected scheme
colour through the effective colour map and ordered transform stack.

Pass the resulting series colours through every bar, line, wedge, area, marker,
radar, data-label, and legend-swatch path that currently calls the placeholder
selector. Keep plot geometry, clipping, alpha policy, and z-order unchanged.
Delete the placeholder palette once no production path uses it.

## Rejected alternatives

- Copy colour-transform math into `rpptx-chart`. The established
  `oxml-drawing` resolver already owns exact DrawingML semantics.
- Correct `rdocx_oxml::theme::apply_tint_shade`. That path is deliberately held
  for output stability and is not used by presentation charts.
- Add a colour-provider trait. There is one concrete DrawingML theme and colour
  map implementation today.
- Keep a no-context public renderer that silently uses the Office default
  theme. Callers already own the effective theme, and a hidden default would
  render custom templates incorrectly.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `unstyled_four_series_use_accent_one_through_four` | Four unstyled series resolve to the effective theme's accent1 through accent4 values |
| unit | `direct_series_solid_colour_overrides_theme_accent` | Direct `c:spPr` solid fill or line paint wins over the matching accent slot |
| unit | `series_accent_cycle_repeats_after_six` | Series seven resumes at accent1 without changing plot order |
| unit | `series_colours_honor_colour_map_and_transform_order` | Scheme mapping, tint, shade, luminance, and alpha resolve through the existing exact pipeline |
| regression | `chart_colours_do_not_use_word_tint_shade` | Presentation chart output is independent of the deliberately naive Word helper |
| negative | `unsupported_direct_series_paint_is_contextual` | A present unsupported direct paint returns a chart-specific error rather than falling back silently |
| golden | `resolved_chart_palette_raster_is_deterministic` | Repeated deterministic-font renders have identical pixels with the final palette |

The test gate is: an unstyled four-series chart uses accent1 through accent4.

## HLD impact

- `docs/hld/09-charts-spec.md`

Replace the temporary palette description with the concrete direct-style and
theme-accent resolution contract and the final renderer inputs.

## Risk routing

- Theme colour, tint, shade, colour mapping. Read HLD 05. Reuse the
  spec-correct `oxml-drawing` resolver, leave the Word helper unchanged, run
  the exact theme-colour tests, and run the focused chart colour suite.
- Layout and text shaping. Read HLD 08. Run the palette raster only with
  deterministic fonts and treat any baseline change as deliberate evidence.

No parser, serialiser, crate graph, published API, binding, feature, new file,
external oracle, or unit-conversion rider applies.

## Hash harness

Expected unchanged. The Word sample renderer does not invoke presentation
chart geometry. All 28 hashes must match.

## Implementation checklist

- [ ] Add effective theme and colour-map inputs to chart rendering.
- [ ] Resolve direct solid series styling with exact precedence.
- [ ] Resolve the six-slot accent cycle through the established colour map and
      transform stack.
- [ ] Replace every placeholder series and legend colour call site.
- [ ] Add focused exact-colour, negative, cycle, and deterministic raster tests
      to the existing crate root.
- [ ] Update exactly HLD 09.
- [ ] Run focused checks, routed checks, microscope, and worker preparation.

## Open questions

None. HLD 05 owns colour semantics and HLD 09 fixes direct `c:spPr` precedence
over the effective theme accent cycle.
