# Current Sprint, S64

**Milestone**: M21 Presentation depth.

**Goal**: turn common modern content sources into editable or explicitly
preserved slide content. HTML maps a bounded DOM and CSS subset into ordinary
slide shapes, while PDF import offers either a preserved page graphic or a
declared editable subset. Neither path promises arbitrary browser or PDF-engine
compatibility.

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
  lowering used by deterministic source comparisons.
- `docs/hld/10-bindings-spec.md`, for additive native facade surfaces and the
  rule that Python, WASM, and CLI exposure is explicit rather than implied.
- `docs/hld/12-testing-strategy.md`, for source-built differential fixtures,
  deterministic fonts, external oracle discipline, and unchanged hash gates.
- `docs/hld/14-development-backlog.md`, for the F-224 and F-225 acceptance
  gates, dependency order, and the M21 completion boundary.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-224 | HTML slide content import | L | done | - |
| F-225 | PDF page content import | L | done | - |
| F-X074 | Tag rpptx-v0.9.0 | S | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

Both stories can begin independently because F-224 depends on the completed
F-110 and F-112 authoring surfaces, while F-225 depends on the completed F-109,
F-110, and F-111 shape and picture surfaces. Their designs must agree on shared
image, text, path, link, diagnostic, and transactional publication semantics.
The integrated sprint review validates those interactions before M21 closes.

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
- Full verification passes with every deterministic hash explained, every
  package archive below 10 MiB, and the bounded sprint review clean.
- The reviewed `rpptx-v0.9.0` release publishes and independently verifies the
  exact 15-package incubating family before sprint closure.
