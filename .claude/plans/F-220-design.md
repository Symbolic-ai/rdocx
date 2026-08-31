# F-220, SmartArt layout and rendering

**Status**: approved
**Sprint**: S62
**Size**: L
**Depends on**: F-219

## Problem

`crates/rpptx-layout/src/context.rs` currently resolves every
`GraphicDataPayload::SmartArt` to an unsupported bounds fallback. Once F-219
provides typed data, layout, style, colour, and relationship ownership, the
resolver still needs deterministic placement for the six layout families in
the story and must lower them through existing DrawingML geometry, paint, and
text paths.

SmartArt producers can encode many algorithm variants and cached drawings. The
supported subset must be explicit and must never make an unsupported algorithm
disappear. Geometry and pixels must be compared with a pinned PowerPoint oracle
at declared tolerances, using deterministic fonts on the Rust side.

## Spec reference

- ECMA-376 Part 1, DrawingML diagram layout algorithms, constraints, rules,
  style labels, colour transforms, and presentation nodes.
- `docs/hld/02-scope-and-non-goals.md`, the SmartArt scope table.
- `docs/hld/03-architecture.md`, resolver and renderer seams.
- `docs/hld/07-inheritance-and-resolution.md`, graphic-frame resources,
  visible unsupported fallbacks, group transforms, and scoped media.
- `docs/hld/08-rendering-spec.md`, "The renderer's input", deterministic
  presentation entry points, text shaping, and SmartArt fallback behavior.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The deck corpus", and
  native presentation differential evidence.
- `docs/hld/14-development-backlog.md`, "F-220, SmartArt layout and
  rendering".

## Approach

Complete and integrate F-219 first, then consume its exact public diagram
types. Do not introduce a second SmartArt object tree or parse XML in the
layout crate.

Add a private `crates/rpptx-layout/src/diagram.rs` module. It receives one
resolved F-219 resource set, graphic-frame bounds, theme and colour mapping,
and the caller's existing `FontManager`. It returns the existing
`ResolvedContent::Group` made only from backend-neutral paths, fills, strokes,
transforms, and shaped text. `rpptx-render` gains no SmartArt branch, module,
dependency, or public API.

Recognize only a declared mapping of layout definition identities and
algorithm trees to these families:

- List places presentation nodes in one horizontal or vertical sequence.
- Hierarchy derives levels from typed parent connections and lays out bounded
  trees with assistants in their declared lane.
- Cycle places nodes at equal angular intervals with deterministic start angle
  and clockwise direction from the layout definition.
- Relationship places two to six nodes around the declared shared centre and
  renders the supported connector relationships.
- Matrix places row-major nodes in a bounded rectangular grid with optional
  title bands.
- Pyramid divides the available height by weighted levels and centres each
  progressively wider tier.

Each family applies checked layout constraints and rules in schema order.
Unsupported algorithms, condition functions, impossible graphs, invalid
constraints, non-finite results, or resource failures retain the current
visible bounds fallback and emit a stable diagnostic naming the layout identity
and unsupported reason. A failure in one diagram does not hide sibling shapes.

Style labels choose existing DrawingML shape styles. Colour labels resolve
through the existing theme colour map and transform logic. Node text uses the
F-219 `CT_TextBody` value and the shared DrawingML text engine, including
deterministic font selection, paragraph layout, vertical anchoring, and clipping.
Connectors use existing path and stroke values. The outer graphic-frame
transform remains authoritative, and the complete diagram group is clipped to
its bounds.

Package assembly adds scoped parsed diagram resources beside the existing
scoped chart resources. Layout lookup is restricted to the producing slide,
layout, or master. The existing static, timeline, media, and animation facade
paths all use the same assembly and therefore gain SmartArt rendering without a
new entry point.

No new trait, generic, feature, crate, dependency, integration binary, or
binary asset is added. The new module is private and the published API remains
unchanged.

## Rejected alternatives

- Rendering only the cached diagram drawing would not make supported layouts
  respond to F-219 node edits.
- Reimplementing DrawingML text or paint inside a SmartArt renderer would make
  two engines disagree on fonts, colours, and clipping.
- Passing SmartArt XML into `rpptx-render` would reverse the resolver boundary.
- Guessing unsupported layout algorithms from point count would create
  plausible but incorrect output without a diagnostic.
- Recording only screenshot hashes would hide geometry errors behind
  antialiasing noise.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `smartart_list_hierarchy_cycle_relationship_matrix_and_pyramid_place_nodes_deterministically` | Exact normalized node bounds, order, connector endpoints, finite transforms, and clipping for all six families. |
| unit | `smartart_constraints_rules_styles_and_colours_resolve_in_declared_order` | Supported constraint and rule precedence, style labels, theme mapping, colour transforms, and text bounds use shared engines. |
| regression | `unsupported_or_invalid_smartart_remains_visible_and_does_not_hide_siblings` | Stable diagnostic and bounds fallback appear for unsupported algorithms, malformed graphs, missing parts, and non-finite geometry. |
| integration | `smartart_rendering_uses_producing_scope_and_updates_after_node_edit` | Colliding relationship ids do not alias, one F-219 text edit changes only the owning diagram, and static, timeline, media, and animation paths agree. |
| differential | `supported_smartart_corpus_matches_pinned_powerpoint_geometry_and_ssim` | Every supported family records oracle identity, source and render hashes, exact dimensions, node geometry, diagnostics, maximum point error, and luminance SSIM. |
| regression | `smartart_differential_rejects_geometry_and_pixel_perturbations` | A one-point node displacement and calibrated pixel mutation fail the declared thresholds. |

The exact backlog **test gate is differential**: "The supported corpus renders
within the declared PowerPoint geometry and SSIM thresholds."

Use Microsoft PowerPoint 16.104, Info.plist build 16.104.25121423, and
AppleScript build 1214 as the pinned oracle already used by the presentation
timeline differential. Generate source decks in code for list, hierarchy,
cycle, relationship, matrix, and pyramid. Keep source decks, PowerPoint PNGs,
and raw extraction outside the repository under the ignored corpus policy.
Commit only a textual manifest that pins every source and oracle artifact by
SHA-256 and records exact slide, output dimensions, node bounds, and expected
diagnostics.

Render Rust output with bundled deterministic fonts at literal 150 dpi.
Normalize the verified PowerPoint image once to the exact Rust dimensions.
Foreground node geometry permits at most 1 point absolute error per edge and
global luminance SSIM must be at least 0.99. These thresholds match the existing
PowerPoint timeline differential and are proven sensitive by the negative
mutation test. Missing artifacts fail when `RDOCX_PPTX_CORPUS_REQUIRED` is set
and skip with an explicit reason otherwise. Classify every divergence as a Rust
defect, an oracle limitation, or a documented ECMA-376 interpretation.

Add unit tests inside the new layout module and integration cases to the
existing `crates/rpptx/tests/integration.rs` binary. Do not add an integration
binary or committed binary fixture.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout, pagination, line breaking, and text shaping: use bundled
  deterministic fonts for every unit baseline and differential render. Run
  `cargo test -p oxml-layout --no-default-features` in addition to the normal
  gate.
- Crate dependency graph and cross-family uses: consume `rpptx-oxml` only from
  `rpptx-layout` and keep `rpptx-render` on backend-neutral values. Run
  `cargo tree -p rpptx-layout -e normal`, `cargo tree -p rpptx-render -e normal`,
  and the shared-crate dependency-direction test.
- New module or file: explicit approval is required for
  `crates/rpptx-layout/src/diagram.rs`. Six related algorithms share constraint,
  style, colour, and text lowering while remaining separate from the already
  large general resolver.
- External oracle comparison: follow
  `.claude/skills/differential-testing.md`. Pin PowerPoint 16.104 and both build
  identities, SHA-256-pin the source and rendered artifacts, use literal 150
  dpi, permit at most 1 point geometry error, require luminance SSIM at least
  0.99, fail required-corpus mode on missing evidence, and classify every
  divergence.

The public API is unchanged. No published-crate API rider is added.

## Hash harness

Expected unchanged, 49 of 49. The existing hash harness does not contain a
SmartArt presentation render. Any delta is unexplained and blocks integration.

## Implementation checklist

- [ ] Complete the F-219 dependency-prefix checkpoint and reconcile its final
  diagram model and resource types.
- [ ] Add the approved private layout module and scoped diagram resource path.
- [ ] Implement checked list, hierarchy, cycle, relationship, matrix, and
  pyramid placement.
- [ ] Resolve supported constraints, rules, style labels, colours, connectors,
  and text through shared engines.
- [ ] Preserve visible bounded fallbacks and stable diagnostics for every
  unsupported or invalid case.
- [ ] Reuse the same resolved group across static, timeline, media, and
  animation assembly.
- [ ] Add deterministic unit, integration, differential, and negative-sensitivity
  tests to existing targets.
- [ ] Run the pinned PowerPoint corpus gate and every routed rider.

## Open questions

None. The private layout module and the PowerPoint 16.104 differential at 150
dpi, at most 1 point geometry error, and luminance SSIM of at least 0.99 are
approved.
