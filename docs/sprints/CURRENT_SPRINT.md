# Current Sprint, S31

**Milestone**: M12 Charts.

**Goal**: Turn the completed ChartML model into authored and natively rendered
presentation content. Add charts through the owning `rpptx` facade, lower the
supported plot families to geometry, and complete axes, gridlines, labels, and
legend output so S32 can focus on colour and preserved-chart fallback polish.

## Spec references

- `docs/hld/03-architecture.md`, for ownership across `rpptx`, `rpptx-chart`,
  `rpptx-render`, `oxml-layout`, and the embedded `oxml-sml` workbook.
- `docs/hld/04-opc-and-packaging.md`, for relationship resolution, content-type
  registration, package validation, and collision-safe numbered part names.
- `docs/hld/06-presentationml-model.md`, for the owning facade, borrowed shape
  handles, staged package mutation, and canonical graphic-frame insertion.
- `docs/hld/08-rendering-spec.md`, for the backend-neutral page-frame types,
  path and text lowering, render ownership, and deterministic raster output.
- `docs/hld/09-charts-spec.md`, for the `add_chart` package graph, cache and
  workbook consistency, plot geometry, nice-number axes, labels, and legend.
- `docs/hld/12-testing-strategy.md`, for corpus, differential, deterministic
  raster, and native PowerPoint acceptance evidence.
- `docs/hld/13-risks-and-open-questions.md`, for schema child ordering, raw XML
  preservation, and containment of the self-contained chart subsystem.
- `docs/hld/14-development-backlog.md`, for F-124 through F-126 dependencies,
  sizes, focused test gates, and the later M12 milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for the unpublished chart and
  presentation crate boundaries and their allowed dependency direction.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-124 | add_chart | L | pending | - |
| F-125 | Chart rendering: geometry | L | pending | - |
| F-126 | Chart rendering: axes, gridlines and labels | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order. F-124 and F-125 both use
the completed S30 plot model and may begin independently because authoring owns
the presentation package while geometry owns renderer output. F-126 follows
F-125 so axis layout, gridlines, tick labels, and the legend share one reviewed
plot rectangle and scale boundary.

## Definition of done for this sprint

- `add_chart` atomically writes the chart part, embedded workbook, slide and
  chart relationships, content-type overrides, and canonical graphic frame. A
  generated chart opens in PowerPoint and Edit Data shows the source values.
- Bars, lines, wedges, areas, and markers lower from cached chart data into
  finite backend-neutral paths. A representative bar chart rasterises with bars
  at the computed positions.
- Nice-number value-axis ticks, axis lines, gridlines, tick labels, and the
  legend lower into the same plot geometry. A 0 to 100 value axis produces the
  expected tick set.
- The full workspace gate passes, all 28 deterministic hashes remain unchanged
  unless a design plan declares a reviewed delta, and development chart crates
  remain unpublished.
