# Current Sprint, S30

**Milestone**: M12 Charts.

**Goal**: Type the ChartML axis and plot surfaces on top of the S29 data layer,
covering the seven v1 plot families without weakening schema ordering or raw
preservation. Add data labels and number formats so the completed model is
ready for S31 authoring and rendering work.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the seven chart families included
  in v1 and the boundary that unsupported 3-D content remains preserved.
- `docs/hld/03-architecture.md`, for `rpptx-chart` ownership and the acyclic
  dependency direction between ChartML and the other workspace crates.
- `docs/hld/09-charts-spec.md`, for the axis forms, paired `crossAx` ids, plot
  variants, series attachment, data-label surface, and number formats.
- `docs/hld/12-testing-strategy.md`, for corpus structural round-trip,
  deterministic rendering, and differential evidence requirements.
- `docs/hld/13-risks-and-open-questions.md`, for schema child ordering, raw XML
  preservation, and containment of the chart scope.
- `docs/hld/14-development-backlog.md`, for F-120 through F-123 dependencies,
  sizes, focused test gates, and the M12 milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for the unpublished `rpptx-chart`
  package boundary and the rule that it remains at version 0.0.0.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-120 | Axes | L | done | - |
| F-123 | Data labels and number formats | M | done | - |
| F-121 | Bar and line plots | M | done | - |
| F-122 | Pie, doughnut, area, scatter and radar plots | L | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order. F-120 and F-123 both use
the completed S29 model, but they share the ChartML source and number-format
seam, so F-120 runs first and F-123 reuses that reviewed value. F-121 then
attaches bar and line plots to the typed axes and labelled series. F-122 follows
F-121 so every remaining v1 plot family extends one reviewed plot boundary
instead of introducing a competing representation.

## Definition of done for this sprint

- Category, value, date, and series axes parse and write in schema order, with
  scaling, gridlines, ticks, labels, number formats, and consistent paired
  `crossAx` identifiers.
- Bar and line plots round-trip and render, then pie, doughnut, area, scatter,
  and radar plots use the same typed plot boundary and pass their focused
  render gates.
- Data labels and number formats attach to the series model without breaking
  formula, cache, or raw-preservation invariants. A percentage-formatted label
  renders with the expected text.
- Every supported axis and plot surface passes the pinned corpus structural
  round-trip gate, the full workspace gate passes, all 28 deterministic hashes
  remain unchanged unless a design plan declares a reviewed delta, and
  `rpptx-chart` remains unpublished at version 0.0.0.
