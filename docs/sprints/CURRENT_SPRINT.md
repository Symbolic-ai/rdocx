# Current Sprint, S24

**Milestone**: M10 Renderer.

**Goal**: Make rendered slides useful by laying out shape text inside the
resolved preset text rectangle with concrete insets, wrapping, paragraph
formatting, line stacking, and anchoring. Add bullets, stored and computed
autofit, and vertical text while preserving the frozen resolver boundary and
keeping every PowerPoint development crate unpublished.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the v1 text capabilities and the
  explicit `eaVert` rotated fallback with a diagnostic.
- `docs/hld/03-architecture.md`, for the shared `oxml-layout` text machinery
  and the one-way `rpptx-layout` to `rpptx-render` seam.
- `docs/hld/05-drawingml-model.md`, for the typed text body, body properties,
  paragraph, run, list-style, and bullet inputs.
- `docs/hld/07-inheritance-and-resolution.md`, for concrete body properties,
  nine-level paragraph resolution, bullets, text runs, direction, and autofit
  at the frozen renderer boundary.
- `docs/hld/08-rendering-spec.md`, for content-box construction, line layout,
  anchoring, bullet markers, the autofit ladder, and rotated vertical text.
- `docs/hld/12-testing-strategy.md`, for deterministic rendering and the pinned
  50-deck corpus that exercises ordinary shape text.
- `docs/hld/14-development-backlog.md`, for F-098 through F-101 dependencies,
  split requirement, and focused test gates.
- `docs/hld/15-build-and-toolchain.md`, for the version 0.0.0 and
  publication-disabled policy for every PowerPoint development crate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-098 | Shape text layout | XL | pending | - |
| F-098a | Text content box | M | in-progress | codex |
| F-098b | Paragraph inline resolution | L | in-progress | codex |
| F-098c | Line stacking | M | in-progress | codex |
| F-098d | Text anchoring | S | in-progress | codex |
| F-099 | Bullets | M | pending | - |
| F-100 | Autofit | M | pending | - |
| F-101 | Vertical text | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not merely by F-ID. F-098 is the umbrella
gate for the sequential F-098a through F-098d implementation chain. F-099,
F-100, and F-101 all depend on the completed anchoring child. Their behavior is
logically independent, but they share the same renderer text path and therefore
run in separate waves.

## Definition of done for this sprint

- F-098a through F-098d have their own design and delivery records, and every
  child closes before the F-098 umbrella story closes.
- Shape text uses the preset text rectangle and resolved insets to produce a
  fixed content box with correct wrapping, paragraph resolution, line stacking,
  and vertical anchoring.
- Bottom-centred text lands at an independently computed baseline inside its
  inset content box.
- Character, automatic, and no-bullet forms render with inherited size, colour,
  and font properties, including a visible Unicode mapping for Wingdings
  `F0B7`.
- Stored `normAutofit` values apply verbatim, `spAutoFit` trusts the stored
  extent, `noAutofit` overflows without clipping, and bare `normAutofit` uses
  the quantised 2.5 percent ladder.
- Vertical text renders through a transposed layout and rotated group, while
  `eaVert` degrades to rotated vertical text with a diagnostic.
- Every PowerPoint development crate remains version 0.0.0 with publication
  disabled, no crate is published, and the full workspace gate passes with all
  28 deterministic hashes unchanged.
