# F-093, Shape geometry, fills and lines

**Status**: approved
**Sprint**: S23
**Size**: L
**Depends on**: F-091, F-092

## Problem

`crates/rpptx-render/src/lib.rs` currently stops at package assembly and the
owned `RenderInput` boundary. It does not implement the
`layout_presentation` or `layout_slide` functions specified by the renderer
contract. As a result, the concrete geometry, paint, and stroke values already
produced in `crates/rpptx-layout/src/context.rs` never become
`oxml_layout::PositionedElement::Path` values and cannot reach either backend.

F-091 now supplies evaluated preset and custom paths. F-092 supplies the
resolved slide and content-addressed media boundary. This story must lower the
shape-only subset without rendering text early and without changing the
released Word output path.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Geometry" and the module ownership for
  fills and lines.
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap",
  "Extending PositionedElement", "The PDF backend", "The rasteriser", and
  "Performance".
- `docs/hld/14-development-backlog.md`, "F-093, Shape geometry, fills and
  lines".

## Approach

Implement the two existing HLD entry points in
`crates/rpptx-render/src/lib.rs`:

```rust
pub fn layout_presentation(input: &RenderInput) -> Result<LayoutResult, RenderInputError>;
pub fn layout_slide(input: &RenderInput, index: usize) -> Result<PageFrame, RenderInputError>;
```

Extend `RenderInputError` with an index error that reports both the requested
index and slide count. `layout_presentation` lowers slides in order, carries
metadata and resolver diagnostics, and leaves font shaping for F-098. It does
not inspect raw PresentationML or DrawingML types.

Lower each `ResolvedShape` into backend-neutral local paths. Rectangle and
bounds fallback geometry use a local rectangle from `(0, 0)` to the resolved
width and height. Evaluated custom geometry keeps its existing local path
coordinates. A translation-only `GroupElement` places the local geometry at
the resolved bounds and gives F-094 one stable location to extend with
rotation, flips, and accumulated group transforms. Copy the resolved fill and
stroke onto every emitted geometry path. When a bounds fallback has neither
paint, synthesize the documented visible fallback as a deterministic 1 point
black outline so unsupported content cannot disappear. Preserve draw order and
skip text and table content until their owning stories.

Add `oxml-pdf` and `tiny-skia` as test-only dependencies of `rpptx-render`.
Focused tests render at 72 DPI in deterministic font mode, decode the PNG, and
sample pixels away from antialiased edges. Keep the tests in the existing
`lib.rs` test module so no integration test binary or new source file is added.

## Rejected alternatives

- Emit `FilledRect` for common shapes. That cannot express custom paths,
  gradients, outlines, or the F-094 transform seam.
- Translate every path coordinate into page space. That would make F-094
  rewrite geometry and gradient coordinates instead of extending one group
  transform.
- Render text bodies as empty glyph runs. Shape text belongs to F-098 and an
  empty placeholder would make the S23 gate vacuous.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `solid_gradient_and_outlined_shapes_rasterise_at_sampled_pixels` | The backlog gate samples correct interior fill, gradient endpoints, outline, and white exterior pixels |
| unit | `preset_and_custom_geometry_lower_to_ordered_paths` | Rectangle, custom, and evaluated preset geometry preserve path order, fill, and stroke |
| regression | `bounds_fallback_emits_a_visible_black_outline` | Unsupported geometry with no paint remains visible instead of disappearing |
| unit | `layout_slide_rejects_an_out_of_range_index` | Invalid slide access returns contextual error data |
| integration | `layout_presentation_preserves_page_order_and_diagnostics` | Page numbers, sizes, metadata, and resolver diagnostics cross the renderer boundary |

The backlog test gate is
`solid_gradient_and_outlined_shapes_rasterise_at_sampled_pixels`.

## HLD impact

None. The implementation follows the existing page-frame and path lowering
contract.

## Risk routing

- Layout and rendering. Read `docs/hld/08-rendering-spec.md`. Raster evidence
  uses deterministic font mode and a generated in-memory fixture at 72 DPI.
  Do not record a system-font baseline.
- Crate dependency graph and a new use across families. Read
  `docs/hld/03-architecture.md`. Add only test edges from `rpptx-render` to
  `oxml-pdf` and `tiny-skia`, then inspect
  `cargo tree -p rpptx-render --edges normal` and
  `cargo tree -p rpptx-render --edges dev` for the one-way dependency rule.

## Hash harness

Expected to be unchanged. The unpublished PowerPoint renderer is not connected
to any released Word rendering path.

## Implementation checklist

- [ ] Add presentation and single-slide layout entry points with contextual index errors.
- [ ] Lower resolved rectangle, custom, and visibly outlined fallback geometry to path elements.
- [ ] Carry solid and gradient fills plus stroke width, cap, join, and dash.
- [ ] Preserve page order, page size, metadata, and resolver diagnostics.
- [ ] Prove solid, gradient, and outline pixels through deterministic raster output.

## Open questions

Resolved. The user approved a deterministic 1 point black outline for an
otherwise unpainted bounds fallback. This matches the existing neutral stroke
defaults and avoids adding a renderer option for one fallback policy.
