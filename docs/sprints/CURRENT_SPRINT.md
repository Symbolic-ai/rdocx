# Current Sprint, S64

**Milestone**: M21 Presentation depth with a cross-cutting Word correction.

**Goal**: turn common modern content sources into editable or explicitly
preserved slide content. HTML maps a bounded DOM and CSS subset into ordinary
slide shapes, while PDF import offers either a preserved page graphic or a
declared editable subset. Neither path promises arbitrary browser or PDF-engine
compatibility.

Issue 67 identified a restart-pagination regression in the unreleased F-X073
work while this sprint was at its release boundary. S64 also removes that
bounded performance cliff before the separately reviewed stable Word release.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for bounded format support, explicit
  unsupported-content policy, and the prohibition on implied compatibility.
- `docs/hld/03-architecture.md`, for facade-owned import, transactional model
  publication, and reuse of existing package, layout, and rendering layers.
- `docs/hld/04-opc-and-packaging.md`, for relationship-safe embedded resources,
  MIME resolution, and package limits.
- `docs/hld/06-presentationml-model.md`, for editable slide shapes, text,
  tables, pictures, links, and preserved source content.
- `docs/hld/08-rendering-spec.md`, for shared geometry, text, image, and path
  lowering used by deterministic source comparisons, plus complete-boundary
  restart pagination for page-spanning prose.
- `docs/hld/10-bindings-spec.md`, for additive native facade surfaces and the
  rule that Python, WASM, and CLI exposure is explicit rather than implied.
- `docs/hld/12-testing-strategy.md`, for source-built differential fixtures,
  deterministic fonts, external oracle discipline, and unchanged hash gates.
- `docs/hld/14-development-backlog.md`, for the F-224 and F-225 acceptance
  gates, the F-X075 regression boundary, dependency order, and the M21
  completion boundary.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-224 | HTML slide content import | L | done | - |
| F-225 | PDF page content import | L | done | - |
| F-X074 | Tag rpptx-v0.9.0 | S | done | - |
| F-X075 | Preserve restart pagination across page-spanning paragraphs | M | done | - |
| F-X076 | Tag v0.12.0 | S | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

Both stories can begin independently because F-224 depends on the completed
F-110 and F-112 authoring surfaces, while F-225 depends on the completed F-109,
F-110, and F-111 shape and picture surfaces. Their designs must agree on shared
image, text, path, link, diagnostic, and transactional publication semantics.
The integrated sprint review validates those interactions before M21 closes.
F-X075 is a late cross-cutting correction requested after Issue 67 reproduced
an unreleased F-X073 regression. It is independent of the presentation
implementation, but its stable release follows the reviewed
`rpptx-v0.9.0` publication so the selected package families remain separate.
F-X076 owns that later stable publication and its seven reviewed contribution
notifications.

## Definition of done for this sprint

- Source-built HTML projects the declared DOM and CSS subset into editable
  shapes, text, tables, images, and links, then matches the pinned browser
  structure and pixels after save and reopen.
- Unsupported HTML structure and style produce stable source-path diagnostics
  without publishing a partial presentation or implying browser compatibility.
- PDF pages import through both the preserved page-graphic path and the
  declared editable text, raster image, path, and link subset.
- Imported PDF page geometry and source rendering match the pinned reference,
  while font substitutions and unsupported operators remain explicit stable
  diagnostics.
- Both import paths reuse the existing PresentationML authoring, package,
  layout, and rendering surfaces without introducing a second rendering engine.
- A portable source-built representative deck combines comments, sections,
  minimal SmartArt preservation, exact media bytes, timeline fade, signatures,
  the macro-enabled package variant, notes, and a three-up handout. A separate
  captured no-repair signed deck supplies authentic SmartArt release evidence.
  Recorded PowerPoint 16.104 static, movie, A4 portrait notes, and A4 portrait
  handout outputs bind directly to that signed source through one configured
  oracle directory. The exact captured bytes and their saved/reopened form both
  pass the complete package, collaboration, section, media, playback, timing,
  signature, slide-order, and authentic SmartArt semantic contract. All three
  static pages pass exact normalized token
  cardinality and order, 6-pixel full-page ink, and per-region raster
  boundaries, with only the page-one audio rectangle masked. Page three proves
  the complete SmartArt graph and relationships plus visible three-node text
  and ink. Notes pass exact per-page tokens, exact band cardinality, 0.06
  normalized semantic-component size, and 0.35 ink-occupancy boundaries without
  equating placement across different notes masters. Handout output passes
  exact token and 0.05 normalized thumbnail geometry boundaries.
- Full verification passes with every deterministic hash explained, every
  package archive below 10 MiB, and the bounded sprint review clean.
- Page-spanning ordinary prose keeps one recorded pagination pass, publishes
  only complete-boundary restart checkpoints, and reuses bounded warm work
  without weakening any existing unsafe-state exclusion.
- The reviewed `rpptx-v0.9.0` release publishes and independently verifies the
  exact 15-package incubating family before sprint closure.
