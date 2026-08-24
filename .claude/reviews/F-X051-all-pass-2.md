# F-X051, all, pass 2

**Reviewed**: uncommitted working diff, 4 files, 654 insertions and 40 deletions, 694 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, replacing additional fonts drops aliases for the constructor font universe
`crates/oxml-layout/src/font.rs:333`

`new_with_fonts` records label and embedded-family metadata for its initial
caller fonts, and those faces become `base_db`. A later changed
`load_additional_fonts` call restores that database but clears both metadata
maps, then rebuilds them only from the replacement slice. The constructor
faces therefore remain loaded while their document-facing labels and caller
priority disappear. For example, a manager created with Caladea bytes labelled
`Document Serif` resolves that label until any different additional-font set
is loaded, after which the same label no longer reaches the still-loaded
Caladea face. Both public loading paths must preserve the alias behavior of
every caller font that remains in the database.

### D2, caller-face matching no longer follows the established CSS selection
`crates/oxml-layout/src/font.rs:1105`

`best_caller_face` collapses every non-normal stretch into one rank and chooses
weight by absolute distance. That is not the CSS-like matching used by
`fontdb::Database::query`, which resolves stretch by direction and has
directional weight preferences. With caller faces at weights 300 and 500, a
normal 400 request must prefer 500 when 400 is absent. This implementation can
instead select 300 when it appears first because both weights have the same
absolute distance. The same tie problem occurs for a bold 700 request with
600 and 800 faces. Exact caller-family and alias resolution can therefore
select the wrong face bytes based on input order.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D1: label-derived aliases now retain the exact caller-loaded face ids,
  and the focused same-family bundled-font regression passes.
- Pass 1 D2: case-only labels now retain a label alias under the lookup's
  case-sensitive family semantics, and the focused regression passes.
- Reusable-engine context: exact alias identity participates in cache and
  transfer compatibility, changed aliases invalidate dependent state, and
  rejected transfers preserve both engines.
- Facade contract: default, option-taking, and checked-transfer alias-aware
  bundled-fallback methods are present. Existing strict and bundled-fallback
  signatures remain unchanged.
- Panics: no new production panic, unchecked indexing, slicing, or arithmetic
  hazard was found.
- OOXML: the diff does not parse or serialize XML, and no schema-order,
  namespace, whitespace, or unmodelled-subtree issue was found.
- Structure: no unjustified trait, generic, wrapper, feature flag, crate,
  module, or file was introduced.
