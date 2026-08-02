# Current Sprint, S20

**Milestone**: M9 Inheritance resolver.

**Goal**: Resolve every inherited property needed by the slide model. Build
the placeholder, transform, body, list-style, format-scheme, and typeface
chains so later resolved-slide work receives concrete values rather than theme
or hierarchy references.

## Spec references

- `docs/hld/05-drawingml-model.md`, for the typed theme, format scheme,
  style-reference, text-style, colour, and font inputs consumed by resolution.
- `docs/hld/06-presentationml-model.md`, for placeholder matching, slide to
  layout to master relationships, and the preserved PresentationML model.
- `docs/hld/07-inheritance-and-resolution.md`, for the six inheritance chains,
  seven-source list-style cascade, `ResolveCtx` boundary, and concrete output
  rules.
- `docs/hld/14-development-backlog.md`, for the five F-ID dependencies and
  their focused test gates.
- `docs/hld/15-build-and-toolchain.md`, for keeping PowerPoint development
  crates at version 0.0.0 with publication disabled.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-081 | ResolveCtx skeleton and placeholder chain | M | in-progress | codex |
| F-082 | Effective transform and body properties | M | pending | - |
| F-083 | The seven-step list style merge | L | pending | - |
| F-084 | Format scheme reference resolution | M | pending | - |
| F-085 | Typeface resolution | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not implementation priority. F-081
establishes the context and placeholder chain required by F-082 and F-083.
F-084 and F-085 depend only on completed M7 theme and shape work, so they can
proceed independently while the F-081 chain is being established.

## Definition of done for this sprint

- A slide placeholder resolves to its matching layout and master placeholders.
- A placeholder without its own transform inherits the layout position, and
  effective body properties follow the documented defaults and chain.
- Text properties merge through all seven sources and all nine list levels,
  with later sources winning per property and level.
- Format-scheme fill, line, effect, and font references use one-based indices,
  apply the background-fill rule above 1000, and substitute `phClr` from the
  shape reference.
- Theme tokens including `+mn-lt`, `+mj-lt`, `+mn-ea`, and `+mn-cs` resolve to
  the correct theme and per-script typefaces.
- The full workspace gate passes with all 28 deterministic hashes unchanged,
  and every PowerPoint development crate remains version 0.0.0 and
  unpublished.
