# F-122, Pie, doughnut, area, scatter and radar plots

**Status**: completed
**Sprint**: S30
**Size**: L
**Depends on**: F-121

## Problem

F-121 establishes the owned plot-area boundary but types only bar and line
containers. Pie, doughnut, area, scatter, and radar plots remain opaque, so the
v1 chart surface cannot parse, edit, validate, or author five of its seven
required families.

The families do not all share the same XML shape. Pie and doughnut plots have
no axes, area and radar plots use category plus value series, and scatter plots
write `c:xVal` plus `c:yVal` instead of `c:cat` plus `c:val`. The design must
represent those real differences without duplicating the F-119 cache model or
claiming unsupported 3-D, stock, surface, bubble, or `ofPie` behavior.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Charts" and the unsupported 3-D
  boundary.
- `docs/hld/04-opc-and-packaging.md`, XML preservation and prefix contract.
- `docs/hld/06-presentationml-model.md`, schema-ordered typed XML behavior.
- `docs/hld/09-charts-spec.md`, "The ChartML model" and the typed `Plot`
  boundary.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The deck corpus", and
  external render evidence.
- `docs/hld/13-risks-and-open-questions.md`, "R5, schema child ordering" and
  "R7, scope".
- `docs/hld/14-development-backlog.md`, "F-122, Pie, doughnut, area, scatter
  and radar plots" and "F-125, Chart rendering: geometry".

## Approach

Extend the existing F-121 `Plot` enum in `crates/rpptx-chart/src/lib.rs`. Add
no crate, file, module, feature, dependency, trait, generic parameter, or
forwarding wrapper.

Add the remaining v1 variants:

```rust
pub enum Plot {
    // F-121 Bar and Line variants remain.
    Pie {
        first_slice_angle: u16,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
    },
    Doughnut {
        first_slice_angle: u16,
        hole_size: u8,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
    },
    Area {
        grouping: Grouping,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Scatter {
        style: ScatterStyle,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Radar {
        style: RadarStyle,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
}
```

Validate first-slice angle from 0 through 360, doughnut hole size from 10
through 90, required grouping and style enums, nonempty series, and exactly two
resolving axis references for area, scatter, and radar. Pie and doughnut reject
axis references. Preserve optional varying-colour state and producer lexical
markup when present.

Keep one public `Series` value. In a scatter plot, a plot-specific writer maps
numeric `Series::categories` to `c:xVal` and `Series::values` to `c:yVal`.
Scatter parsing maps those wrappers back to the same current fields while
private markup remembers the original wrapper names. Reject string x values,
missing x or y caches, and simultaneous category/value plus x/y choices. This
avoids parallel public series types and keeps one cache implementation.

Parse each supported two-dimensional root under an aliased ChartML prefix and
write the fixed root and exact schema sequence. Preserve explosion points,
markers, drop lines, extension lists, unknown attributes, comments, whitespace,
and unsupported family-specific content in ordered raw slots. 3-D, `ofPie`,
stock, surface, and bubble plots remain opaque choices. Combination plot areas
remain preserved under the F-121 rule.

The corpus currently supplies one two-dimensional pie plot and no non-vacuous
doughnut, area, scatter, or radar plot. Inline fixtures therefore cover every
variant and scatter wrapper branch. For the backlog render gate, insert each
typed candidate into the same SHA-bound representative chart deck used by
F-121, render through pinned LibreOffice 26.2.5.2 and Poppler 26.01.0, and
require a successful page plus a stated nonblank-pixel threshold inside the
known chart rectangle. Native path generation remains F-125 work.

## Rejected alternatives

- Add a second scatter-series type. The current numeric category and value
  caches already represent x and y data, while private wrapper markup preserves
  the XML distinction.
- Treat every plot as bar-like. Axis-free plots and scatter x/y wrappers have
  different required sequences and validation.
- Type bubble, stock, surface, 3-D, or `ofPie` plots. They are outside the v1
  authored surface and remain safely preserved.
- Add native wedge, area, marker, or radar geometry. F-125 owns rendering
  geometry after the full typed plot boundary exists.
- Rely on corpus coverage alone. Four required variants are absent from the
  pinned set and need explicit non-vacuous fixtures.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip, gate | `remaining_v1_plots_round_trip_and_render` | Pie, doughnut, area, scatter, and radar plots serialize, reparse equally, retain caches, and pass pinned viewer rendering |
| unit | `remaining_plot_families_write_fixed_prefixes_in_schema_order` | Every variant writes its properties, series, labels, and axis references in the required order |
| unit | `scatter_series_map_numeric_categories_and_values_to_x_and_y` | Numeric categories become `xVal`, values become `yVal`, and both caches remain the one public series source |
| negative | `malformed_remaining_plots_return_errors_without_panicking` | Bad angles, hole sizes, styles, groupings, series, scatter wrappers, duplicates, and axis references return errors |
| preservation | `unsupported_plot_families_and_children_remain_byte_preserved` | 3-D, `ofPie`, stock, surface, bubble, extensions, attributes, comments, and whitespace retain bytes and order |
| round-trip | `every_supported_corpus_plot_round_trips_structurally` | Corpus pie plus all supported bar and line plots still reparse equally, while inline fixtures keep absent variants non-vacuous |

The test gate is: pie, doughnut, area, scatter, and radar plots each round-trip
and render.

## HLD impact

- `docs/hld/09-charts-spec.md`

Document all seven completed plot variants, scatter x/y mapping, validation,
axis ownership, unsupported-family boundary, corpus gaps, inline coverage, and
SHA-bound viewer evidence.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias-prefix,
  fixed-prefix, exact schema-order, malformed-value, scatter-wrapper,
  byte-preservation, public mutation, and corpus structural round-trip checks.
- External oracle comparison. Follow differential-testing guidance. Pin
  LibreOffice 26.2.5.2 and Poppler 26.01.0, bind every candidate render to its
  SHA, state the chart-rectangle pixel metric and threshold, and keep oracle
  tools outside normal dependencies.

No crate graph, published API, binding, feature, new file, version, release,
native layout, unit-conversion, or baseline rider applies.

## Hash harness

Expected unchanged. The unpublished plot model and external viewer evidence do
not enter Word sample generation or rendering. All 28 hashes must match.

## Implementation checklist

- [x] Extend the plot enum with pie, doughnut, area, scatter, and radar.
- [x] Add plot-specific validation and axis ownership rules.
- [x] Map scatter x/y wrappers onto the existing numeric series data.
- [x] Parse and write all five families in exact schema order.
- [x] Preserve unsupported plot families and children verbatim.
- [x] Add negative, ordering, preservation, scatter, corpus, and inline tests.
- [x] Produce SHA-bound LibreOffice and Poppler render evidence.
- [x] Update exactly HLD 09.
- [x] Run focused parser, corpus, oracle, microscope, and worker preparation
      checks.

## Open questions

None. The absent corpus families receive explicit inline and pinned viewer
fixtures, while F-125 retains native geometry ownership.
