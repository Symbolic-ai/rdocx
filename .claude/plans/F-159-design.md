# F-159, Chart rendering in the Word paginator

**Status**: completed
**Sprint**: S45
**Size**: M
**Depends on**: F-158

## Problem

The Word layout path currently turns every inline drawing into
`InlineItem::Image` at `crates/rdocx-layout/src/engine.rs:576`, and its anchored
drawing discriminator chooses only image or shape at
`crates/rdocx-layout/src/engine.rs:774`. The shared line model likewise has no
backend-neutral group item at `crates/oxml-layout/src/line.rs:74`, so chart
geometry cannot travel through line breaking to pagination without being
rasterized.

`Document::build_layout_input` currently resolves image bytes but ignores chart
relationships at `crates/rdocx/src/document.rs:2461`. As a result, the editable
chart authored by F-158 is present in the package but absent from the rendered
page even though `oxml-chart::render_chart` already returns the exact
`GroupElement` consumed by the PDF and raster backends.

## Spec reference

- `docs/hld/03-architecture.md`, `oxml-layout` as the format boundary and the
  shared chart dependency seam.
- `docs/hld/08-rendering-spec.md`, backend-neutral groups, deterministic font
  mode, and Word pagination.
- `docs/hld/09-charts-spec.md`, "Rendering" and package chart relationship
  resolution.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The golden-PNG gate",
  and deterministic render comparison.
- `docs/hld/14-development-backlog.md`, "F-159, Chart rendering in the Word
  paginator".

## Approach

Resolve internal document-to-chart relationships while building
`LayoutInput`. Parse each target as `CT_ChartSpace`, retain it by relationship
ID, parse the document's DrawingML theme as
`CT_OfficeStyleSheet`, and carry the standard `ColorMap`. A missing theme uses
the shared Office default, matching the existing chart renderer contract.
Malformed, external, or missing chart targets remain visible diagnostics rather
than becoming empty images.

Add one generic group-bearing variant to the existing shared line types:

```rust
InlineItem::Group { width: f64, height: f64, group: GroupElement }
LineItem::Group { width: f64, height: f64, group: GroupElement }
```

The line breaker treats it exactly like an image for width, ascent, descent,
wrapping, and line height. The paginator translates its child-local group to
the inline top-left position and appends `PositionedElement::Group`. This is a
backend-neutral layout capability with one concrete chart producer and the
existing PDF and raster consumers. It adds no chart dependency to
`oxml-layout`.

In `rdocx-layout`, detect a typed chart relationship before the picture path.
Call `oxml_chart::render_chart` with local bounds starting at zero, the
document theme, color map, and the layout operation's `FontManager`. Inline
charts become the group line item. Anchored charts become a new concrete
`AnchoredContent::Group` and reuse the current anchor placement, wrapping,
z-order, and translation path. Unsupported typed chart projection emits the
existing visible chart placeholder and a stable diagnostic.

Build the golden comparison from one `ChartData` source and equal point bounds.
Author it once into a Word document and once into a PowerPoint deck. Render
both with bundled fonts at 150 DPI, crop the declared chart rectangles, require
equal dimensions and zero differing RGBA pixels, and record both artifact
SHAs plus the pinned rasterizer identity. The existing `rdocx` test location
uses a dev-only `rpptx` dependency for the PowerPoint artifact. This adds no
production dependency edge. Rasterisation uses the already pinned
`pdftoppm 26.01.0` executable and adds no image-decoder dependency.

## Rejected alternatives

- Rasterize the chart into an image before pagination. That creates a second
  rendering path and cannot prove backend-neutral pixel identity.
- Make `oxml-layout` depend on `oxml-chart`. The line engine needs only an
  opaque shared group, and a chart dependency would reverse the ownership seam.
- Special-case chart drawing in the PDF backend. The renderer already emits
  `PositionedElement` values, so backend branching would duplicate geometry.
- Support only inline charts. The drawing parser already distinguishes inline
  and anchored extents, and the existing anchor placement can carry the same
  group without a second renderer.
- Record a tolerance. The two artifacts use the same chart engine, fonts,
  bounds, and rasterizer, so the story explicitly requires pixel identity.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden, gate | `word_and_powerpoint_chart_pixels_are_identical` | Equal-size chart crops rendered at 150 DPI with bundled fonts have equal dimensions and zero differing RGBA pixels |
| unit | `group_inline_item_breaks_and_positions_like_an_image` | Width, line height, wrap boundary, and top-left translation preserve child-local chart coordinates |
| integration | `inline_word_chart_renders_backend_neutral_group` | An authored inline chart becomes paths, text, and groups rather than an image or empty frame |
| integration | `anchored_word_chart_uses_existing_wrap_and_z_order` | Anchored chart geometry follows its frame, wrap distances, behind-text flag, and resolved position |
| regression | `word_chart_uses_document_theme_and_default_color_map` | Series colors match the same chart rendered from the same effective theme in a deck |
| negative | `missing_or_malformed_word_chart_is_visible` | Missing, external, malformed, or unsupported chart relationships emit contextual diagnostics and a visible placeholder without panic |

The test gate is golden. A chart in a Word document renders pixel-identical to
the same chart on a slide at the same size.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/09-charts-spec.md`
- `docs/hld/12-testing-strategy.md`

Record the Word chart-to-layout dependency, generic inline group transport,
theme and relationship resolution, inline and anchored placement, visible
fallback, and exact cross-family golden gate.

## Risk routing

- Layout and pagination. Read HLD 08. Run every render baseline in bundled-font
  mode, exercise inline and anchored pagination, and never record a system-font
  baseline.
- Any parser or serialiser. Read HLD 04 and HLD 06. Add relationship-target,
  prefix, malformed-chart, round-trip, and preserved-drawing checks without
  changing the ChartML serialization contract.
- Crate dependency graph and new cross-family uses. Read HLD 03. Confirm
  `rdocx-layout -> oxml-chart` points inward, while `oxml-layout` remains free
  of chart and format-family dependencies. Verify both production and
  all-target dependency trees, including the dev-only `rdocx -> rpptx` golden
  edge, remain architecture-compliant.
- Public API of a published crate. Read HLD 10 and the structural rules. State
  the additive non-exhaustive inline group variants and run affected package
  dry-runs and size assertions.
- An external oracle comparison. Follow differential-testing guidance. Bind
  both generated artifacts to SHA-256, run bundled fonts at 150 DPI with the
  pinned rasterizer, and require exact pixel equality with no tolerance.

## Hash harness

Expected unchanged across all 49 entries. The existing sample set has no native
Word chart, so any delta is unrelated and blocks the sprint.

## Implementation checklist

- [x] Resolve Word chart parts, theme, and color map into layout input.
- [x] Add generic inline group transport to the shared line breaker and paginator.
- [x] Render inline and anchored chart relationships through `oxml-chart`.
- [x] Preserve wrapping, z-order, local bounds, diagnostics, and visible fallback.
- [x] Add exact deterministic Word-versus-PowerPoint pixel evidence.
- [x] Run focused layout, renderer, dependency, package, and unchanged-output checks.
- [x] Update exactly the listed HLD files.

## Open questions

None. F-158 authors inline charts, the existing drawing model also parses
anchors, and both placements can carry the same backend-neutral group through
the established paginator paths.

## Progress deviations

- Approved on 2026-08-17: name `rpptx` as a dev-only dependency of `rdocx` for
  the exact Word-versus-PowerPoint golden in the existing rdocx test location.
  The production dependency graph remains unchanged. The gate uses pinned
  `pdftoppm 26.01.0`, and the four-file HLD impact remains unchanged.
