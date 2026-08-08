# Current Sprint, S26

**Milestone**: M11 Write API.

**Goal**: Establish the slide-creation foundation with a bundled zero-slide
template, collision-safe shape and media allocation, synthesised slide creation,
and a complete validation surface. End with a three-slide generated deck that
opens without repair and a validator that accepts the pinned corpus while
identifying every deliberately corrupted package.

## Spec references

- `docs/hld/01-glossary.md`, for placeholder `idx` as the inheritance join key
  that new slides must preserve.
- `docs/hld/02-scope-and-non-goals.md`, for the bundled `Presentation::new()`,
  synthesised `add_slide`, slide collection, and content-hash media deduplication
  contract.
- `docs/hld/03-architecture.md`, for the `rpptx` facade and its crate-local
  `assets/default.pptx` ownership.
- `docs/hld/04-opc-and-packaging.md`, for cheap non-panicking package integrity
  validation before debug saves.
- `docs/hld/06-presentationml-model.md`, for slide and shape identifier rules,
  recursive shape-tree allocation, the nine-step slide synthesis sequence,
  every `ValidationIssue`, and the bundled template contents.
- `docs/hld/12-testing-strategy.md`, for pinned corpus validation and the
  milestone requirement to open saved decks without repair.
- `docs/hld/13-risks-and-open-questions.md`, for synthesising placeholders
  instead of deep-copying raw XML with hidden relationship identifiers.
- `docs/hld/14-development-backlog.md`, for F-105 through F-108 dependencies,
  sizes, and focused test gates.
- `docs/hld/15-build-and-toolchain.md`, for the default-template feature,
  crate-local asset packaging, and unpublished PresentationML crates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-105 | Bundled default.pptx | M | in-progress | codex |
| F-106 | ShapeIdAllocator and MediaStore | M | in-progress | codex |
| F-107 | add_slide | L | in-progress | codex |
| F-108 | validate() | M | in-progress | codex |

## Sequencing note

F-105 and F-106 have completed prerequisites and can begin independently.
F-107 follows both because slide synthesis needs the bundled template plus safe
shape and media allocation. F-108 follows F-107 so validation covers the new
creation path and protects every later M11 mutation story.

## Definition of done for this sprint

- `Presentation::new()` loads a crate-local 16:9 template containing one
  master, eleven standard layouts, a full theme, notes infrastructure, table
  styles, and zero slides, and PowerPoint opens it without repair.
- `ShapeIdAllocator` scans nested groups and `mc:AlternateContent` fallbacks,
  while `MediaStore` deduplicates equal image bytes by content hash.
- `add_slide` synthesises minimal non-latent placeholders, preserves their
  `type` and `idx`, creates the layout and presentation relationships, registers
  the content type, and allocates unique slide ids at or above 256.
- A generated three-slide deck opens in PowerPoint without repair and preserves
  the selected layouts and placeholder inheritance.
- `Presentation::validate()` reports every documented `ValidationIssue` without
  panicking, runs under debug assertions before save, detects one deliberate
  corruption per variant, and accepts all 50 pinned corpus decks.
- The default template is included in the `rpptx` package through the existing
  default-on feature. Every PowerPoint development crate remains version 0.0.0
  with publication disabled, no crate is published, the full workspace gate
  passes, and all 28 deterministic hashes remain unchanged.
