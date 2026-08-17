# Current Sprint, S45

**Milestone**: M15 Charts beyond PowerPoint.

**Goal**: make the chart engine serve Word as well as PowerPoint. The engine is
already format-neutral below its crate name, so this sprint moves it to the
shared family, adds native editable Word chart authoring, and routes the same
backend-neutral geometry through the Word paginator.

## Spec references

- `docs/hld/03-architecture.md`, for the shared-crate dependency rule and the
  current ChartML-to-layout seam that F-156 moves without changing behaviour.
- `docs/hld/04-opc-and-packaging.md`, for deterministic package saves,
  relationship handling, and collision-safe chart and workbook part naming in
  F-157.
- `docs/hld/09-charts-spec.md`, for the typed ChartML model, editable embedded
  workbook, atomic authoring, and backend-neutral rendering contracts reused by
  all four stories.
- `docs/hld/12-testing-strategy.md`, for the byte-identical hash gate,
  round-trip evidence, and exact pixel comparison required across the move and
  the Word rendering path.
- `docs/hld/14-development-backlog.md`, for the M15 boundary, the four story
  definitions, their dependency chain, and the editable native-chart milestone
  gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-156 | Extract oxml-chart | L | done | - |
| F-157 | Word chart part and embedded workbook | M | done | - |
| F-158 | Document::add_chart | M | done | - |
| F-159 | Chart rendering in the Word paginator | M | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

The chain is strict. F-156 first performs only the crate move and compatibility
shim, with no behaviour change and no hash delta. F-157 then gives Word a chart
part, relationship, content type, and embedded workbook. F-158 can expose the
Word authoring API only after those package pieces exist, and F-159 can route a
chart through pagination only after the authored Word shape is available.

The hash baseline is exclusive during F-156 and must stay at 49 of 49. No story
in this sprint is expected to move it. F-157 through F-159 build on the settled
shared-crate path rather than overlapping the move.

## Definition of done for this sprint

- `rpptx-chart` moves to `oxml-chart` behind the established deprecation shim,
  every existing chart test uses the new path, and the hash harness remains
  byte-identical at 49 of 49.
- A Word document saves a native chart part, its document relationship, content
  type, chart-to-workbook relationship, and editable embedded workbook, and
  Microsoft Word opens the result without repair.
- `Document::add_chart` authors bar, line, and pie charts with the requested
  series, categories, and number formats.
- An inline or anchored Word chart renders through the Word paginator and is
  pixel-identical to the same chart on a slide at the same size.
