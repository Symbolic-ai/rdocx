# Current Sprint, S22

**Milestone**: M10 Renderer.

**Goal**: Settle the licensed source for preset shape definitions, generate a
reproducible checked-in preset table, and evaluate known and unknown preset
geometry without dropping visible content. Establish the `rpptx-render` input
boundary so M10 can consume the frozen resolver contract with relationship
scopes and media resolved correctly, while keeping every PowerPoint development
crate unpublished.

## Spec references

- `docs/hld/03-architecture.md`, for the `rpptx-layout` to `rpptx-render` seam
  and one-way dependency graph.
- `docs/hld/08-rendering-spec.md`, for the preset generator, evaluator,
  fallback, and `RenderInput`, `SlideBundle`, and `RelScopes` contracts.
- `docs/hld/12-testing-strategy.md`, for the 50-deck corpus and render-fidelity
  evidence requirements.
- `docs/hld/13-risks-and-open-questions.md`, for preset-table provenance and
  rejection of the MPL-2.0 LibreOffice source.
- `docs/hld/14-development-backlog.md`, for F-089 through F-092 dependencies,
  story gates, and the M10 boundary.
- `docs/hld/15-build-and-toolchain.md`, for checked-in generated artifacts and
  the version 0.0.0, publication-disabled policy.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-089 | Resolve the preset geometry licensing question | S | in-progress | codex |
| F-092 | rpptx-render skeleton and RenderInput | M | in-progress | codex |
| F-090 | Preset table generator | L | in-progress | codex |
| F-091 | Preset evaluation and fallback | M | in-progress | codex |

## Sequencing note

F-089 must settle the permitted source before F-090 generates the preset table.
F-091 then consumes that table. F-092 depends only on the completed F-036 and
F-087 contracts, so it may proceed in parallel with the F-089 through F-091
chain. The row order puts the two independent starting stories first.

## Definition of done for this sprint

- A written HLD decision records the preset geometry source and its licensing
  basis. No MPL-2.0 LibreOffice table or code is used.
- The offline preset generator covers every preset name found in the corpus,
  checks its output into the repository, and regenerates byte-identically.
- Known presets evaluate through the generated table. An unknown preset falls
  back to its shape bounds, preserves its text, and emits a diagnostic.
- The `rpptx-render` skeleton consumes the frozen resolver contract through
  `RenderInput`, `SlideBundle`, and three distinct relationship scopes. Equal
  relationship IDs in slide, layout, and master scopes resolve to their own
  targets.
- The package and dependency direction follows the architecture contract.
  Every PowerPoint development crate remains version 0.0.0 with publication
  disabled, and no crate is published to crates.io.
- The full workspace gate passes with all 28 deterministic hashes unchanged.
