# Current Sprint, S10

**Milestone**: M5 PDF backend.

**Goal**: Complete the staged PDF backend so PDF and raster output cover every
paint and element form currently exposed by the shared layout types. Add
transform-aware gradients, recursive raster groups, paths, clips, dashes, and
page backgrounds while keeping released dependencies and every development
crate publication boundary unchanged.

## Spec references

- `docs/hld/03-architecture.md`, for the format-neutral `oxml-pdf` boundary and
  its permitted dependencies on `oxml-layout` and `oxml-media` only.
- `docs/hld/08-rendering-spec.md`, for PDF shading resources, gradient matrices,
  recursive raster transforms, clip masks, path paint, dashes, and backgrounds.
- `docs/hld/11-migration-plan.md`, for finishing the staged backend before
  PowerPoint implementation and deferring every released rdocx cutover.
- `docs/hld/12-testing-strategy.md`, for sampled gradient pixels, rotated raster
  geometry, dashed-line gaps, exact golden-PNG comparison, and the hash gate.
- `docs/hld/13-risks-and-open-questions.md`, for the silent-output-drift risk
  and the exact rendering evidence required to control it.
- `docs/hld/14-development-backlog.md`, for the F-043 and F-045 contracts,
  dependencies, sizes, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for deterministic rendering and the rule
  that development crates remain at 0.0.0 with publication disabled.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-043 | Gradient shading dictionaries | L | pending | - |
| F-045 | Rasteriser: groups, paths, gradients, dashes | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-043 lands first because F-045 depends on its gradient behaviour as well as
the completed F-040 group and F-041 path work. The PDF gradient resources must
therefore establish the shared paint semantics before the raster backend proves
the same semantics across transforms, clips, dashes, and backgrounds.

## Definition of done for this sprint

- PDF output emits type 2 axial and type 3 radial shadings with type 3
  stitching functions, element-local matrices, normalized stops, and a sampled
  rotated-linear-gradient pixel regression.
- Raster output recursively composes group transforms, applies clip masks,
  renders paths and gradients, honours dash patterns, and uses the page
  background instead of a hardcoded white fill.
- A rotated rectangle at 72 DPI has the expected interior and empty-corner
  pixels, and a dashed line contains deterministic gaps.
- All seven golden page-one buffers match exactly, the existing 28-entry hash
  harness remains unchanged, and the injected one-pixel proof still fails
  precisely.
- `oxml-pdf` remains at version 0.0.0 with publication disabled and without an
  `rdocx-*` or `rpptx-*` dependency. No crate is published.
- The full workspace, no-default-features, WASM, documentation, package, and
  supply-chain gates pass.
