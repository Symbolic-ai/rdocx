# Current Sprint, S28

**Milestone**: M11 Write API.

**Goal**: Complete the M11 write API with mutable tables, slide collection
operations, and slide and presentation properties. End with a generated
10-slide deck that exercises the complete write surface and opens cleanly in
PowerPoint, Keynote, Google Slides, and LibreOffice.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the complete v1 table, slide
  collection, presentation property, and cross-viewer surface.
- `docs/hld/03-architecture.md`, for facade ownership in `rpptx` and dependency
  direction through PresentationML, DrawingML, and package crates.
- `docs/hld/04-opc-and-packaging.md`, for relationship rewriting, media
  transfer, content types, and package integrity during slide duplication.
- `docs/hld/05-drawingml-model.md`, for table grids, merge spans, continuation
  cells, banding flags, widths, and schema-ordered writing.
- `docs/hld/06-presentationml-model.md`, for slide ordering, deep copy,
  relationship scopes, backgrounds, hidden slides, and slideshow content type.
- `docs/hld/07-inheritance-and-resolution.md`, for resolved table styling,
  merge ownership, slide backgrounds, and hidden-slide behavior.
- `docs/hld/12-testing-strategy.md`, for round-trip, corpus, deterministic
  rendering, and pinned PowerPoint, Keynote, Google Slides, and LibreOffice
  acceptance evidence.
- `docs/hld/13-risks-and-open-questions.md`, for schema child ordering and safe
  edits in the presence of preserved XML.
- `docs/hld/14-development-backlog.md`, for F-113 through F-116 dependencies,
  sizes, focused test gates, and the M11 milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for the pinned native and LibreOffice
  tools and the unpublished PowerPoint development crates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-113 | Table facade | L | pending | - |
| F-114 | remove_slide, move_slide, duplicate_slide | M | pending | - |
| F-115 | Slide and presentation properties | S | pending | - |
| F-116 | Cross-viewer acceptance | M | pending | - |

## Sequencing note

F-113, F-114, and F-115 have independent completed prerequisites and may
proceed before the acceptance story. F-116 depends on the complete F-107
through F-115 write surface, so it runs last and closes M11.

## Definition of done for this sprint

- `add_table` exposes cells, text, formatting, banding, and column widths.
  Merging and then splitting cells restores the original grid.
- Removing, moving, and duplicating slides preserve valid presentation order,
  ids, relationships, media, and custom-show references. Duplicated images
  resolve through the new slide's own relationships.
- Slide size, background, hidden state, core properties, and slideshow output
  each survive save and reload with valid content types and package graphs.
- A generated 10-slide deck exercising every M11 feature validates cleanly and
  opens without repair in PowerPoint, Keynote, Google Slides, and LibreOffice.
- The full workspace gate passes, all 28 deterministic hashes remain unchanged,
  every PowerPoint development crate remains unpublished at version 0.0.0, and
  no crate is published.
