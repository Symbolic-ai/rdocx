# F-128, Preserved chart fallback

**Status**: approved
**Sprint**: S32
**Size**: S
**Depends on**: F-125

## Problem

`rpptx-layout` currently turns every chart graphic frame into an empty bounds
fallback at `crates/rpptx-layout/src/context.rs:494`, even when its relationship
targets a supported typed chart. `ResolvedContent` at
`crates/rpptx-layout/src/lib.rs:150` has no backend-neutral grouped-content
case, and `rpptx-render` consequently lowers only the fallback rectangle at
`crates/rpptx-render/src/lib.rs:317`.

The F-124 integration test proves that authored ChartML can be parsed and
rendered directly, but it bypasses slide relationship resolution and the
presentation rendering pipeline. Preserved unsupported charts also lack the
required cached-picture selection and labelled diagnostic placeholder.

## Spec reference

- `docs/hld/03-architecture.md`, PresentationML ownership and dependency
  direction.
- `docs/hld/04-opc-and-packaging.md`, scoped relationship resolution and target
  normalization.
- `docs/hld/06-presentationml-model.md`, graphic frames, shape-tree alternate
  content, and raw subtree preservation.
- `docs/hld/07-inheritance-and-resolution.md`, source-scoped resources and
  visible unsupported-content diagnostics.
- `docs/hld/08-rendering-spec.md`, `RenderInput`, backend-neutral groups,
  deterministic font mode, and media lowering.
- `docs/hld/09-charts-spec.md`, "Rendering" and "What is not in v1".
- `docs/hld/14-development-backlog.md`, "F-128, Preserved chart fallback".

## Approach

Add the minimum read-only chart relationship projection to the existing
graphic-frame and alternate-content models. Resolve `c:chart@r:id` by namespace
URI, retain the original raw bytes as the only serialization source, and expose
an immediate typed picture fallback when an `mc:AlternateContent` chart choice
provides one.

Add source-scoped chart resources beside the existing media and hyperlink
resources. Upstream package assembly resolves the chart relationship target,
parses the target through `CT_ChartSpace`, and carries any compatible cached
picture as a resolved media reference. Missing or external relationships remain
contextual diagnostics and never fall through to another relationship scope.

Extend the owned resolver contract with one backend-neutral grouped-content
variant. A chart-aware resolver entry point receives the scoped chart resources
and the caller's `FontManager`. It passes supported typed charts, their frame
bounds, the effective theme, and effective colour map to the F-127 renderer,
then freezes the returned `GroupElement` in `ResolvedContent`. The ordinary
renderer inserts that group beneath the graphic frame transform without adding
a second chart-specific backend path.

If ChartML is opaque or native rendering returns an unsupported projection,
select the cached picture when present and record a stable unsupported-chart
diagnostic. Otherwise emit a visible labelled rectangle inside the frame and
record the same diagnostic category. Parsing failures and missing targets keep
their more specific diagnostics. Direct supported chart graphic frames and
chart choices in alternate content use the same routing function.

## Rejected alternatives

- Keep manually parsing the chart part in integration tests. That does not
  exercise relationship scope, slide placement, fallback selection, or
  `RenderInput`.
- Put ChartML logic in a PDF or raster backend. The chart crate already emits
  backend-neutral groups, and format-neutral backends must remain unaware of
  charts.
- Cross the frozen resolver boundary with raw ChartML. The resolver can freeze
  the finished `GroupElement` instead.
- Drop unsupported charts or leave an unlabelled rectangle. The v1 contract
  requires visible content and a diagnostic.
- Add a renderer or package-assembly trait. Each boundary has one concrete
  implementation today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `three_dimensional_chart_uses_cached_image_and_diagnostic` | A 3-D chart choice lowers its cached picture in frame bounds and records the stable unsupported-chart diagnostic |
| integration | `authored_chart_relationship_enters_presentation_renderer` | An `rpptx::add_chart` package resolves slide relationship to ChartML and emits native paths and labels through the presentation pipeline |
| integration | `same_chart_relationship_id_is_scoped_to_its_source_part` | Equal identifiers in slide, layout, and master scopes resolve only to their owning chart targets |
| regression | `unsupported_chart_without_preview_keeps_labelled_bounds` | Missing cached media produces a visible labelled placeholder and diagnostic rather than an empty frame |
| negative | `missing_or_external_chart_relationship_is_contextual` | Missing, external, and missing-target relationships keep source scope and identifier in diagnostics |
| round-trip | `chart_choice_and_picture_fallback_remain_byte_preserved` | Read-only alternate-content projection leaves every original byte and child order unchanged on serialization |
| golden | `supported_and_fallback_charts_render_deterministically` | Native geometry and cached-image fallback produce identical repeated deterministic renders |

The test gate is: a 3-D chart renders its cached image and records a diagnostic.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/09-charts-spec.md`

Record the approved inward presentation-crate edges, read-only chart and
alternate-content projection, scoped chart resources, frozen group boundary,
native routing, cached-picture fallback, and labelled diagnostic placeholder.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Resolve by namespace URI,
  preserve alternate content and chart payload bytes as the sole serialization
  source, and add exact byte, child-order, alias, and malformed-value tests.
- Crate dependency graph, a new `use` across families. Read HLD 03. Run the
  dependency-direction check and confirm every new edge remains inward among
  `rpptx-*` crates or from them to format-neutral `oxml-*` crates.
- Layout and text shaping. Read HLD 08. Shape the placeholder and chart labels
  with the caller's `FontManager`, use deterministic fonts for raster evidence,
  and never record a system-font baseline.

No published API, binding, feature, new file, external oracle, or unit-conversion
rider applies. All chart frame coordinates are already points after resolution.

## Hash harness

Expected unchanged. The new presentation-only routing does not enter the Word
sample generator or renderer. All 28 hashes must match.

## Implementation checklist

- [ ] Project chart relationship identifiers and paired picture fallbacks
      without changing their raw serialization source.
- [ ] Assemble source-scoped parsed chart resources and cached media.
- [ ] Add a backend-neutral resolved group content case and lower it normally.
- [ ] Route supported charts through the F-127 renderer with effective theme
      and colour-map inputs.
- [ ] Route unsupported charts to cached pictures or a labelled diagnostic
      placeholder.
- [ ] Replace the F-124 bypass test with an end-to-end presentation rendering
      assertion while retaining its editable-data evidence.
- [ ] Add focused relationship, preservation, diagnostic, and deterministic
      render tests to existing test binaries.
- [ ] Update exactly the five listed HLD files.
- [ ] Run focused checks, routed checks, microscope, and worker preparation.

## Open questions

None. A cached chart image is the immediate typed `p:pic` fallback paired with
the chart choice in preserved `mc:AlternateContent`. Direct chart graphic frames
without that fallback use native geometry when supported and labelled bounds
when unsupported.
