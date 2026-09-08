# Current Sprint, S65

**Milestone**: M22 Word depth with a cross-cutting chart correction.

**Goal**: author, convert, lay out, and render modern Word equations. In
parallel, make authored Word charts portable across Word and Pages while
retaining their editable workbook data.

## Spec references

- `docs/hld/03-architecture.md`, for facade-owned authoring and transactional
  publication through existing model and package layers.
- `docs/hld/04-opc-and-packaging.md`, for relationship-safe chart, workbook,
  theme, and content-type assembly.
- `docs/hld/05-drawingml-model.md`, for Office theme defaults and typed color
  semantics.
- `docs/hld/09-charts-spec.md`, for editable chart data, schema-ordered
  ChartML, formulas, caches, plots, axes, and legends.
- `docs/hld/10-bindings-spec.md`, for the native Word facade and explicit
  pre-1.0 source-compatibility decisions.
- `docs/hld/12-testing-strategy.md`, for source-built differential fixtures,
  external oracle discipline, and unchanged deterministic hash gates.
- `docs/hld/14-development-backlog.md`, for the F-228 through F-230 and
  F-X077 acceptance gates and dependency order.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X077 | Portable authored Word charts | L | pending | - |
| F-228 | OfficeMath model and authoring | L | pending | - |
| F-229 | OfficeMath layout and PDF rendering | M | pending | - |
| F-230 | MathML and LaTeX conversion | M | pending | - |

## Sequencing note

Rows are listed in review order, not F-ID order. F-X077 is a late
cross-cutting correction requested after editable chart export exposed missing
theme, axis, number-format, point-color, and legend semantics in consumers
outside Word. It is independent of the OfficeMath dependency chain and lands
first so the consuming Symbolic integration can use a reviewed public API.
F-228 then establishes the equation model. F-229 and F-230 depend on that
model and may proceed independently after it lands.

## Definition of done for this sprint

- Authored line, bar, pie, and doughnut charts retain exact editable workbook
  categories, series, values, formulas, and caches after save and reopen.
- Chart packages add a relationship-owned default Office theme only when no
  document theme exists, without replacing or mutating caller-owned themes.
- Authored axes are explicitly visible in schema order, value-axis number
  formats preserve literal percentages, and caller palettes style series and
  required category points.
- Microsoft Word 16.104 opens source-built chart documents without repair. A
  pinned Pages build renders the intended axes, colors, legend, and values,
  then exports semantically exact editable charts.
- OfficeMath authors and round-trips the supported equation tree while
  preserving unsupported sibling XML.
- Supported equations lay out and render against the pinned Word PDF oracle,
  and supported MathML and LaTeX conversions preserve their normalized tree.
- Full verification passes with every deterministic hash explained, public
  package dry-runs remain bounded, and the integrated sprint review is clean.
