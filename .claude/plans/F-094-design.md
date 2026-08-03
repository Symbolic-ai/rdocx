# F-094, Rotation, flips and groups

**Status**: approved
**Sprint**: S23
**Size**: M
**Depends on**: F-093, F-031

## Problem

F-093 places each resolved shape through a translation-only group, but
`ResolvedShape` also carries its own rotation and horizontal or vertical flips
plus the parent-group transform accumulated by the resolver. Ignoring those
fields produces plausible but incorrectly positioned output, especially for
nested `p:grpSp` content.

`oxml-layout::GroupElement` and both backends already support recursive affine
transforms. The missing work is the PowerPoint-specific composition from the
frozen resolver values into that shared representation.

## Spec reference

- `docs/hld/05-drawingml-model.md`, the `xfrm.rs` module contract and
  "Geometry".
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract" and
  "Draw order".
- `docs/hld/08-rendering-spec.md`, "Why Group is the whole design", "The
  recursion hazard", "The PDF backend", and "The rasteriser".
- `docs/hld/14-development-backlog.md`, "F-094, Rotation, flips and groups".

## Approach

Extend the existing shape-lowering helper in
`crates/rpptx-render/src/lib.rs`. Build one local-to-page transform in the exact
DrawingML order: local rotation about the shape centre, centre-based horizontal
and vertical flips, bounds translation, then the already accumulated
`ResolvedShape::group_transform`. In `Transform::then` notation this is
`rotate.then(flip).then(translate).then(parent)`, so the child transform is
applied before the parent transform.

Keep geometry, fill, outline, and later text together inside the existing
`GroupElement`. Identity orientation still uses the same group path so later
content does not need a second code path. Trust the frozen resolver boundary to
have removed invalid extents rather than duplicating upstream validation in the
renderer.

Tests construct independent corner coordinates by scalar trigonometry rather
than reusing the production transform helper. Nested-group tests inspect and
rasterise the integrated group so they cover composition order as well as the
matrix coefficients. All work stays in the existing renderer source and test
module.

## Rejected alternatives

- Pre-transform every path point. That duplicates affine logic for path,
  gradient, image, outline, and later text content.
- Put rotation fields on each positioned-element arm. That does not compose
  recursively and contradicts the shared `GroupElement` design.
- Flatten parent transforms in the renderer. The resolver already owns group
  hierarchy semantics and exposes the accumulated neutral transform.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `rotated_shape_corners_match_hand_computed_coordinates` | The backlog gate checks all four transformed corners against independent trigonometry |
| unit | `horizontal_and_vertical_flips_are_about_the_shape_centre` | Either flip keeps the centre fixed and swaps the expected corners |
| regression | `nested_group_transform_applies_child_before_parent` | Parent and child transforms compose in DrawingML order |
| integration | `rotated_gradient_and_outline_share_the_shape_transform` | Fill axis, path, and outline rotate as one rasterised group |

The backlog test gate is
`rotated_shape_corners_match_hand_computed_coordinates`.

## HLD impact

None. The implementation follows the transform order and group boundary
already documented.

## Risk routing

- Layout and rendering. Read `docs/hld/08-rendering-spec.md`. Run deterministic
  raster checks for rotated gradient and outline content. No baseline may use
  system fonts.

## Hash harness

Expected to be unchanged. PowerPoint transform lowering does not enter the
released Word renderer.

## Implementation checklist

- [ ] Compose rotation, centre flips, translation, and accumulated parent transforms in DrawingML order.
- [ ] Keep all shape paint and geometry under one group transform.
- [ ] Prove corners, flips, nesting order, and rotated paint through focused tests.

## Open questions

None. The resolver and shared `Transform::then` contract determine the order.
