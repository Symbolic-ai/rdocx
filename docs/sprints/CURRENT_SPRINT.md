# Current Sprint, S86

**Milestone**: M24, modern DOCX authoring completeness.

**Goal**: establish rich header, footer, and note authoring on the existing
story model, while removing repeated canonical-prefix rebinding from retained
elements. Complete the footnote substrate before extending it to endnotes.

## Spec references

- `docs/hld/03-architecture.md`, for story ownership, part-scoped
  relationships, staged mutation, and the separate footnote and endnote layout
  streams.
- `docs/hld/04-opc-and-packaging.md`, for retained namespace bindings and
  canonical-prefix serialization across Word parts.
- `docs/hld/12-testing-strategy.md`, for the integration, regression, and
  differential gates and the pinned rendering oracle.
- `docs/hld/14-development-backlog.md`, for each story's scope, dependencies,
  and acceptance test gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X133 | Stop rebinding a canonical prefix on every retained element | S | in-progress | codex |
| F-271 | Uniform rich header and footer editing | L | done | - |
| F-272 | Rich footnote authoring | L | in-progress | codex |
| F-273 | Rich endnote authoring | L | pending | - |

## Sequencing note

Rows are listed in dependency order. F-X133 is independent of the authoring
stories and may proceed alongside F-271. F-271 and F-272 build on the completed
story and related-part foundations. Complete and integrate F-272 before starting
F-273, which extends the note-authoring model while preserving a separate
endnote identifier namespace. Reconcile shared story and relationship changes
on the sprint branch before the integrated gate.

## Definition of done for this sprint

- A plain save preserves retained attributes and new namespace bindings without
  repeating canonical-prefix declarations on retained elements.
- Rich content authored in every header and footer variant reopens with the
  correct part-scoped relationships.
- Rich footnotes and endnotes can be created, edited, reordered, and removed.
  Their references, relationship scopes, independent identifiers, numbering,
  placement, continuation, and round-trip structure pass the declared Word
  comparisons.
- Each story passes its own acceptance gate and scoped verification. The final
  integrated result passes the full gate, hash harness, and sprint review before
  `/close-sprint S86`.
