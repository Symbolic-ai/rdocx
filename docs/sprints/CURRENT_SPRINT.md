# Current Sprint, S17

**Milestone**: M8 PresentationML.

**Goal**: Model placeholders, cropped pictures, graphic-frame payload dispatch,
and DrawingML tables inside the ordered shape tree while preserving unsupported
payloads verbatim. Prove the matching, crop, dispatch, and merged-cell contracts
against focused fixtures and the pinned deck corpus without publishing any
PowerPoint development crate.

## Spec references

- `docs/hld/03-architecture.md`, for `rpptx-oxml` ownership, dependency
  direction, raw XML preservation, and the unpublished 0.0.0 boundary.
- `docs/hld/05-drawingml-model.md`, for shared picture fills, crop rectangles,
  schema child order, fixed write prefixes, and verbatim unsupported XML.
- `docs/hld/06-presentationml-model.md`, for the shape-tree child union,
  placeholder keys and matching rules, picture and graphic-frame ownership,
  and the preservation strategy.
- `docs/hld/12-testing-strategy.md`, for the pinned 50-deck corpus and the
  structural, raw, and modelled round-trip gates.
- `docs/hld/14-development-backlog.md`, for the F-071 through F-074 contracts,
  dependencies, sizes, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for keeping implemented `oxml-*` and
  `rpptx-*` crates at version 0.0.0 with publication disabled.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-071 | Placeholders | M | in-progress | codex |
| F-072 | Pictures | M | pending | - |
| F-073 | Graphic frames | M | pending | - |
| F-074 | DrawingML tables | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not implementation-wave order. F-071 and
F-073 depend on the completed F-070 shape tree. F-072 depends on the completed
F-060 fill model, and F-074 depends on the completed F-064 text model. All
declared prerequisites are already integrated. F-073 owns graphic-data payload
dispatch while F-074 owns the typed table payload, so their design plans must
make that integration boundary explicit before parallel waves are assigned.

## Definition of done for this sprint

- Placeholder keys parse and write `p:ph`, match by shared `idx` when present,
  otherwise match by type with absent type defaulting to body, and honour the
  title and body equivalence classes.
- A cropped `p:pic` round-trips with its `p:blipFill` relationship and
  `a:srcRect` crop rectangle intact.
- `p:graphicFrame` recognises table, chart, SmartArt, and OLE graphic-data
  payload kinds while preserving every unmodelled payload verbatim.
- DrawingML tables preserve grid, rows, cells, merge origins, spans, and
  banding flags, including a merged-cell round-trip fixture.
- The relevant placeholder, picture, graphic-frame, and table payloads in all
  50 pinned decks pass structural round-trip and unsupported-XML preservation
  checks.
- The full workspace gate passes with all 28 deterministic hashes unchanged,
  and no PowerPoint development crate is published.
