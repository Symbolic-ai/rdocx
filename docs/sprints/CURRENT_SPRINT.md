# Current Sprint, S19

**Milestone**: M8 PresentationML.

**Goal**: Open any deck and read it through the public `rpptx` facade. Build
the read-only presentation, slide, shape, and text access surface, then prove
modelled round-trip integrity across all 50 pinned decks and complete the
manual PowerPoint repair-prompt gate without publishing any PowerPoint
development crate.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the in-scope presentation, slide,
  shape, text, and notes read surface.
- `docs/hld/03-architecture.md`, for the `rpptx` facade boundary, read-only
  handle conventions, dependency direction, and unpublished versioning.
- `docs/hld/04-opc-and-packaging.md`, for deterministic package saves, main
  part discovery, relationship resolution, and package integrity.
- `docs/hld/06-presentationml-model.md`, for the typed part and shape models,
  document order, preservation strategy, and presentation validity rules.
- `docs/hld/12-testing-strategy.md`, for the modelled corpus round-trip,
  part-by-part comparison, python-pptx oracle, and manual PowerPoint open gate.
- `docs/hld/14-development-backlog.md`, for the F-079 and F-080 contracts and
  the M8 end-of-milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for keeping implemented `oxml-*` and
  `rpptx-*` crates at version 0.0.0 with publication disabled.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-079 | The rpptx read facade | L | in-progress | codex |
| F-080 | Modelled round-trip gate | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order. F-079 builds the public
facade over the completed PresentationML and OPC models. F-080 depends on that
facade and exercises the saved package produced through it, so its structural,
part-by-part, and manual PowerPoint gates run only after F-079 is integrated.

## Definition of done for this sprint

- `Presentation::open`, `from_bytes`, and `to_bytes` expose ordered slides,
  shapes, text, notes, and read-only handle types without panicking on indexed
  access.
- A `dump_deck` example matches python-pptx shape and text output across the
  pinned corpus for the supported read surface.
- Every one of the 50 pinned decks parses, serialises, reparses to a
  structurally equal model, and passes the saved-package part comparison.
- Every saved corpus deck is opened manually in PowerPoint without a repair
  prompt, completing the M8 gate with recorded evidence.
- The full workspace gate passes with all 28 deterministic hashes unchanged,
  and every PowerPoint development crate remains version 0.0.0 and
  unpublished.
