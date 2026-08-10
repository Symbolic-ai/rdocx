# Current Sprint, S29

**Milestone**: M12 Charts.

**Goal**: Establish the chart data layer with a deliberately minimal embedded
workbook writer, the core ChartML tree, and series references whose formulae
and caches remain consistent. The result must provide the data foundation that
later chart authoring and rendering stories can use without expanding
`oxml-sml` into a general spreadsheet library.

## Spec references

- `docs/hld/00-vision.md`, for the decision that chart support spans ChartML
  and a minimal SpreadsheetML writer.
- `docs/hld/01-glossary.md`, for the `oxml-sml` and `rpptx-chart` vocabulary and
  ownership boundary.
- `docs/hld/02-scope-and-non-goals.md`, for the v1 chart surface and the
  permanent limit that `oxml-sml` is not a spreadsheet library.
- `docs/hld/03-architecture.md`, for crate ownership and the acyclic dependency
  direction between format-neutral and PresentationML code.
- `docs/hld/04-opc-and-packaging.md`, for chart and embedded-workbook part
  locations and collision-safe numeric suffix allocation.
- `docs/hld/09-charts-spec.md`, for the workbook package, core ChartML types,
  series model, formula references, and mandatory caches.
- `docs/hld/12-testing-strategy.md`, for corpus round-trip, differential, and
  deterministic verification requirements.
- `docs/hld/13-risks-and-open-questions.md`, for schema child ordering and the
  deliberate containment of chart scope.
- `docs/hld/14-development-backlog.md`, for F-117 through F-119 dependencies,
  sizes, focused test gates, and the M12 milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for the reserved unpublished
  `oxml-sml` and `rpptx-chart` packages and their publication ordering.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-117 | oxml-sml workbook writer | L | done | - |
| F-118 | ChartML core types | L | done | - |
| F-119 | Series and data references | L | done | - |

## Sequencing note

F-117 and F-118 are independent and may proceed in parallel. F-119 follows
F-118 because series, category, value, formula-reference, and cache types attach
to the core ChartML tree. The workbook and ChartML data paths remain separate
in this sprint and converge in the later `add_chart` story.

## Definition of done for this sprint

- `oxml-sml` writes a complete one-worksheet `.xlsx` with numeric and string
  cells, shared strings when needed, number formats, and defined ranges. Excel
  and LibreOffice Calc open the result cleanly.
- The core ChartML space, chart, plot-area, title, and legend types parse and
  write in schema order while preserving unmodelled XML. A corpus chart part
  round-trips structurally.
- Series, category, value, string-reference, numeric-reference, and cache types
  preserve one source of truth. A written chart carries a formula reference and
  a consistent cache so viewers can render it without opening the workbook.
- The full workspace gate passes, all 28 deterministic hashes remain unchanged,
  the chart development crates remain unpublished at version 0.0.0, and no
  crate is published.
