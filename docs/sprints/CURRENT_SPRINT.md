# Current Sprint, S16

**Milestone**: M8 PresentationML.

**Goal**: Establish the external deck corpus and prove that the OPC layer can
round-trip every deck with all parts opaque. Then model the presentation, slide,
layout, master, and shape-tree parts while preserving everything outside the
rendered contract verbatim.

## Spec references

- `docs/hld/03-architecture.md`, for `rpptx-oxml` ownership, dependency
  direction, and the format-neutral crate boundary.
- `docs/hld/04-opc-and-packaging.md`, for deterministic package saves, part
  resolution, canonical PresentationML paths, and byte-identical round-trips.
- `docs/hld/05-drawingml-model.md`, for the DrawingML shape and text contracts
  exercised by the carried M7 corpus gate.
- `docs/hld/06-presentationml-model.md`, for the core parts, schema order,
  shape-tree variants, identifier constraints, and raw XML preservation.
- `docs/hld/12-testing-strategy.md`, for the external 50-deck corpus, the
  carried DrawingML structural gate, and raw and modelled round-trip evidence.
- `docs/hld/13-risks-and-open-questions.md`, for the requirement to settle a
  redistributable or fetched corpus source before M8 implementation proceeds.
- `docs/hld/14-development-backlog.md`, for the F-067 through F-070 contracts,
  sizes, dependencies, test gates, and M8 milestone boundary.
- `docs/hld/15-build-and-toolchain.md`, for keeping `rpptx-oxml` at version
  0.0.0 with publication disabled until PowerPoint development is complete.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-067 | Create rpptx-oxml and the corpus harness | M | in-progress | codex |
| F-068 | presentation.xml | M | pending | - |
| F-069 | Slide, layout and master parts | L | pending | - |
| F-070 | The shape tree | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order. F-067 settles the corpus
source, creates the harness, executes the carried M7 DrawingML gate, and proves
opaque package round-trips before any M8 XML modelling begins. F-068 then
establishes the presentation root. F-069 and F-070 may proceed after that
boundary because their DrawingML prerequisites, F-064 and F-063 respectively,
are already complete.

## Definition of done for this sprint

- The external corpus source is pinned and reproducible, remains outside every
  published crate, and supplies all 50 expected decks.
- The carried M7 gate passes for every `a:txBody` and `a:spPr` in the corpus
  before M8 model work begins.
- The unpublished `rpptx-oxml` crate opens and saves all 50 decks with every
  part opaque and byte-identical.
- Every corpus presentation part parses, serialises, and reparses structurally
  with slide order, identifier bounds, and unsupported XML preserved.
- Every corpus slide, layout, and master part round-trips structurally with its
  required relationships and colour-map forms intact.
- A deck containing nested groups round-trips with shape-tree document order,
  required group properties, and all six child variants preserved.
- The full workspace gate passes with all 28 deterministic hashes unchanged,
  and no crate is published.
