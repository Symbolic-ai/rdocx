# Current Sprint, S09

**Milestone**: M5 PDF backend.

**Goal**: Make nested groups and backend-neutral paths render in the staged PDF
backend, then move all three collection passes onto transform-aware traversal.
Add reusable alpha graphics states while keeping the released dependency graph
and every development-crate publication boundary unchanged.

## Spec references

- `docs/hld/03-architecture.md`, for the staged dependency direction and the
  rule that shared backends do not depend on released format crates.
- `docs/hld/08-rendering-spec.md`, for group graphics-state nesting, path
  operators, accumulated transforms, clipping, collection traversal, and
  `/ExtGState` alpha reuse.
- `docs/hld/11-migration-plan.md`, for completing the staged `oxml-pdf` arms
  and walk-based passes before any released shared-crate cutover.
- `docs/hld/12-testing-strategy.md`, for the three nested collection
  regressions, path and balanced graphics-state assertions, exact golden-PNG
  comparison, and the unchanged hash harness.
- `docs/hld/13-risks-and-open-questions.md`, for the output-drift gate and the
  R3 risk of silently losing fonts, images, or links inside groups.
- `docs/hld/14-development-backlog.md`, for the F-040, F-041, F-042, and F-044
  contracts, dependencies, sizes, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for deterministic rendering, package
  verification, and the rule that PowerPoint development crates stay at 0.0.0
  with publication disabled.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-040 | Group rendering | M | done | - |
| F-041 | Path rendering | M | done | - |
| F-042 | Rewrite the three collection passes on walk | M | done | - |
| F-044 | ExtGState alpha | S | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-040 must land before F-042 because the R3 regressions need real nested group
output before they can prove font, image, and link collection. F-041 and F-044
depend only on the completed F-039 page transform, but they share the staged PDF
writer surface with group rendering and must remain distinct reviewable
behavioural commits.

## Definition of done for this sprint

- A three-deep group emits balanced `q` and `Q` operators, composes each `cm`
  in the approved child-to-page order, applies optional `W n` clipping, and
  restores graphics state after every subtree.
- Fill-only, stroke-only, and combined paths emit the required PDF path and
  paint operators, including width, cap, join, miter, and dash state.
- Font subsetting, XObject registration, and link annotation collection all
  use `walk`, with one nested target regression for each pass and transformed
  link rectangles.
- Distinct alpha values reuse one `/ExtGState` each, and a 50 percent fill over
  white rasterises to the midpoint colour.
- All seven golden page-one buffers match exactly, the existing 28-entry hash
  harness remains unchanged, and the injected one-pixel proof still fails
  precisely.
- `oxml-pdf` remains at version 0.0.0 with publication disabled and without an
  `rdocx-*` or `rpptx-*` dependency. No crate is published.
- The full workspace, no-default-features, WASM, documentation, package, and
  supply-chain gates pass.
