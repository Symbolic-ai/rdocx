# Current Sprint, S49

**Milestone**: M16 Document generation and analysis.

**Goal**: evaluate the field codes real documents are full of. The field
instruction parser establishes the structured representation that evaluation,
cached-result policy, and later templating build on.

## Spec references

- `docs/hld/03-architecture.md`, for `rdocx-oxml` ownership of the
  WordprocessingML grammar, prefix-tolerant readers, and raw subtree
  preservation.
- `docs/hld/04-opc-and-packaging.md`, for package-preserving reader behaviour
  and schema-ordered serialization.
- `docs/hld/12-testing-strategy.md`, for unit and round-trip evidence.
- `docs/hld/14-development-backlog.md`, for the field-parser, evaluator, and
  update-policy story contracts and acceptance gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-160 | Field instruction parser | L | in-progress | codex |
| F-161 | Field evaluation engine | L | pending | - |
| F-162 | Field update policy | M | pending | - |
| F-203 | Reader compatibility corrections | M | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-160 defines the recursive field structure. F-161 consumes that structure and
also depends on F-154. F-162 depends on evaluation because it decides when a
field result may be recomputed. F-203 independently corrects reader
preservation found during upstream review.

## Definition of done for this sprint

- Every supported simple and complex field form parses into field name,
  arguments, switches, and cached-result structure without discarding unknown
  WordprocessingML XML.
- Supported fields evaluate to the value Word computes for the pinned expected
  set.
- Field-result update policies preserve cached results unless the selected
  policy permits recomputation.
- The M16 end gate is ready for template syntax, loops, conditionals, and
  repeating table rows to consume the field representation.
