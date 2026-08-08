# Current Sprint, S27

**Milestone**: M11 Write API.

**Goal**: Add mutable shape and text APIs to the presentation facade, including
direct geometry and styling setters, shape creation, picture insertion, and
text-frame editing. End with saved decks that round-trip every mutation, render
edited placeholder text, and open in PowerPoint without repair.

## Spec references

- `docs/hld/01-glossary.md`, for placeholder identity and the `idx` join key
  that text mutation must preserve.
- `docs/hld/02-scope-and-non-goals.md`, for the complete v1 shape, picture, and
  text mutation surface.
- `docs/hld/03-architecture.md`, for facade ownership in `rpptx` and the
  dependency direction through PresentationML and shared DrawingML crates.
- `docs/hld/04-opc-and-packaging.md`, for image sniffing, intrinsic EMU sizing,
  media naming, and package integrity after picture insertion.
- `docs/hld/05-drawingml-model.md`, for transforms, geometry adjustments,
  fills, lines, text bodies, whitespace, and schema-ordered writing.
- `docs/hld/06-presentationml-model.md`, for recursive shape-tree handles,
  collision-safe non-visual ids, presentation writing, and validation.
- `docs/hld/07-inheritance-and-resolution.md`, for the direct properties that
  mutation adds before the resolver collapses them to concrete render values.
- `docs/hld/12-testing-strategy.md`, for round-trip, rendering, corpus, and
  native PowerPoint acceptance evidence.
- `docs/hld/13-risks-and-open-questions.md`, for schema child ordering and safe
  mutation in the presence of preserved XML.
- `docs/hld/14-development-backlog.md`, for F-109 through F-112 dependencies,
  sizes, and focused test gates.
- `docs/hld/15-build-and-toolchain.md`, for keeping every PowerPoint development
  crate at version 0.0.0 with publication disabled.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-109 | Shape mutation facade | L | in-progress | codex |
| F-110 | add_textbox, add_shape, add_connector, group | M | in-progress | codex |
| F-111 | add_picture | M | in-progress | codex |
| F-112 | Text frame mutation | L | in-progress | codex |

## Sequencing note

F-109 establishes the mutable shape handle used by F-110 and F-112, which can
then proceed independently. F-111 is independent of F-109 because its F-106
media store and F-026 intrinsic-size prerequisites are already complete.

## Definition of done for this sprint

- Shape position, size, rotation, name, fill, line, and adjustment setters each
  survive save and reload without disturbing unmodelled XML.
- `add_textbox`, `add_shape`, `add_connector`, and `add_group_shape` allocate
  collision-free ids, emit schema-ordered XML, validate cleanly, and produce
  shapes that PowerPoint opens without repair.
- `add_picture` deduplicates media through the existing content-addressed store
  and uses native dimensions when no explicit size is supplied.
- Text frames expose paragraphs, runs, font properties, and bullets for
  mutation. Setting text on a placeholder round-trips and renders visibly.
- The full workspace gate passes, native PowerPoint acceptance covers the new
  shape constructors, all 28 deterministic hashes remain unchanged, every
  PowerPoint development crate remains unpublished at version 0.0.0, and no
  crate is published.
