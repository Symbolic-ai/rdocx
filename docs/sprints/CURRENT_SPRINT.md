# Current Sprint, S25

**Milestone**: M10 Renderer.

**Goal**: Complete the renderer milestone with tables, hyperlinks,
slide-number fields, and a visible diagnostic surface. Establish the
deterministic SSIM fidelity harness across the pinned corpus, meet the M10
quality target, and keep every PowerPoint development crate unpublished.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for table, hyperlink, field, and
  diagnostic coverage plus the measurable 150 dpi fidelity bar.
- `docs/hld/03-architecture.md`, for the frozen `rpptx-layout` to
  `rpptx-render` seam and the publication-disabled PowerPoint crate boundary.
- `docs/hld/05-drawingml-model.md`, for typed table grids, rows, cells, spans,
  merge continuations, banding flags, and cell text bodies.
- `docs/hld/07-inheritance-and-resolution.md`, for source-neutral resolved
  tables and diagnostics at the renderer boundary.
- `docs/hld/08-rendering-spec.md`, for page-frame diagnostics, link
  annotations, recursive grouped-content handling, and renderer fallbacks.
- `docs/hld/12-testing-strategy.md`, for the pinned corpus, LibreOffice oracle,
  SSIM thresholds, deterministic 150 dpi rendering, and PowerPoint spot-check.
- `docs/hld/14-development-backlog.md`, for F-102 through F-104 dependencies,
  focused test gates, and the M10 end-of-milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for deterministic font isolation used
  by the SSIM harness and the unpublished version 0.0.0 policy.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-102 | Table rendering | L | done | - |
| F-103 | Hyperlinks, fields and diagnostics | M | done | - |
| F-104 | SSIM fidelity harness | L | done | - |

## Sequencing note

F-102 and F-103 can start independently because their prerequisites closed in
earlier sprints. F-104 follows F-102 so the fidelity corpus measures complete
table rendering rather than recording a baseline with a known missing content
class. The final milestone gate runs once over their integrated result.

## Definition of done for this sprint

- A banded table with merged cells renders concrete fills, borders, margins,
  and cell text without duplicated continuation-cell borders.
- Cell text reuses the fixed-box text layout completed in S24 while preserving
  the documented no-table-cell-autofit fallback.
- Slide-number fields render the correct page number, hyperlinks emit link
  annotations, and supported fallbacks remain visible with diagnostics.
- The deterministic harness renders the pinned corpus to 150 dpi PNGs and
  compares them with the LibreOffice oracle without relying on system fonts.
- Every corpus slide renders without panic, missing output, dimension mismatch,
  or a dropped bounded shape.
- CI records 0.95 SSIM on at least 80 percent of slides as a trend reference.
  Representative fidelity output is accepted against native PowerPoint once
  for M10, with the observation recorded as milestone evidence.
- Every PowerPoint development crate remains version 0.0.0 with publication
  disabled, no crate is published, and the full workspace gate passes with all
  28 deterministic hashes unchanged.
