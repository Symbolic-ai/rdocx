# Current Sprint, S62

**Milestone**: M21 Presentation depth.

**Goal**: model the two major opaque PresentationML surfaces without executing
untrusted content, and integrate the four reviewed Word reader contributions.
The sprint adds safe OLE, ActiveX, and VBA inventory and mutation, gives the
bounded SmartArt corpus typed editing and deterministic rendering, and exposes
the contributed reader safety facts without weakening XML preservation.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the bounded SmartArt, OLE, and
  ActiveX policy and the rule that executable content remains unexecuted.
- `docs/hld/03-architecture.md`, for the package, model, resolver, layout, and
  renderer boundaries and prefix-tolerant XML ownership.
- `docs/hld/04-opc-and-packaging.md`, for relationship, content-type, payload,
  signature, and atomic package-mutation ownership.
- `docs/hld/06-presentationml-model.md`, for graphic-frame dispatch, SmartArt
  and embedded-object relationships, raw serialization, and copied-part id
  remapping.
- `docs/hld/07-inheritance-and-resolution.md`, for source-scoped preview and
  diagram resolution plus visible deterministic fallbacks and diagnostics.
- `docs/hld/08-rendering-spec.md`, for shared DrawingML and text lowering,
  deterministic page geometry, OLE previews, and bounded SmartArt rendering.
- `docs/hld/10-bindings-spec.md`, for additive native PowerPoint facade
  surfaces without implicit Python, WASM, or CLI exposure.
- `docs/hld/12-testing-strategy.md`, for the pinned presentation corpus,
  exact package preservation, source-built fixtures, and declared PowerPoint
  geometry and SSIM differentials.
- `docs/hld/14-development-backlog.md`, for the F-218, F-219, and F-220
  acceptance gates, dependency order, and the still-open M21 representative
  deck gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-218 | Embedded object and macro inventory | L | in-progress | codex |
| F-219 | SmartArt typed model | L | done | - |
| F-220 | SmartArt layout and rendering | L | in-progress | codex |
| F-X071 | Integrate PRs 61 through 64 | L | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-218 owns safe OLE, ActiveX, and VBA package inventory and can proceed
independently of the SmartArt model. F-219 owns diagram parts, relationships,
typed editing, and unsupported-algorithm preservation. F-220 depends on F-219
and must consume that completed model through the existing resolver, DrawingML,
text, layout, and renderer boundaries rather than create a second diagram
projection.

F-X071 is independent of the PresentationML wave. It adopts the four Word
reader contributions at their reviewed source SHAs, preserves contributor
credit, and hardens the two incomplete submissions before integration.

## Definition of done for this sprint

- OLE objects, ActiveX controls, and VBA projects report exact payload hashes,
  relationships, and signature state without executing content.
- Embedded content can be extracted, replaced, and removed atomically, while
  ordinary edits preserve retained payload bytes and relationships exactly.
- Supported SmartArt data, layout, style, colour, text, and relationship
  ownership remain editable, and unsupported diagram algorithms remain
  byte-preserved through unrelated mutations.
- Supported list, hierarchy, cycle, relationship, matrix, and pyramid diagrams
  render through shared DrawingML and text engines within the declared
  PowerPoint geometry and SSIM thresholds.
- Missing previews and unsupported diagram behavior remain visible through
  deterministic fallbacks and stable diagnostics rather than disappearing.
- Hyperlink, drawing, table, numbering, revision, and field reader facts retain
  namespace identity, schema order, and effective-style semantics.
- Full verification passes with every deterministic hash explained, every
  package archive below 10 MiB, and the bounded sprint review clean.
