# F-X077, Portable authored Word charts

**Status**: approved
**Sprint**: S65
**Size**: L
**Depends on**: F-158

## Problem

The native chart authoring path writes an editable chart part and embedded
workbook, but it relies on viewer defaults for several visible semantics. A
new document has no theme relationship, axes omit an explicit visible state,
the chart number format does not reach the value axis, one-series category
charts have no explicit point colors, and pie or doughnut charts omit their
category legend because they contain only one series.

Microsoft Word supplies enough defaults to mask parts of this gap. Pages does
not. It can show a blank or incomplete chart, and a literal value such as 12.5
can lose its intended percentage display. Flattening the chart to an image
would avoid viewer differences but destroy the editable data contract.

## Spec reference

- `docs/hld/03-architecture.md`, for facade-owned authoring and transactional
  publication through existing crate boundaries.
- `docs/hld/04-opc-and-packaging.md`, for complete relationship-owned chart,
  workbook, theme, and content-type assembly.
- `docs/hld/05-drawingml-model.md`, for default Office themes and typed color
  handling.
- `docs/hld/09-charts-spec.md`, for chart data, plots, axes, legends, formulas,
  caches, schema order, and raw sibling preservation.
- `docs/hld/10-bindings-spec.md`, for the native Word facade and intentional
  pre-1.0 source compatibility.
- `docs/hld/12-testing-strategy.md`, for differential source fixtures and
  pinned external-oracle evidence.
- `docs/hld/14-development-backlog.md`, "F-X077, Portable authored Word
  charts".

## Approach

Extend `ChartData` with three cohesive authoring inputs: an optional category
axis title, an optional value axis title, and a typed RGB palette. Re-export
the existing DrawingML RGB type from the `rdocx` facade. This intentionally
changes complete `ChartData` struct literals before 1.0. Keep an empty palette
as the existing theme-derived behavior, and cycle a non-empty palette
deterministically when there are more series or categories than colors.

Teach the existing ChartML model the minimum typed behavior needed by that
surface. Author plain-text axis titles, explicit axis deletion state, series
shape properties, and indexed data-point shape properties in schema order.
Apply palette colors by series for every supported plot. For a one-series bar
and for pie or doughnut plots, author indexed category-point colors so viewers
do not need to infer varying colors. Keep pie and doughnut legends even with a
single series. Put the supplied number format on the value axis as well as the
workbook and caches.

When the Word facade authors a chart into a document with no effective theme,
stage the existing Office default theme part, its document relationship, and
its content type in the same cloned package transaction as the chart. Reuse an
existing theme unchanged. Choose a collision-safe part name rather than
overwriting an unrelated package entry.

Do not add a second chart representation, post-serialization XML replacement,
viewer-specific branch, new dependency, crate, module, or production file.
The Symbolic caller remains responsible for choosing its palette and for
mapping its visual pie contract to a doughnut chart.

## Rejected alternatives

- Render charts to images. This loses editable workbook data and does not meet
  the product contract.
- Put Symbolic brand colors in rdocx defaults. Brand policy belongs to the
  caller, while rdocx should expose a generic typed palette.
- Depend on `varyColors` alone. Pages does not reliably assign distinct colors
  to one-series bar categories, so the required point styles must be explicit.
- Always replace the document theme. Existing themes are caller-owned package
  semantics and must remain authoritative.
- Inject raw ChartML strings after serialization. That bypasses schema order,
  preservation, and the typed round-trip model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | portable authored line chart | A source-built line chart gains a default theme only when needed, explicit visible axes, a formatted value axis, typed titles, series color, and unchanged editable data after reopen. |
| regression | category point colors | One-series bar, pie, and doughnut charts emit deterministic indexed point colors and preserve them after parse and write. |
| regression | category legends | Pie and doughnut charts keep a legend with one series, while cartesian one-series legend behavior remains unchanged. |
| package | theme ownership and collision | Chart authoring reuses an existing theme, otherwise adds one collision-safe theme part, relationship, and content type transactionally. |
| round-trip | typed ChartML additions | Titles, explicit false deletion state, series styles, data-point styles, and unrelated raw siblings survive parse, mutate, save, and reopen in schema order. |
| compatibility | existing chart kinds | Bar, line, area, scatter, radar, pie, and doughnut authoring still saves and reopens with exact formulas, caches, categories, series, and values. |
| external oracle | Word and Pages | Word 16.104 opens without repair. A pinned Pages build renders axes, palette colors, category legend, and literal percentages, then exports semantically exact editable chart data. |
| integration | changed and consuming crates | Focused crate tests, the complete workspace, hash harness, and public package dry-runs pass. |

The **test gate is differential**. Source-built line, bar, pie, and doughnut
documents save and reopen with exact editable workbook semantics. Word 16.104
opens them without repair. A pinned Pages build renders the authored semantics
and exports a DOCX whose chart and workbook data remain exact.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/09-charts-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Parser or serializer changes**. Preserve unmodelled XML, remain prefix
  tolerant, and prove every new child at its schema-defined position.
- **Theme color changes**. Reuse `CT_OfficeStyleSheet::office_default` and the
  existing DrawingML color model. Do not alter legacy Word tint math.
- **Public published API changes**. Record the intentional pre-1.0 source
  break, run package dry-runs, and report packed sizes.
- **External oracle comparison**. Pin exact Word and Pages versions, construct
  source documents from the public API, and record viewer behavior without
  adding oracle output or binaries to the repository.

## Hash harness

Expected unchanged across all existing entries. The standard sample set does
not author charts. Any delta blocks integration until classified and approved.

## Implementation checklist

- [ ] Add failing source-built tests for theme ownership, axis semantics,
  value formatting, titles, palette styles, point colors, and legends.
- [ ] Extend the typed ChartML model and writer in schema order.
- [ ] Extend the cohesive `ChartData` authoring input and facade export.
- [ ] Stage a default Office theme only when an authored chart needs one.
- [ ] Prove exact chart and workbook semantics after save and reopen.
- [ ] Record pinned Word and Pages interoperability evidence.
- [ ] Run focused crates, full verification, hash harness, and package dry-runs.
- [ ] Update exactly the six listed HLD files.

## Open questions

None. The user approved extracting the generic chart portability work from the
Symbolic integration into rdocx, with Symbolic retaining its brand palette and
chart-kind choices.
