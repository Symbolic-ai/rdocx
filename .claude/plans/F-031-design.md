# F-031, Transform

**Status**: approved
**Sprint**: S06
**Size**: M
**Depends on**: F-029

## Problem

The staged layout output has points and rectangles but no format-neutral affine
transform. Presentation groups, rotation, flips, clipping, and later PDF
emission all require the same 2x3 matrix, and an ambiguous composition order
would place nested content incorrectly without necessarily failing a test.

## Spec reference

- `docs/hld/01-glossary.md`, "Units".
- `docs/hld/08-rendering-spec.md`, "Extending PositionedElement" and "Why
  Group is the whole design".
- `docs/hld/12-testing-strategy.md`, "oxml-layout".
- `docs/hld/14-development-backlog.md`, "F-031, Transform".

## Approach

Add `transform.rs` to `oxml-layout` with the public concrete matrix specified
by the rendering contract:

```rust
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}
```

Provide these operations:

```rust
pub const IDENTITY: Transform;
pub fn rotate_about(degrees: f64, cx: f64, cy: f64) -> Transform;
pub fn then(self, next: Transform) -> Transform;
pub fn apply(self, point: Point) -> Point;
pub fn is_identity(self) -> bool;
pub fn transform_rect_bbox(self, rect: Rect) -> Rect;
```

Document `self.then(next)` as applying `self` first and `next` second.
Implement it with explicit 2x3 multiplication matching the point equations and
the PDF `cm` concatenation order. Rotation accepts degrees because DrawingML's
60000ths are converted before this format-neutral boundary. Rectangle bounds
transform all four corners and return their axis-aligned minimum and maximum
extents.

Use exact identity comparison for the six stored coefficients. This avoids an
implicit tolerance policy in a geometry primitive. Callers that construct a
near-identity matrix still retain that transform rather than silently dropping
it. Export the type from the crate root and add no dependency.

## Rejected alternatives

- Use a backend matrix type. That would make layout depend on PDF or raster
  implementation details and reverse the intended dependency direction.
- Store DrawingML rotation units. The layout boundary operates in points and
  degrees, while schema-unit conversion belongs upstream.
- Add approximate equality to `is_identity`. A hidden epsilon could discard a
  small intentional transform and is not required by the story.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `identity_is_neutral_for_points_and_composition` | Identity preserves points and both composition positions. |
| unit | `rotate_about_keeps_the_pivot_fixed` | Rotation leaves its center unchanged and moves a known point to hand-computed coordinates. |
| regression | `then_matches_the_pdf_cm_composition_order` | A hand-computed translate and scale sequence applies `self` first and `next` second. |
| unit | `transform_rect_bbox_contains_all_four_transformed_corners` | Rotated rectangle bounds equal the minima and maxima of all transformed corners. |
| unit | `is_identity_is_exact` | Only the exact six identity coefficients report true. |

The backlog test gate is that composition order matches the PDF `cm` operator,
verified against a hand-computed matrix.

## HLD impact

None. The rendering and testing specifications already define the matrix shape,
coordinate equations, method set, and composition gate.

## Risk routing

- Unit coordinates and angles. Read the glossary, accept degrees after schema
  conversion, and use hand-computed positive, negative, and fractional cases.
  Do not introduce a unit rounding policy.
- Layout geometry. Run deterministic hash verification and require all 28
  entries to remain unchanged because no released consumer uses the new type.
- Public API in an unpublished staged crate. Add only the planned concrete type
  and six operations, then run package and archive-size checks.
- New module and file. F-031 explicitly authorizes `transform.rs`. Add no trait,
  generic parameter, wrapper, dependency, or speculative geometry surface.

## Hash harness

Expected to remain unchanged. `Transform` has no released construction site in
S06.

## Implementation checklist

- [ ] Add and export the exact six-coefficient `Transform`.
- [ ] Implement rotation about a point and documented composition order.
- [ ] Implement point application, exact identity, and rectangle bounds.
- [ ] Add the hand-computed PDF order gate and focused geometry tests.
- [ ] Run package, public-surface, deterministic-hash, and dependency riders.

## Open questions

None.
