# F-033, Paint and Stroke

**Status**: completed
**Sprint**: S07
**Size**: M
**Depends on**: F-032, F-036

## Problem

The staged layout output moves directly from `Color` to font types at
`crates/oxml-layout/src/output.rs:19`, with no backend-neutral gradient, tile,
or stroke vocabulary. Shapes therefore cannot carry the paint data needed by
the planned PDF and raster backends. The current backlog also omits F-036 even
though tile paint requires `MediaId`.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning".
- `docs/hld/08-rendering-spec.md`, "Extending `PositionedElement`" and "The PDF
  backend".
- `docs/hld/11-migration-plan.md`, "Order of operations".
- `docs/hld/12-testing-strategy.md`, "The hash harness".
- `docs/hld/14-development-backlog.md`, "F-033, Paint and Stroke".

## Approach

Add the explicitly authorized `crates/oxml-layout/src/paint.rs` module and
export these concrete types:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Solid(Color),
    Linear {
        start: Point,
        end: Point,
        stops: Vec<GradientStop>,
        extend: (bool, bool),
    },
    Radial {
        center: Point,
        radius: f64,
        focal: Point,
        stops: Vec<GradientStop>,
        extend: (bool, bool),
    },
    Tile {
        image: MediaId,
        tile: Rect,
        transform: Transform,
    },
}

impl Paint {
    pub fn linear(
        start: Point,
        end: Point,
        stops: Vec<GradientStop>,
        extend: (bool, bool),
    ) -> Self;

    pub fn radial(
        center: Point,
        radius: f64,
        focal: Point,
        stops: Vec<GradientStop>,
        extend: (bool, bool),
    ) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub paint: Paint,
    pub width: f64,
    pub cap: LineCap,
    pub join: LineJoin,
    pub dash: Option<Vec<f64>>,
}

impl Stroke {
    pub fn new(paint: Paint, width: f64) -> Self;
}
```

The gradient constructors degrade exactly one stop to `Paint::Solid`. They
preserve empty and multi-stop vectors unchanged because sorting, clamping, and
duplicate precedence belong to later gradient-building work. `Stroke::new`
preserves paint and width while defaulting to `Butt`, `Miter`, and no dash.

Update the backlog dependency to F-032 and F-036 because `Paint::Tile` directly
uses `MediaId`.

## Rejected alternatives

- Put every paint type in `output.rs`. A cohesive `paint.rs` keeps the output
  element module focused and matches the existing geometry modules.
- Store `u64` in tile paint. That duplicates and weakens F-036's named handle.
- Return `Result<Paint>`. The story specifies degradation rather than fallible
  validation.
- Sort, clamp, or deduplicate gradient stops now. Those policies are not part
  of this story's construction gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `single_stop_gradients_degrade_to_solid_at_construction` | Both gradient constructors return the stop's solid color. |
| unit | `multiple_stop_gradients_preserve_their_geometry_and_stops` | Multi-stop gradients retain their variant and every supplied field. |
| unit | `stroke_new_uses_pdf_defaults` | Cap is Butt, join is Miter, and dash is None. |
| unit | `tile_paint_uses_a_media_id` | Tile paint stores F-036's handle, rectangle, and transform. |

The backlog test gate is that a single-stop gradient degrades to solid at
construction time.

## HLD impact

- `docs/hld/14-development-backlog.md`, add F-036 to F-033's dependency list.

## Risk routing

- Layout model. Use deterministic font mode for the consolidated hash gate and
  require all 28 entries to remain unchanged.
- Crate dependency graph. Run `cargo tree -p oxml-layout --edges normal` and
  reject every `rdocx-*` or `rpptx-*` dependency.
- New module and file. F-033 explicitly authorizes the cohesive `paint.rs`
  module. Add no trait, generic parameter, dependency, or forwarding layer.

The consolidated gate also runs both feature modes and a package dry-run with
the existing sub-10 MiB archive bound. The package must not be published.

## Hash harness

Expected to remain unchanged. The paint model has no released consumer.

## Implementation checklist

- [x] Wait for integrated F-032 and F-036.
- [x] Add and export the exact paint, stop, line cap, line join, and stroke types.
- [x] Implement the single-stop gradient degradation constructors.
- [x] Implement the minimal stroke constructor and defaults.
- [x] Add the focused paint and stroke tests to the existing crate test target.
- [x] Correct F-033's dependency in the backlog HLD.
- [x] Run the scoped checks and consolidated sprint riders.

## Open questions

None.
