# F-121, Bar and line plots

**Status**: approved
**Sprint**: S30
**Size**: M
**Depends on**: F-119, F-120

## Problem

`CT_PlotArea` still stores every plot container as opaque XML at
`crates/rpptx-chart/src/lib.rs:1873`. Its F-119 `series()` method projects
series for validation but cannot edit or author a bar or line plot, retain a
plot's axis references as typed values, or validate plot-specific ranges.

The pinned corpus contains twelve two-dimensional bar plots and three line
plots, alongside unsupported 3-D and combination content. Real bar and line
plots reference the plot-area-owned axes using the producer-compatible signed
identifier domain established by F-120. The typed boundary must keep that
ownership clear while retaining unsupported plots verbatim.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, XML preservation and prefix contract.
- `docs/hld/06-presentationml-model.md`, schema-ordered typed XML behavior.
- `docs/hld/09-charts-spec.md`, "The ChartML model" and the typed `Plot`
  boundary.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The deck corpus", and
  external render evidence.
- `docs/hld/13-risks-and-open-questions.md`, "R5, schema child ordering" and
  chart scope containment.
- `docs/hld/14-development-backlog.md`, "F-121, Bar and line plots" and
  "F-125, Chart rendering: geometry".

## Approach

Extend the existing `rpptx-chart` crate root after F-120 and F-123 are
integrated. Add no crate, file, module, feature, dependency, trait, generic
parameter, or forwarding wrapper.

Add the first two variants of the HLD plot value:

```rust
pub enum Plot {
    Bar {
        direction: BarDirection,
        grouping: BarGrouping,
        gap_width: u16,
        overlap: i8,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Line {
        grouping: Grouping,
        marker: bool,
        smooth: bool,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
}
```

`CT_PlotArea` owns plot values and the axes introduced by F-120. Each plot owns
its series and exactly two references into the plot-area axis set. Axes are not
embedded in plots. Validate that both references resolve and that the axis set
retains F-120's reciprocal `crossAx` invariant.

Parse `c:barChart` and `c:lineChart` under any prefix bound to ChartML. Write
fixed prefixes and exact child order. Bar plots validate direction, grouping,
gap width from 0 through 500, overlap from -100 through 100, nonempty series,
and exactly two axis references. Line plots validate grouping, booleans,
nonempty series, and the same references. Preserve `varyColors`, drop lines,
high-low lines, up/down bars, series markers and points, extensions, unknown
attributes, comments, and whitespace in ordered schema slots.

Promote supported single-family plot areas from a read-only projection to
owned typed plot and axis collections with a constructor for F-124. A plot area
containing 3-D, stock, surface, `ofPie`, or multiple plot families remains an
opaque preserved choice. It must not be partially rewritten merely because
one child is recognized. Public edits that would combine an opaque choice and
typed plots return a duplicate-state error.

The backlog's render clause is an external-viewer acceptance gate, not native
geometry. F-125 explicitly owns bar, line, area, wedge, and marker path
generation. Build SHA-bound candidate copies of representative corpus bar and
line decks, replacing only their chart part with `CT_ChartSpace::to_xml()`
output. Render originals and candidates with pinned LibreOffice 26.2.5.2 and
Poppler 26.01.0. Require successful conversion and pixel-equivalent chart
pages. This proves the serialized plots remain renderable without stealing
F-125 scope.

## Rejected alternatives

- Embed axes in each plot. OOXML stores axis objects once in the plot area and
  plots reference them by id.
- Type 3-D and combination plots opportunistically. They are outside the v1
  authored surface and must remain lossless preserved content.
- Add native path generation here. F-125 owns rendering geometry and depends on
  the typed values from this story.
- Keep using only `CT_PlotArea::series()`. A read-only projection cannot support
  F-124 authoring or plot-specific validation.
- Create separate bar and line modules. Both current variants belong to one
  concrete plot boundary in the existing crate root.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip, gate | `bar_and_line_plots_round_trip_and_render` | One bar and one line plot serialize, reparse equally, retain caches and axis references, and pass pinned viewer rendering |
| unit | `bar_and_line_plots_write_fixed_prefixes_in_schema_order` | Plot properties, series, labels, gap, overlap, smooth, and axis ids use fixed prefixes and required order |
| negative | `malformed_bar_and_line_plots_return_errors_without_panicking` | Missing fields, unknown enums, bad ranges, invalid booleans, empty series, duplicate children, and invalid axis references return errors |
| preservation | `unsupported_and_combo_plots_remain_byte_preserved` | 3-D, combination, producer extensions, attributes, comments, and whitespace retain bytes and sibling order |
| mutation | `public_plot_edits_preserve_axes_and_unselected_payloads` | Editing one supported plot changes only its typed fields and retains plot-area axes and raw content |
| round-trip | `every_corpus_bar_and_line_plot_round_trips_structurally` | All twelve corpus bar plots and three line plots reparse equally with nonzero exact coverage |

The test gate is: bar and line plots each round-trip and render.

## HLD impact

- `docs/hld/09-charts-spec.md`

Document the concrete bar and line variants, line grouping, plot-axis
references, plot-area ownership, validation, unsupported combination boundary,
corpus counts, and SHA-bound viewer evidence.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias-prefix,
  fixed-prefix, exact schema-order, malformed-value, byte-preservation, public
  mutation, and corpus structural round-trip checks.
- External oracle comparison. Follow differential-testing guidance. Pin
  LibreOffice 26.2.5.2 and Poppler 26.01.0, bind original and candidate renders
  to their SHAs, use a stated pixel metric and threshold, and keep oracle tools
  outside normal dependencies.

No crate graph, published API, binding, feature, new file, version, release,
native layout, unit-conversion, or baseline rider applies.

## Hash harness

Expected unchanged. The unpublished plot model and external viewer evidence do
not enter Word sample generation or rendering. All 28 hashes must match.

## Implementation checklist

- [ ] Add bar and line plot enums, values, constructors, and validation.
- [ ] Promote supported single-family plot areas to owned plots and axes.
- [ ] Parse and write bar and line children in exact schema order.
- [ ] Preserve unsupported, 3-D, and combination plot areas verbatim.
- [ ] Add negative, mutation, ordering, preservation, and corpus tests.
- [ ] Produce SHA-bound LibreOffice and Poppler render evidence.
- [ ] Update exactly HLD 09.
- [ ] Run focused parser, corpus, oracle, microscope, and worker preparation
      checks.

## Open questions

None. The external viewer gate satisfies the backlog wording, while F-125
retains exclusive ownership of native rendering geometry.
