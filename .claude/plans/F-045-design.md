# F-045, Rasteriser groups, paths, gradients, dashes, and background

**Status**: completed
**Sprint**: S10
**Size**: L
**Depends on**: F-040, F-041, F-043

## Problem

`crates/oxml-pdf/src/raster.rs:42` always fills a page white,
`crates/oxml-pdf/src/raster.rs:73` discards line dash patterns, and
`crates/oxml-pdf/src/raster.rs:114` skips every `Path` and `Group`. The staged
tiny-skia backend therefore loses the exact element forms introduced for
PowerPoint, even though `oxml-layout` already carries transforms, clips, solid
and gradient paints, strokes, and page backgrounds.

The current flat loop also applies only the page scale. A local patch for each
new arm would still fail on nested transforms and group clips, so the raster
backend needs one recursive rendering seam shared by old and new elements.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The rasteriser".
- `docs/hld/12-testing-strategy.md`, "oxml-pdf".
- `docs/hld/14-development-backlog.md`, "F-045, Rasteriser: groups, paths,
  gradients, dashes, background".

## Approach

Replace the page's flat match loop with one private recursive renderer that
carries the accumulated `tiny_skia::Transform` and current clip mask. Existing
text, rectangles, lines, and images continue through their current helpers with
the accumulated transform. A group pre-concatenates its local transform,
rasterizes its optional clip path into a mask, intersects that mask with its
parent clip, and renders children in order. Group opacity applies uniformly to
its subtree without introducing a public abstraction.

Translate backend-neutral path commands into one tiny-skia path. Map non-zero
and even-odd fill rules, solid and gradient paint, stroke width, cap, join, and
dash arrays onto tiny-skia. Linear and radial gradients use their declared
points, stops, spread behaviour, and the accumulated transform. Tile paint and
outer-shadow blur remain unsupported because no S10 story owns them.

Fill the page with `PageFrame.background` when it is a supported solid or
gradient paint, falling back to white only when no background is present.
Wire the existing line tuple and path dash vectors through tiny-skia's stroke
dash support with phase zero. Keep all code and tests in the existing
`raster.rs` file and add no dependency, module, feature flag, or public API.

## Rejected alternatives

- Flatten groups through `walk`. A flat leaf walk cannot preserve clip masks,
  group opacity, or child draw order with scoped state.
- Render each group into a separate pixmap unconditionally. That adds memory
  and compositing differences when direct recursive drawing is sufficient.
- Keep special path code separate from clips. One path builder prevents
  geometry differences between visible paths and group masks.
- Add tile paint or shadow blur here. Neither is in the F-045 backlog contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `rotated_rectangle_has_a_filled_interior_and_empty_corner` | At 72 DPI, recursive transform composition puts a path interior and exterior at exact sampled pixels. |
| regression, gate | `dashed_line_contains_deterministic_gaps` | The existing line dash tuple produces painted runs separated by transparent gaps. |
| unit | `nested_group_transforms_apply_child_before_parent` | Three-deep raster geometry follows the layout transform composition contract. |
| regression | `group_clip_masks_children_and_preserves_outside_pixels` | A child is visible inside its declared clip and absent outside it. |
| unit | `path_fill_rule_selects_the_tiny_skia_rule` | Non-zero and even-odd paths produce different expected interior pixels. |
| regression | `linear_and_radial_gradients_sample_expected_colours` | Both gradient kinds interpolate the expected deterministic endpoint and midpoint colours. |
| regression | `page_background_replaces_the_white_default` | A supported page paint covers empty-page pixels while `None` remains white. |
| regression | `path_dash_array_contains_deterministic_gaps` | Backend-neutral stroke dash vectors are honored with phase zero. |

The backlog test gate is a rotated rectangle at 72 DPI with a filled interior
pixel and an empty corner, plus a dashed line with gaps.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- No table row adds an external rider. Run focused `oxml-pdf` tests, the exact
  seven-sample golden comparison, dependency inspection, and the consolidated
  workspace verification required by the normal sprint gate.

## Hash harness

Expected to remain unchanged. The staged raster backend is not a released
sample consumer. Do not update `scripts/hash_baseline.json`.

## Implementation checklist

- [x] Add one private recursive raster element renderer.
- [x] Compose group transforms and intersect clip masks.
- [x] Translate backend-neutral paths and fill rules to tiny-skia.
- [x] Translate solid and gradient fills and strokes.
- [x] Honor existing line and path dash patterns.
- [x] Render supported page backgrounds with a white default.
- [x] Add the rotated rectangle, dashed line, clip, gradient, and background tests.
- [x] Update exactly the declared HLD files to current intent.
- [x] Prove the hash and exact golden baselines remain unchanged.

## Open questions

None. F-045 follows F-043 and reuses the paint semantics already fixed by the
layout model and PDF gradient implementation.
