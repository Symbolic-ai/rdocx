# Current Sprint, S23

**Milestone**: M10 Renderer.

**Goal**: Build the first concrete `rpptx-render` output path so slides with
backgrounds, shapes, gradients, outlines, rotations, groups, arrowheads, and
cropped or tiled pictures render without text. Preserve the frozen resolver
boundary and keep every PowerPoint development crate unpublished while S24
adds shape text.

## Spec references

- `docs/hld/03-architecture.md`, for the one-way `rpptx-layout` to
  `rpptx-render` seam and the shared `oxml-layout` output boundary.
- `docs/hld/05-drawingml-model.md`, for geometry, fill, line, transform,
  arrowhead, picture-fill, and raw-preservation inputs.
- `docs/hld/06-presentationml-model.md`, for the recursive shape tree and typed
  picture crop and relationship data consumed by rendering.
- `docs/hld/07-inheritance-and-resolution.md`, for the owned `ResolvedSlide`
  contract, accumulated group transforms, concrete paint, and effective
  backgrounds.
- `docs/hld/08-rendering-spec.md`, for page-frame lowering, group recursion,
  paths, gradients, images, backgrounds, and the renderer input boundary.
- `docs/hld/12-testing-strategy.md`, for deterministic rendering, sampled-pixel
  evidence, the 50-deck corpus, and the later M10 fidelity gate.
- `docs/hld/14-development-backlog.md`, for F-093 through F-097 dependencies,
  story gates, and the M10 boundary.
- `docs/hld/15-build-and-toolchain.md`, for deterministic font mode and the
  version 0.0.0, publication-disabled policy.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-093 | Shape geometry, fills and lines | L | done | - |
| F-096 | Pictures with crop and tile | M | done | - |
| F-097 | Backgrounds | S | done | - |
| F-094 | Rotation, flips and groups | M | done | - |
| F-095 | Arrowheads | S | done | - |

## Sequencing note

F-093 consumes the completed preset evaluator and renderer input boundary, and
it blocks F-094 and F-095. F-096 depends on the existing picture and media
contracts, while F-097 depends on the completed flattener, so both may proceed
alongside F-093. Rotation, flips, groups, and arrowheads follow only after the
base shape lowering is stable.

## Definition of done for this sprint

- Preset and custom shape geometry lower to page-frame paths with solid,
  gradient, and outline paint matching sampled colours.
- Rotation, flips, and nested group transforms place corners at independently
  computed coordinates.
- Line head and tail ends lower to the required filled arrowhead paths.
- Pictures render through content-addressed media with crop and tile behavior
  proven by focused image-region tests.
- Slide, layout, and master backgrounds lower through the resolved background
  contract, including inherited gradient backgrounds.
- Slides containing shapes and pictures render without dropped visible content.
  Shape text remains explicitly deferred to S24.
- Every PowerPoint development crate remains version 0.0.0 with publication
  disabled, no crate is published, and the full workspace gate passes with all
  28 deterministic hashes unchanged.
