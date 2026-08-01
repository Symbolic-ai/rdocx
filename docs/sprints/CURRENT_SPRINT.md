# Current Sprint, S14

**Milestone**: M7 DrawingML.

**Goal**: Model the DrawingML line, effect, shape-property, style-reference,
and text vocabularies needed to describe a business-deck shape. Preserve
unmodelled XML at its schema boundary, keep every writer in schema order, and
split the XL text story into reviewable sub-IDs before implementation.

## Spec references

- `docs/hld/03-architecture.md`, for the `oxml-drawing` ownership boundary and
  the rule that format-neutral crates cannot depend on Word or PowerPoint
  crates.
- `docs/hld/05-drawingml-model.md`, for the line, effect, shape property, style
  reference, and text modules, plus schema order, significant `a:t`
  whitespace, and raw-XML preservation.
- `docs/hld/08-rendering-spec.md`, for the downstream shape-text layout and
  autofit semantics that the text model must retain without implementing the
  renderer here.
- `docs/hld/12-testing-strategy.md`, for sentence-named regressions,
  round-trip coverage, deterministic inline fixtures, and the external deck
  corpus boundary.
- `docs/hld/13-risks-and-open-questions.md`, for the deck corpus, schema-order,
  raw-preservation, and DrawingML scope risks.
- `docs/hld/14-development-backlog.md`, for the F-061 through F-064 contracts,
  dependencies, sizes, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for deterministic verification and the
  rule that PowerPoint development crates remain unpublished.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-061 | Lines | M | pending | - |
| F-062 | Effects | S | pending | - |
| F-064 | DrawingML text model | XL | pending | - |
| F-064a | Text body properties and shell | M | pending | - |
| F-064b | Text paragraphs and runs | L | pending | - |
| F-064c | Text bullets | S | pending | - |
| F-064d | Nine-level list styles | M | pending | - |
| F-063 | Shape properties and style references | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-061 consumes the colour model completed in F-054. F-062 and F-064a can begin
from the completed raw-child foundation in F-053. F-064b follows F-064a,
F-064c follows F-064b, and F-064d follows both F-064b and F-064c. The F-064
parent closes only after those four children. F-063 follows F-061 because
shape properties compose the completed fill and line models and own the
style-reference boundary.

## Definition of done for this sprint

- `a:ln` models width, dash presets, cap, join, and head and tail ends, with
  every `ST_PresetLineDashVal` mapped to a dash array.
- `a:effectLst` models outer shadow and preserves unsupported effects such as
  glow byte for byte at their schema boundary.
- `a:spPr` and the four style-reference forms write in schema order, and
  `fillRef@idx = 1001` resolves to background fill style 1.
- F-064 is split before implementation, and the resulting text model preserves
  `a:t` whitespace through `xml:space` while structurally round-tripping the
  available schema-valid fixtures. The external corpus gate remains required
  at the M7 boundary.
- The full workspace gate passes with all 28 deterministic hashes unchanged,
  and no crate is published.
