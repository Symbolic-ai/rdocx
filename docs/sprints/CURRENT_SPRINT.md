# Current Sprint, S13

**Milestone**: M7 DrawingML.

**Goal**: Model DrawingML transforms, custom geometry, and fills so any shape's
outline and fill can be described. Nested transforms must compose to exact
matrices, guide-driven geometry must evaluate deterministically, and every fill
form must preserve its schema structure through round trips.

## Spec references

- `docs/hld/03-architecture.md`, for the `oxml-drawing` ownership boundary and
  the rule that format-neutral crates cannot depend on Word or PowerPoint
  crates.
- `docs/hld/05-drawingml-model.md`, for the `a:xfrm`, geometry, and fill module
  boundaries, schema child ordering, guide environment, `GuideOp` semantics,
  and conversion of `a:arcTo` into cubic Beziers.
- `docs/hld/12-testing-strategy.md`, for sentence-named regressions,
  round-trip coverage, deterministic fixtures, and unchanged output gates.
- `docs/hld/13-risks-and-open-questions.md`, for the preset-geometry provenance
  boundary and why the guide evaluator remains useful before that source is
  settled.
- `docs/hld/14-development-backlog.md`, for the F-057 through F-060 contracts,
  dependencies, sizes, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for deterministic verification and the
  rule that PowerPoint development crates remain unpublished.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-057 | a:xfrm | M | in-progress | codex |
| F-058 | Guide evaluator | L | in-progress | codex |
| F-060 | Fills | L | pending | - |
| F-059 | a:custGeom | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-057, F-058, and F-060 can begin from the completed S12 DrawingML foundation.
F-058 also consumes the shared unit types completed in F-014, while F-060
consumes the colour choices completed in F-054. F-059 runs after F-058 because
custom geometry depends on the complete guide evaluator. The three independent
stories still need isolated ownership because each extends the same crate root.

## Definition of done for this sprint

- `a:xfrm` models offset, extent, child offset and extent, rotation, and flips.
  A nested group transform composes to the hand-computed matrix.
- The full guide operation set evaluates from a seeded environment with adjust
  values, and `a:arcTo` is flattened to cubic Beziers. A hand-written custom
  geometry produces the expected path coordinates.
- `a:custGeom` models adjust lists, guide lists, path lists, and the text
  rectangle. A corpus shape round-trips and evaluates to a closed path.
- `a:noFill`, `a:solidFill`, linear and path `a:gradFill`, `a:pattFill`, and
  stretched or tiled `a:blipFill` with `a:srcRect` round-trip, with gradient
  stops in order.
- Writers preserve schema child order and unmodelled XML, the full workspace
  gate passes with all 28 deterministic hashes unchanged, and no crate is
  published.
