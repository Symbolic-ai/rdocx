# Current Sprint, S47

**Milestone**: M14 Word collaboration layer.

**Goal**: read, write, and resolve tracked revisions. The sprint first gives
WordprocessingML revisions a typed model with stable identity and metadata,
then adds accept and reject operations that reproduce Word's document state.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the post-v1 roadmap boundary and
  the requirement that content outside a typed model remains preserved
  verbatim.
- `docs/hld/03-architecture.md`, for the `rdocx-oxml` ownership boundary,
  ordered raw XML preservation, and `rdocx` facade coordination of document
  mutations.
- `docs/hld/04-opc-and-packaging.md`, for package integrity and the rule that a
  failed validation or mutation leaves document and package state unchanged.
- `docs/hld/10-bindings-spec.md`, for the stable native Word facade and the
  requirement that additive Rust APIs preserve existing Python, WASM, and CLI
  surfaces.
- `docs/hld/12-testing-strategy.md`, for round-trip and regression test gates,
  deterministic in-code fixtures, and normalized external-oracle comparison.
- `docs/hld/14-development-backlog.md`, for the M14 boundary, the two revision
  story definitions, their dependency, and their exact acceptance gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-149 | Revision model | L | in-progress | codex |
| F-150 | Accept and reject revisions | L | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-149 comes first because F-150 must resolve typed revision identity, metadata,
content, and recorded prior properties before it can accept or reject a scoped
change. F-150 then applies those transformations atomically and compares the
accepted result with Word's normalized body XML.

## Definition of done for this sprint

- Insertions, deletions, moves, deleted text, and run, paragraph, table, and
  section property changes survive load and save unchanged.
- Every modeled revision reports its author, timestamp, kind, and identity
  through the public native Word API.
- Accept and reject operations work across all revisions and when scoped by
  author, date range, or revision id.
- Rejecting an insertion removes its content, rejecting a deletion restores
  its content, and rejecting a property change restores the recorded prior
  value.
- Accepting every revision produces the normalized body XML that Word produces
  from the same input.
