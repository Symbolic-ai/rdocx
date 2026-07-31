# F-032, Path and PathCommand

**Status**: completed
**Sprint**: S07
**Size**: M
**Depends on**: F-029

## Problem

The staged layout crate exposes points, rectangles, and transforms, but it has
no backend-neutral path representation. `crates/oxml-layout/src/output.rs:3`
defines the geometry primitives and `crates/oxml-layout/src/lib.rs:3` exports
the current modules. Without path commands, later shape and clip work would
have to depend on backend geometry types or duplicate geometry construction.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Extending `PositionedElement`", "The PDF
  backend", and "The rasteriser".
- `docs/hld/12-testing-strategy.md`, "oxml-layout" and "The hash harness".
- `docs/hld/14-development-backlog.md`, "F-032, Path and PathCommand".

## Approach

Add the explicitly authorized `crates/oxml-layout/src/path.rs` module and
export these concrete types:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    CurveTo { c1: Point, c2: Point, to: Point },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub commands: Vec<PathCommand>,
    pub fill_rule: FillRule,
}

impl Path {
    pub fn bounds(&self) -> Option<Rect>;
    pub fn rect(rect: Rect) -> Path;
    pub fn round_rect(rect: Rect, radius: f64) -> Path;
    pub fn ellipse(rect: Rect) -> Path;
}
```

`bounds` returns the conservative axis-aligned bounds of every move point, line
endpoint, cubic endpoint, and cubic control point. It returns `None` when no
point-bearing command exists. It does not solve cubic extrema.

`rect` emits a closed nonzero path. `ellipse` uses four cubic segments with the
standard kappa approximation. `round_rect` uses one circular radius, clamps
negative values to zero, and clamps the upper value to half the shorter side.

## Rejected alternatives

- Use a PDF or tiny-skia path type. That would reverse the format-neutral
  dependency boundary.
- Compute exact cubic extrema. The story explicitly calls for conservative
  control-point bounds.
- Add separate horizontal and vertical corner radii. The current contract names
  one rounded-rectangle constructor and no asymmetric radius model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `empty_path_has_no_bounds` | Empty and close-only paths return `None`. |
| unit | `bounds_include_cubic_control_points` | Bounds include both cubic controls and the endpoint. |
| unit | `rect_constructor_emits_a_closed_nonzero_path` | Rectangle command order, fill rule, and closure are stable. |
| unit | `round_rect_clamps_the_radius_to_half_the_shorter_side` | Oversized and negative radii are normalized to valid circular corners. |
| unit | `round_rect_with_zero_radius_matches_rect` | Zero radius has rectangle geometry. |
| unit, gate | `ellipse_path_bounds_contain_the_ellipse_and_lie_within_its_control_hull` | The ellipse bounds meet the backlog gate. |
| unit | `bounds_do_not_depend_on_the_fill_rule` | Both fill rules produce identical bounds. |

The backlog test gate is that an ellipse path's bounds contain the ellipse and
lie within its control hull.

## HLD impact

None. The rendering specification already defines the command variants, fill
rules, and conservative bounds contract.

## Risk routing

- Layout geometry. Use deterministic font mode for the consolidated hash gate
  and require all 28 entries to remain unchanged.
- New module and file. F-032 explicitly authorizes the cohesive `path.rs`
  module. Add no trait, generic parameter, dependency, or forwarding layer.

The consolidated sprint gate also runs
`cargo test -p oxml-layout --no-default-features`,
`cargo tree -p oxml-layout --edges normal`, and a package dry-run with the
existing sub-10 MiB archive bound. The package must not be published.

## Hash harness

Expected to remain unchanged. The path model has no released consumer.

## Implementation checklist

- [x] Add and export the exact path types and command variants.
- [x] Implement conservative point and control-point bounds.
- [x] Implement closed rectangle and rounded-rectangle constructors.
- [x] Implement the four-cubic ellipse constructor.
- [x] Add the focused geometry tests to the existing crate test target.
- [x] Run the scoped checks and the consolidated sprint riders.

## Open questions

None. Approved with one circular `radius: f64`, clamped as described.
