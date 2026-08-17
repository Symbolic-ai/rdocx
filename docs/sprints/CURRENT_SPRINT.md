# Current Sprint, S48

**Milestone**: M14 Word collaboration layer.

**Goal**: close M14 by making tracked revisions visible during rendering and
making the author's document-protection intent readable through the public API.
The sprint combines the revision model completed in S47 with settings-level
protection metadata, then proves the full collaboration-layer milestone gate.

## Spec references

- `docs/hld/03-architecture.md`, for the `rdocx-oxml` ownership boundary, the
  typed revision model, and facade coordination across document and settings
  state.
- `docs/hld/04-opc-and-packaging.md`, for package integrity, preservation of
  settings metadata, and atomic document changes.
- `docs/hld/08-rendering-spec.md`, for the shared renderer input, Word
  pagination, and deterministic PDF and raster lowering used by the revision
  display gate.
- `docs/hld/10-bindings-spec.md`, for native Word facade stability and the
  requirement that new inspection options leave existing Python, WASM, and CLI
  surfaces compatible.
- `docs/hld/12-testing-strategy.md`, for deterministic golden tests, in-code
  fixtures, and the mixed-document milestone regression.
- `docs/hld/14-development-backlog.md`, for the M14 end gate, the revision
  display and document-protection stories, their dependencies, and their exact
  acceptance gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-151 | Revision display in the renderer | M | done | |
| F-155 | Document protection | M | done | |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-151 depends on the typed revision model completed by F-149 in S47. F-155 is
independent and works through the document settings part, so the two stories
may be implemented in parallel. Both must land before the combined M14 gate can
run because it covers revisions, comments, content controls, and bookmarks in
one document.

## Definition of done for this sprint

- A render option selects the accepted view or the tracked-change view, with
  the accepted view as the default.
- The tracked-change view underlines insertions, strikes through deletions, and
  draws a change bar in the margin.
- The accepted view is pixel-identical to rendering the same document after all
  revisions have been accepted and removed.
- Read-only, comments-only, tracked-changes-forced, and forms-only protection
  modes round-trip with the recorded hash and salt intact.
- The public native Word API reports the document-protection mode and its
  recorded metadata without changing existing binding surfaces.
- One document carrying tracked changes, comments, content controls, and
  bookmarks round-trips byte-identically in every unmodelled part, and all four
  subsystems are readable and writable through the public API.
