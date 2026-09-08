# Current Sprint, S71

**Milestone**: M23 From-scratch business documents.

**Goal**: make a blank document own the complete package, style, theme, font,
numbering, and deterministic identifier foundation required by the private
business-document corpus. The public facade must create each foundation from
scratch, save and reopen it without repair, and preserve deterministic package
and rendering behavior. The approved scope exceptions also harden Kevin
Brown's PR 71 authored Word charts and verify the S70 Issue 69 fixes before the
issue is closed.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the approved modern DOCX
  capability rows and the F-243 through F-249 completion boundaries.
- `docs/hld/03-architecture.md`, for facade ownership, typed projection seams,
  style and numbering identities, and preservation of unmodeled XML.
- `docs/hld/04-opc-and-packaging.md`, for deterministic package construction,
  content types, relationship ownership, part naming, and integrity checks.
- `docs/hld/05-drawingml-model.md`, for shared RGB colour and Office theme
  ownership used by authored charts.
- `docs/hld/08-rendering-spec.md`, for deterministic layout inputs and the
  field, numbering, theme, and font behavior that authored state must drive.
- `docs/hld/09-charts-spec.md`, for chart serialization, editable workbook
  payloads, axes, legends, palettes, and viewer fidelity.
- `docs/hld/10-bindings-spec.md`, for the native facade and re-export boundary.
- `docs/hld/12-testing-strategy.md`, for public and private source-built
  conformance, save-reopen, deterministic-font, package, and Word oracle gates.
- `docs/hld/13-risks-and-open-questions.md`, for bounded completeness,
  private-corpus containment, and atomic cross-part invariant controls.
- `docs/hld/14-development-backlog.md`, for the F-243 through F-249 acceptance
  contracts, dependencies, and the M23 end gate protected by this sprint.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-243 | Word-compatible fresh package profiles | L | done | - |
| F-249 | Deterministic package identifier allocation | M | done | - |
| F-244 | Corpus settings and document properties | L | done | - |
| F-245 | Corpus themes, font tables, and embedded fonts | L | done | - |
| F-246 | Corpus style authoring | L | done | - |
| F-247 | Complete numbering level and instance model | L | pending | - |
| F-248 | Style-linked numbering, counters, TOC, and REF | L | pending | - |
| F-X087 | Portable authored Word charts from PR 71 | L | in-progress | codex |
| F-X088 | Verify and close Issue 69 after S70 fixes | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-243 establishes the valid blank package profiles. F-249 then centralizes
identifier allocation before package-owned settings, themes, fonts, styles,
and numbering expand the graph. F-244, F-245, and F-247 may proceed after
those foundations. F-246 follows F-245 so effective styles resolve through the
authored theme and font state. F-248 begins only after F-246 and F-247 complete
both sides of the style-linked numbering invariant.

F-X087 and F-X088 are user-approved scope exceptions. They raise S71 from the
usual maximum of seven F-IDs to nine. F-X087 integrates Kevin Brown's PR 71
through the sprint branch, preserves contributor credit, and reaches `main`
only through `/close-sprint`. F-X088 verifies the three completed S70 fixes
against the live Issue 69 report before posting evidence and closing the issue.

## Definition of done for this sprint

- Public blank-document profiles create deterministic DOCX, DOCM, DOTX, and
  DOTM packages with complete required parts, content types, relationships,
  metadata, strict validation, save-reopen identity, and no repair prompt.
- Settings, core, application, and custom properties, document variables,
  compatibility facts, and defaults are typed, deterministic, removable, and
  preserve unrelated package content.
- Themes, language defaults, font tables, and caller-authorized embedded fonts
  are authored through the public facade and resolve deterministically without
  discovered system fonts.
- Paragraph, character, and table style graphs support defaults, inheritance,
  links, next styles, and conditional table regions with atomic validation and
  effective formatting parity against the sanitized Word oracle.
- Full numbering levels and instances round-trip in schema order with all
  required formats, marker properties, restarts, overrides, and producer
  extensions preserved.
- Style-linked numbering is written transactionally and matches Word for
  counters, continuation, restarts, tables, sections, numbered TOC entries,
  and numbering-aware REF fields.
- Package, style, numbering, bookmark, comment, drawing, part, relationship,
  and content-type identifiers allocate deterministically, reject collisions,
  and produce byte-identical output for equivalent construction orders.
- The public conformance harness and available private-corpus checks pass with
  deterministic fonts, clean save and reopen, and no unexplained fallback.
- Source-built line, bar, pie, and doughnut charts remain editable in Word and
  Pages with deterministic palettes, schema-ordered XML, and a validated
  relationship-owned Office theme.
- Issue 69 is closed only after all six focused S70 regressions and the full
  gate pass on the reviewed S71 SHA, with `@emptinessform` credited and the
  affected v0.13.1 release stated accurately.
