# Current Sprint, S49

**Milestone**: M16 Document automation.

**Goal**: evaluate the field codes that real Word documents use throughout
their content. This sprint establishes the instruction grammar, evaluation
engine, and explicit cache-update policy that every later M16 template feature
depends on.

## Spec references

- `docs/hld/03-architecture.md`, for WordprocessingML field ownership in
  `rdocx-oxml`, facade coordination, and the format-neutral layout boundary.
- `docs/hld/08-rendering-spec.md`, for existing PAGE, NUMPAGES, REF, and
  PAGEREF evaluation during Word pagination and stored-display fallback.
- `docs/hld/10-bindings-spec.md`, for the native structured field and bookmark
  surface and unchanged Python, WASM, and CLI compatibility.
- `docs/hld/12-testing-strategy.md`, for unit, regression, round-trip, and
  differential evidence using readable in-code fixtures and pinned oracles.
- `docs/hld/14-development-backlog.md`, for the M16 end gate, the three field
  stories, their strict dependencies, and their exact acceptance gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-160 | Field instruction parser | L | in-progress | codex |
| F-161 | Field evaluation engine | L | pending | - |
| F-162 | Field update policy | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-160 defines the grammar consumed by every later field operation. F-161 then
evaluates that grammar and also depends on the bookmark model from F-154.
F-162 is last because its update-on-demand, update-on-save, and leave-alone
policies control when the F-161 evaluator may replace a stored result.

## Definition of done for this sprint

- Simple and complex fields parse into a field name, arguments, and switches,
  including nested fields and instructions split across runs.
- IF, REF, PAGEREF, SEQ, DOCPROPERTY, DOCVARIABLE, STYLEREF, INCLUDETEXT, DATE,
  TIME, FILENAME, AUTHOR, and MERGEFIELD evaluate to the pinned Word results.
- Formatting switches apply without regressing existing PAGE and NUMPAGES
  pagination or stored-display fallback.
- Update on demand, update on save, and leave alone produce their documented
  result caches and Word dirty flags.
- Unsupported fields retain their cached result instead of becoming blank.
