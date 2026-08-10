# F-123, Data labels and number formats

**Status**: approved
**Sprint**: S30
**Size**: M
**Depends on**: F-119

## Problem

F-119 deliberately leaves `c:dLbls` raw in the series boundary at
`crates/rpptx-chart/src/lib.rs:721`. Numeric caches carry a source format code,
but the model has no chart number-format value, label visibility flags,
position, separator, or deterministic text projection. A future renderer
therefore cannot determine whether a cached value should appear as `0.25`,
`25%`, or another document-selected form.

F-120 also introduces the same `formatCode` and `sourceLinked` pair for axes.
The sprint needs one concrete number-format value used by both current
locations, without growing into a general Excel formatting engine.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, XML preservation and prefix contract.
- `docs/hld/06-presentationml-model.md`, schema-ordered typed XML behavior.
- `docs/hld/09-charts-spec.md`, "The ChartML model", plot data labels, and
  number formats.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and external render
  evidence.
- `docs/hld/13-risks-and-open-questions.md`, "R5, schema child ordering" and
  chart scope containment.
- `docs/hld/14-development-backlog.md`, "F-123, Data labels and number
  formats".

## Approach

Extend the existing `rpptx-chart` crate root after F-120 is integrated. Add no
crate, file, module, feature, dependency, trait, generic parameter, or
forwarding wrapper.

Reuse F-120's concrete number-format value and add one label collection:

```rust
pub struct NumberFormat {
    pub format_code: String,
    pub source_linked: bool,
}

pub enum DataLabelPosition {
    BestFit, Bottom, Center, InsideBase, InsideEnd, Left, OutsideEnd, Right, Top,
}

pub struct CT_DLbls {
    pub number_format: Option<NumberFormat>,
    pub position: Option<DataLabelPosition>,
    pub separator: Option<String>,
    pub show_legend_key: bool,
    pub show_value: bool,
    pub show_category_name: bool,
    pub show_series_name: bool,
    pub show_percent: bool,
    pub show_bubble_size: bool,
}
```

Add `Series::data_labels: Option<CT_DLbls>`. F-121 and F-122 reuse the same
type for plot-level label defaults. Individual `c:dLbl` point overrides,
leader lines, shape properties, text properties, extensions, and producer
payloads remain byte-preserved in their ordered slots until a rendering story
needs their behavior.

Readers accept aliased ChartML prefixes and model `c:numFmt`, `c:dLblPos`, the
six show flags, and `c:separator`. Writers use fixed prefixes and the ChartML
sequence. Reject duplicate modelled children, unknown positions, malformed
booleans, empty or XML-invalid format codes, XML-invalid separators, and
nonfinite values passed to formatting.

Add `NumberFormat::format_value(f64) -> Result<String>` for the current renderer
consumer. Support `General`, ordinary zero-placeholder decimal precision, and
percentage forms such as `0%`, `0.0%`, and `0.00%`. Percentage forms multiply
the cached value by 100 before applying declared precision. Preserve other
valid producer codes for round-trip, but return a contextual unsupported-value
error rather than guessing their display text. This is deliberately not an
Excel format-language implementation.

The focused gate builds a chart candidate from a corpus deck, replaces only
the chart part with serialized typed output carrying value `0.25`, `showVal`,
and format `0%`, then renders it through pinned LibreOffice 26.2.5.2. Pinned
Poppler 26.01.0 text extraction must contain `25%`. Bind the candidate and
render evidence to one SHA. This proves correct viewer text without adding the
native label geometry owned by F-126.

## Rejected alternatives

- Reuse `NumericData::format_code` as label state. Cache source formatting and
  chart label formatting are distinct XML locations with independent
  `sourceLinked` behavior.
- Parse the complete Excel number-format language. F-123 needs the common
  numeric and percentage forms, not a spreadsheet formatting subsystem.
- Type individual point overrides now. The gate needs collection defaults, and
  unsupported overrides remain safely preserved.
- Add native glyph placement. F-126 owns axis, gridline, and label rendering.
- Compare serialized bytes with an external writer. Structural comparison and
  viewer output are the relevant contracts.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `data_labels_write_fixed_prefixes_in_schema_order` | Number format, position, show flags, separator, and preserved children write in the required order |
| unit, gate | `percentage_formatted_label_renders_with_correct_text` | A cached value of `0.25` with `0%` projects and externally renders as `25%` |
| unit | `common_number_formats_project_cached_values_deterministically` | General, fixed decimal, and percentage precision produce exact text without locale dependence |
| negative | `malformed_data_labels_and_number_formats_return_errors_without_panicking` | Duplicate fields, invalid booleans, positions, XML text, unsupported projections, and nonfinite values return contextual errors |
| preservation | `data_labels_preserve_point_overrides_and_extensions_byte_for_byte` | Individual labels, leader lines, shape and text payloads, attributes, comments, and whitespace retain bytes and positions |
| round-trip | `every_corpus_data_label_collection_round_trips_structurally` | Every corpus label collection and axis number format reparses equally with nonzero coverage where present |

The test gate is: a percentage-formatted label renders with the correct text.

## HLD impact

- `docs/hld/09-charts-spec.md`

Document the implemented number-format and data-label API, supported text
projection subset, validation and preservation boundary, corpus coverage, and
pinned viewer evidence.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias-prefix,
  fixed-prefix, exact schema-order, malformed-value, byte-preservation, and
  corpus structural round-trip checks.
- External oracle comparison. Follow differential-testing guidance. Pin
  LibreOffice 26.2.5.2 and Poppler 26.01.0, bind the candidate and extracted
  text to one SHA, and keep both tools outside normal crate dependencies.

No crate graph, published API, binding, feature, new file, version, release,
layout, unit-conversion, or baseline rider applies.

## Hash harness

Expected unchanged. The unpublished label model and its external viewer test
do not enter Word sample generation or rendering. All 28 hashes must match.

## Implementation checklist

- [ ] Reuse the F-120 number-format value for axes and labels.
- [ ] Add the typed data-label collection and series attachment.
- [ ] Add deterministic General, decimal, and percentage text projection.
- [ ] Preserve unsupported label payloads in ordered raw slots.
- [ ] Add negative, ordering, preservation, and corpus tests.
- [ ] Produce SHA-bound LibreOffice and Poppler percentage-label evidence.
- [ ] Update exactly HLD 09.
- [ ] Run focused parser, corpus, oracle, microscope, and worker preparation
      checks.

## Open questions

None. F-126 owns native glyph placement. F-123 owns the typed label state and
deterministic display text that renderer consumes.
