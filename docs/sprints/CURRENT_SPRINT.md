# Current Sprint, S12

**Milestone**: M7 DrawingML.

**Goal**: Establish the `oxml-drawing` crate and its schema-ordering foundation,
then model DrawingML colour choices, transform stacks, and colour-map
resolution. A theme colour with its transforms must resolve to the exact RGB
PowerPoint produces without changing the legacy Word colour path.

## Spec references

- `docs/hld/03-architecture.md`, for resolving theme references, colour
  transforms, and inherited properties before the renderer consumes them.
- `docs/hld/05-drawingml-model.md`, for the `oxml-drawing` module boundary,
  schema child order, colour choices, three-stage colour resolution, transform
  semantics, and the prohibition on changing Word's legacy tint and shade path.
- `docs/hld/12-testing-strategy.md`, for sentence-named regressions,
  round-trip coverage, exact differential colour samples, and the unchanged
  deterministic output gates.
- `docs/hld/13-risks-and-open-questions.md`, for the child-ordering risk that
  `OrderedRawChildren` controls and the later corpus-wide repair gate.
- `docs/hld/14-development-backlog.md`, for the F-052 through F-056 contracts,
  dependencies, sizes, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for keeping development crates at
  version 0.0.0 with publication disabled during PowerPoint development.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-052 | Create oxml-drawing and namespace constants | S | in-progress | codex |
| F-053 | OrderedRawChildren | M | in-progress | codex |
| F-054 | Colour choices | M | in-progress | codex |
| F-055 | The colour transform stack | L | in-progress | codex |
| F-056 | Colour map resolution | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-052 creates the crate boundary required by every later story. F-053 and F-054
are semantically independent after that foundation, but both change the new
crate root and manifest, so they run in separate waves. F-055 follows both. It
depends on F-054 and the completed shared unit types from F-014, and it consumes
F-053's ordering helper. F-056 runs last because colour-map resolution feeds
mapped theme colours into the completed transform stack.

## Definition of done for this sprint

- `oxml-drawing` compiles with namespace constants matching the specification,
  stays at version 0.0.0 with publication disabled, and introduces no forbidden
  format-specific dependency.
- `OrderedRawChildren` preserves modelled and unmodelled siblings in exact
  schema order through parse and serialise cycles.
- `a:srgbClr`, `a:schemeClr`, `a:sysClr`, and `a:prstClr` parse and round-trip.
- Every colour transform is applied in document order. The 40 reviewed theme
  colour and transform pairs resolve to PowerPoint's exact RGB values using
  linear-gamma tint and shade and HSL luminance operations.
- A dark master's `p:clrMap` and `p:clrMapOvr` resolution correctly inverts
  `bg1` and `tx1` before theme lookup.
- Word's legacy tint and shade behaviour and all 28 deterministic output hashes
  remain unchanged. The full workspace gate passes and no crate is published.
