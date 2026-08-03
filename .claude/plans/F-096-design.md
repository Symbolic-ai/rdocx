# F-096, Pictures with crop and tile

**Status**: completed
**Sprint**: S23
**Size**: M
**Depends on**: F-092, F-072

## Problem

F-072 exposes picture relationships, source crop rectangles, stretch, and tile
data, and F-092 stores deduplicated bytes by `MediaId`. The resolver currently
turns every picture into `ResolvedContent::None`, however, because it has no
source-scoped relationship-to-media input. The frozen image variant also lacks
stretch and tile placement values. Pictures therefore disappear before
`rpptx-render` can lower them.

Crop can use the existing source rectangle, but complete picture rendering
requires one narrow source-neutral amendment to the resolver boundary. The
renderer then expresses crop and tile through existing shared image, group,
and clip primitives without changing released Word element arms.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, the picture crop and intrinsic-size
  row.
- `docs/hld/06-presentationml-model.md`, "The shape tree" and the typed
  picture relationship and crop contract.
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract".
- `docs/hld/08-rendering-spec.md`, "Extending PositionedElement", "Why Group
  is the whole design", "The rasteriser", and "The renderer's input".
- `docs/hld/14-development-backlog.md`, "F-096, Pictures with crop and tile".

## Approach

In `crates/rpptx-layout/src/lib.rs`, add source-scoped media maps that keep
slide, layout, and master relationship identifiers separate and map each to a
`MediaId`. Add a `resolve_slide_with_media` entry point beside the existing
resolver method. It preserves each flattened item's source while converting a
typed `CT_Picture` to owned, source-model-free image content. The existing
method continues to diagnose unresolved image media when no map is supplied.

Extend `ResolvedContent::Image` with a neutral placement enum. Stretch retains
an optional fill rectangle. Tile retains point translation, fractional scale,
horizontal or vertical flip, one of the nine rectangle alignments, optional
declared DPI, and `rotate_with_shape`. Normalize missing translation to zero,
scale to 100 percent, flip to none, alignment to top-left, and
`rotate_with_shape` to true. A missing declared DPI means use embedded image
DPI, falling back to 96 when none is available. Embedded relationships are in
scope. Linked external media stays unsupported with a diagnostic and no
network access.

Extend the existing lowering helpers in `crates/rpptx-render/src/lib.rs`.
Resolve image bytes only through `RenderInput::media` and return a contextual
missing-media error rather than silently dropping a picture.

For an uncropped picture, emit one `PositionedElement::Image` in the shape's
local bounds. For a crop, clamp the four source fractions, reject a crop that
removes the full source width or height, and compute the larger source-image
rectangle whose retained sub-rectangle exactly fills the shape bounds. Wrap
the image in a `GroupElement` clipped to the shape geometry. This uses the
existing image and clip primitives and works in both shared backends without
adding crop fields to `PositionedElement::Image`.

Lower neutral tile placement in `rpptx-render` before the shared backend sees
it. Probe intrinsic dimensions and embedded DPI through `oxml-media`, apply
the normalized translation, scale, flip, alignment, source crop, and rotation
policy, then emit a deterministic row-major set of content-addressed image
elements clipped to the picture geometry. Bound the repeat count from page and
tile extents so malformed inputs cannot allocate without limit. The direct
backend tile-paint limitation remains explicit because the presentation
renderer expresses picture tiles using existing image groups.

Use tiny in-memory PNG fixtures assembled in tests. Decode only the rendered
PNG for sampled-pixel assertions. No binary fixture or new test file is added.

## Rejected alternatives

- Keep the F-087 image variant unchanged. That would continue to lose tile
  placement and source-scoped relationship information before rendering.
- Implement `Paint::Tile` separately in PDF and raster backends. Lowering once
  in the presentation renderer keeps shared backends format-neutral.
- Look up media by relationship ID while rendering. F-092 already resolved
  source-local relationships to deck-wide content-addressed IDs.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cropped_picture_renders_only_its_crop_region` | The backlog gate samples retained source pixels and proves cropped-away regions cannot appear |
| unit | `same_relationship_id_resolves_to_distinct_media_in_each_source_scope` | Slide, layout, and master picture relationships cannot alias |
| unit | `picture_model_resolves_to_neutral_stretch_and_tile_placement` | Raw picture values do not cross the resolver boundary and all defaults are pinned |
| unit | `crop_lowers_to_clipped_source_image_geometry` | Crop fractions produce the expected expanded image rectangle and shape clip |
| integration | `tile_picture_repeats_media_in_row_major_order_inside_shape_clip` | Tiled pixels repeat at the requested scale and never escape the picture path |
| unit | `tile_dpi_prefers_declared_then_embedded_then_96` | Physical tile size follows the pinned DPI precedence |
| regression | `equal_picture_bytes_reuse_one_media_id_across_elements` | Repeated pictures retain F-092 content-addressed identity |
| robustness | `missing_external_media_and_empty_crop_are_contextual` | Invalid media and crop input fail or diagnose without panic, network access, or unbounded work |

The backlog test gate is `cropped_picture_renders_only_its_crop_region`.

## HLD impact

- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`

## Risk routing

- Unit conversion, `Emu`, and points. Read `docs/hld/01-glossary.md` and
  `CLAUDE.md` pinned conversion rules. Convert tile translation with the
  existing truncating boundary and assert exact values. The hash harness must
  remain unchanged.
- Layout and rendering. Read `docs/hld/08-rendering-spec.md`. Generate image
  fixtures in memory, render at 72 DPI, and sample deterministic pixels. Do not
  record a system-font baseline.

## Hash harness

Expected to be unchanged. Picture and tile lowering remains inside the
unpublished PowerPoint renderer.

## Implementation checklist

- [x] Add source-scoped relationship-to-media input and neutral picture placement.
- [x] Resolve embedded picture bytes only by content-addressed `MediaId` and diagnose external links.
- [x] Lower uncropped and cropped pictures through image and clip primitives.
- [x] Lower tile placement, crop, flip, alignment, DPI, and rotation to bounded repeated images.
- [x] Preserve outline order above picture and tile content.
- [x] Prove crop, tile, deduplication, and invalid-input behavior with focused tests.
- [x] Update the HLD to describe presentation-side tile lowering.

## Open questions

Resolved. The user approved the source-neutral media-scope and image-placement
amendment, including the pinned defaults and full modelled tile scope.
