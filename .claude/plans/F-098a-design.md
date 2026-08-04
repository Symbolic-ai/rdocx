# F-098a, Text content box

**Status**: approved
**Sprint**: S24
**Size**: M
**Depends on**: F-083, F-030

## Problem

Preset and custom geometry already carry an evaluated local text rectangle at
`crates/rpptx-layout/src/lib.rs:99`, and resolved text bodies carry concrete
insets at `crates/rpptx-layout/src/lib.rs:182`. The renderer discards the text
rectangle at `crates/rpptx-render/src/lib.rs:211` and has no shape-local content
box against which later paragraph layout can run.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Custom geometry" and "Text body".
- `docs/hld/07-inheritance-and-resolution.md`, "Body properties".
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/14-development-backlog.md`, "F-098a, Text content box".

## Approach

Add the private S24 text module and a concrete content-box function. It selects
the evaluated geometry text rectangle when present, otherwise the full local
shape bounds, then subtracts the resolved left, top, right, and bottom insets.
Clamp negative width or height to zero while keeping the inset origin, so an
over-inset shape cannot create non-finite layout coordinates.

The helper accepts `&ResolvedShape` and `&ResolvedTextBody` and returns the
existing `oxml_layout::Rect`. It adds no public type, trait, generic, wrapper,
manifest dependency, or renderer behavior beyond the computed box.

## Rejected alternatives

- Use the full shape bounds for every geometry. Non-rectangular presets carry
  a text rectangle specifically to avoid inventing this box.
- Intersect the text rectangle with the painted path. The specification defines
  the rectangle and insets as the text boundary without a path clip.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `preset_text_rectangle_minus_unequal_insets_produces_the_computed_content_box` | The child backlog gate and exact local coordinates |
| unit | `missing_text_rectangle_falls_back_to_local_shape_bounds` | Rectangle and bounds-fallback shapes use width and height from the local shape |
| regression | `insets_larger_than_the_text_rectangle_do_not_create_negative_extents` | Width and height clamp to zero with finite coordinates |

The test gate is a preset text rectangle minus four unequal insets produces the
hand-computed content box.

## HLD impact

None. The content-box algorithm is already specified.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Focused tests use structural coordinates and
  any later raster evidence uses deterministic font mode.
- A new module or file: explicit approval is required for
  `crates/rpptx-render/src/text.rs`. Its first implementer is F-098a and all
  remaining S24 text stories use it immediately.

## Hash harness

Expected to be unchanged. This is an unpublished PowerPoint-only content box.

## Implementation checklist

- [ ] Wire the approved private text module into `rpptx-render`.
- [ ] Compute the geometry text rectangle or local-bounds fallback.
- [ ] Apply four resolved insets and clamp negative extents.
- [ ] Add exact-coordinate and malformed-box regressions.
- [ ] Run focused `rpptx-render` checks.

## Open questions

None. The single private module shared by F-098a through F-101 is approved.
