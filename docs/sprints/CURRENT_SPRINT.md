# Current Sprint, S63

**Milestone**: M21 Presentation depth.

**Goal**: broaden modern presentation interchange and complete the presenter
and audience output surfaces. The sprint first completes the carried SmartArt
renderer, then adds bounded ODP interchange, preserves modern presentation
package variants, and exports notes pages and audience handouts through the
shared rendering backends.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the bounded SmartArt and modern
  presentation scope, executable-content policy, and unsupported-content
  preservation rules.
- `docs/hld/03-architecture.md`, for package, model, resolver, conversion,
  layout, and renderer ownership without a second format-specific engine.
- `docs/hld/04-opc-and-packaging.md`, for relationship and content-type
  ownership across macro-enabled, template, slide-show, notes, and handout
  package graphs.
- `docs/hld/06-presentationml-model.md`, for SmartArt graphic frames, notes and
  handout master hierarchies, package variants, and raw PresentationML
  preservation.
- `docs/hld/07-inheritance-and-resolution.md`, for producing-scope SmartArt
  resolution, master inheritance, visible fallbacks, and stable diagnostics.
- `docs/hld/08-rendering-spec.md`, for shared DrawingML and text lowering,
  deterministic SmartArt geometry, notes pages, handouts, PDF, and image
  output.
- `docs/hld/10-bindings-spec.md`, for additive native facade surfaces and the
  rule that Python, WASM, and CLI exposure is explicit rather than implied.
- `docs/hld/12-testing-strategy.md`, for the pinned PowerPoint and LibreOffice
  differentials, package round trips, deterministic fonts, and unchanged hash
  harness requirements.
- `docs/hld/14-development-backlog.md`, for the F-220, F-222, F-223, and F-226
  acceptance gates, dependency order, and the still-open M21 representative
  deck gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-220 | SmartArt layout and rendering | L | in-progress | codex |
| F-222 | ODP read and write | L | pending | - |
| F-223 | Modern presentation package variants | M | pending | - |
| F-226 | Notes and handout export | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-220 resumes from its retained S62 worker and must close the two remaining
exact fail-closed validator gaps before obtaining a clean microscope pass.
F-222 waits for that completion because ODP interchange must consume the
supported SmartArt model and renderer rather than introduce another diagram
path.

F-223 depends only on the completed F-218 embedded-content inventory and can
run independently once claimed. F-226 depends on the completed F-217 notes and
handout model and can also run independently. Their integration still follows
F-220 and F-222 where shared package or rendering interactions exist.

## Definition of done for this sprint

- Supported list, hierarchy, cycle, relationship, matrix, and pyramid SmartArt
  render through the shared DrawingML and text engines within their declared
  PowerPoint geometry and image thresholds.
- Unsupported or ambiguous SmartArt programs fail closed and preserve source
  bytes, while the carried worker finishes with a clean microscope review.
- Source-built ODP and PPTX conversions match the pinned LibreOffice structure
  and render records in both directions with stable diagnostics.
- PPTM, POTX, POTM, PPSX, and PPSM fixtures reopen in their original package
  class with executable payloads preserved and never executed.
- Speaker notes, notes pages, and audience handouts render in order through the
  declared master hierarchy with matching text, metadata, and geometry.
- Full verification passes with every deterministic hash explained, every
  package archive below 10 MiB, and the bounded sprint review clean.
