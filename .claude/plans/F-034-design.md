# F-034, Path and Group arms

**Status**: approved
**Sprint**: S07
**Size**: M
**Depends on**: F-031, F-033

## Problem

The staged output enum ends with `LinkAnnotation` at
`crates/oxml-layout/src/output.rs:120`, so it cannot represent paths, nested
transforms, clipping, opacity, or effects. `PageFrame` has no background at
`crates/oxml-layout/src/output.rs:124`, and `LayoutResult` has no diagnostics at
`crates/oxml-layout/src/output.rs:182`. The HLD specifies the missing fields but
does not define the minimal diagnostic shape, and the backlog's phrase "both
enums" does not name its targets.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Extending `PositionedElement`", "Why
  `Group` is the whole design", and "The recursion hazard".
- `docs/hld/11-migration-plan.md`, "Order of operations" and "Repository and
  link impact".
- `docs/hld/12-testing-strategy.md`, "oxml-layout".
- `docs/hld/14-development-backlog.md`, "F-034, Path and Group arms".

## Approach

Add these concrete output types:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PathElement {
    pub path: Path,
    pub fill: Option<Paint>,
    pub stroke: Option<Stroke>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Effect {
    OuterShadow {
        dx: f64,
        dy: f64,
        blur: f64,
        color: Color,
    },
}

#[derive(Debug, Clone)]
pub struct GroupElement {
    pub transform: Transform,
    pub clip: Option<Path>,
    pub opacity: f64,
    pub effects: Vec<Effect>,
    pub children: Vec<PositionedElement>,
}
```

Document `GroupElement::transform` as mapping child-local coordinates into the
parent coordinate system. Extend `PositionedElement` with exactly `Path` and
`Group`, and mark both `PositionedElement` and `Effect` non-exhaustive.

Add `background: Option<Paint>` to `PageFrame` and
`diagnostics: Vec<Diagnostic>` to `LayoutResult`. Mark both structs
non-exhaustive as required by the rendering specification and add constructors
that preserve the old required fields:

```rust
impl PageFrame {
    pub fn new(
        page_number: usize,
        width: f64,
        height: f64,
        elements: Vec<PositionedElement>,
    ) -> Self;
}

impl LayoutResult {
    pub fn new(
        pages: Vec<PageFrame>,
        fonts: Vec<FontData>,
        metadata: Option<DocumentMetadata>,
        outlines: Vec<OutlineEntry>,
    ) -> Self;
}
```

`PageFrame::new` initializes no background. `LayoutResult::new` initializes an
empty diagnostics list. Export the new public types from the crate root.

Clarify the rendering and backlog HLD so the current intent names the two
non-exhaustive enums, records the minimal diagnostic representation and
constructor signatures, and describes the unpublished staged boundary rather
than the stale 0.3.0 cut.

## Rejected alternatives

- Add rotation, clipping, and paint fields to all five legacy arms. That breaks
  existing construction sites and still cannot represent nested groups.
- Use `Box<GroupElement>` or trait objects. The children vector already
  provides recursive indirection.
- Add builders for path and group elements. Their public fields express the
  complete small contract.
- Add diagnostic severity, codes, or source locations. No current story or
  specification defines them.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `path_and_group_arms_preserve_their_payloads` | Both new variants retain every supplied field. |
| unit | `page_frame_new_defaults_background_to_none` | The constructor preserves legacy fields and initializes the new field. |
| unit | `layout_result_new_defaults_diagnostics_to_empty` | The constructor preserves legacy fields and initializes diagnostics. |
| unit | `group_transform_maps_child_coordinates_into_parent_coordinates` | The documented direction agrees with self-first transform composition. |
| doctest, gate | public constructor examples | External-style staged construction compiles for both non-exhaustive structs. |
| golden, gate | hash harness | All 28 released-output entries remain unchanged. |

The backlog test gate is that the staged `oxml-layout` construction sites
compile, and the hash harness is unchanged.

## HLD impact

- `docs/hld/08-rendering-spec.md`, name the two non-exhaustive enums, define
  the minimal diagnostic shape and constructor signatures, and state the
  unpublished staged boundary.
- `docs/hld/14-development-backlog.md`, replace "both enums" with
  `PositionedElement` and `Effect`, while retaining the non-exhaustive
  `PageFrame` and `LayoutResult` constructor requirement.

## Risk routing

- Layout model. Use deterministic font mode for the consolidated hash gate and
  require all 28 entries to remain unchanged.
- Crate dependency graph. Run `cargo tree -p oxml-layout --edges normal` and
  reject every `rdocx-*` or `rpptx-*` dependency.
- Public staged API. Compile public constructor examples, run a package dry-run,
  inspect the archive, and retain the existing sub-10 MiB bound.

The package must not be published.

## Hash harness

Expected to remain unchanged. The new arms and fields exist only in the
unpublished staged crate.

## Implementation checklist

- [ ] Wait for integrated F-031 and F-033.
- [ ] Add and export the path, diagnostic, effect, and group output types.
- [ ] Add only Path and Group to `PositionedElement`.
- [ ] Document the child-local to parent transform direction.
- [ ] Add page background and layout diagnostics with neutral constructors.
- [ ] Apply non-exhaustive attributes to the two enums and two structs.
- [ ] Add the focused unit tests and public construction examples.
- [ ] Clarify the two HLD sections listed above.
- [ ] Run the scoped checks and consolidated sprint riders.

## Open questions

None. Approved with `Diagnostic { message: String }`, non-exhaustive
`PositionedElement`, `Effect`, `PageFrame`, and `LayoutResult`, and constructors
on the two structs.
