# Current Sprint, S50

**Milestone**: M16 Document automation.

**Goal**: turn substitution into generation with a template model that remains
correct when Word splits tags across formatted runs. Build nested structural
loops and conditionals on that model, then preserve table and list behavior
when rows and items repeat from a data source.

## Spec references

- `docs/hld/03-architecture.md`, for WordprocessingML tree ownership and the
  native facade boundary that coordinates structural document mutation.
- `docs/hld/04-opc-and-packaging.md`, for package preservation, schema child
  order, numbering integrity, and fail-closed mutation.
- `docs/hld/10-bindings-spec.md`, for the native Rust surface and unchanged
  Python, WASM, and CLI compatibility boundaries.
- `docs/hld/12-testing-strategy.md`, for readable in-code fixtures,
  round-trip preservation, deterministic rendering, and regression gates.
- `docs/hld/14-development-backlog.md`, for the M16 end gate and the exact
  F-163, F-164, and F-165 scope, dependencies, and test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-163 | Template syntax | L | in-progress | codex |
| F-164 | Loops and conditionals | L | in-progress | codex |
| F-165 | Repeating table rows and lists | M | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-163 defines how tags are recognized and rewritten across run boundaries.
F-164 depends on that exact syntax and adds paragraph, row, and section block
semantics over the data model. F-165 follows F-164 because repeating tables
and lists reuse its structural iteration while adding merged-cell, banding,
and continuous-numbering rules.

## Definition of done for this sprint

- A tag split across five differently formatted runs resolves while preserving
  the surrounding formatting and unmodelled XML.
- A readable fixture with a nested loop and conditional produces the expected
  document from its data model.
- Repeating a three-row table template over ten records produces thirty rows
  with merged cells and banding intact.
- Repeated list items retain continuous numbering across every generated item.
- Structural updates preserve package relationships, schema order, and the
  unchanged hash-harness baseline unless a plan declares a reviewed delta.
