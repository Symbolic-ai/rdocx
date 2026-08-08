# F-109, Shape mutation facade

**Status**: completed
**Sprint**: S27
**Size**: L
**Depends on**: F-079

## Problem

`Presentation` exposes only immutable `slide()` and `slides()` handles at
`crates/rpptx/src/lib.rs:625`, while `SlideRef` and `ShapeRef` borrow immutable
shape-tree children at `crates/rpptx/src/lib.rs:1060` and
`crates/rpptx/src/lib.rs:1112`. Callers therefore cannot change a shape's
transform, non-visual name, fill, line, or preset adjustment values through the
facade.

The typed substrate already exists. `CT_ShapeProperties` owns transform, fill,
line, and geometry in schema order at
`crates/oxml-drawing/src/shape_props.rs:102`, but name, group transform, and
preset adjustment storage need narrow mutable accessors in their existing
OOXML modules. Selected `mc:Fallback` content must remain a read-only view so
the facade never silently rewrites an `AlternateContent` choice.

## Spec reference

- `docs/hld/01-glossary.md`, "Units" and "Geometry and text".
- `docs/hld/05-drawingml-model.md`, "Transforms" and "Geometry".
- `docs/hld/06-presentationml-model.md`, "Public read facade" and "The shape
  tree".
- `docs/hld/14-development-backlog.md`, "F-109, Shape mutation facade".

## Approach

Add mutable borrow handles beside the existing read handles and re-export the
shared DrawingML value types needed by the public methods:

```rust
pub use oxml_core::{Angle, Emu};
pub use oxml_drawing::fill::Fill;
pub use oxml_drawing::line::CT_LineProperties;

impl Presentation {
    pub fn slide_mut(&mut self, index: usize) -> Option<SlideMut<'_>>;
}

impl SlideMut<'_> {
    pub fn shape(&self, index: usize) -> Option<ShapeRef<'_>>;
    pub fn shape_mut(&mut self, index: usize) -> Option<ShapeMut<'_>>;
}

impl ShapeMut<'_> {
    pub fn child_mut(&mut self, index: usize) -> Option<ShapeMut<'_>>;
    pub fn set_position(&mut self, left: Emu, top: Emu) -> Result<()>;
    pub fn set_size(&mut self, width: Emu, height: Emu) -> Result<()>;
    pub fn set_rotation(&mut self, rotation: Angle) -> Result<()>;
    pub fn set_name(&mut self, name: &str) -> Result<()>;
    pub fn set_fill(&mut self, fill: Fill) -> Result<()>;
    pub fn set_line(&mut self, line: CT_LineProperties) -> Result<()>;
    pub fn set_adjust_value(&mut self, name: &str, value: f64) -> Result<()>;
}
```

Use a concrete unsupported-mutation error when a selected shape kind does not
own the requested property. Position, size, rotation, and name dispatch to the
existing concrete shape-tree variants. Fill, line, and adjustment mutation are
limited to variants with typed shape properties, and adjustments require
preset geometry. Reject non-finite adjustment values. Indexed access remains
total and returns `Option`.

Add small mutation methods to the existing `rpptx-oxml` and `oxml-drawing`
types for non-visual properties, group transforms, and adjustment list
replacement or insertion. Preserve the raw-child slots around every modelled
field and continue to serialize through the existing schema-ordered writers.
Do not expose mutable access through `AlternateContent`.

## Rejected alternatives

- Return raw `&mut ShapeTreeChild`. That leaks the OOXML storage model and
  bypasses facade invariants.
- Mutate the selected `AlternateContent` fallback. The selected branch is a
  read projection, and rewriting it could invalidate the choice contract.
- Add a mutation trait shared by shape kinds. There is no second implementer
  today, and direct enum dispatch keeps the behavior in one place.
- Add new modules for the handles. The existing facade and OOXML files are the
  smallest locations and repository rules require an explicit ask for a new
  module or file.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `shape_mutation_setters_survive_save_and_reload` | Position, size, rotation, name, fill, line, and adjustment values all round-trip through save and reload |
| round-trip | `shape_mutation_preserves_unmodelled_xml_and_schema_order` | Raw attributes and children survive while typed fields retain valid `xsd:sequence` order |
| integration | `shape_mutation_handles_nested_group_children` | Recursive mutable access changes a nested shape without changing sibling order or ids |
| negative | `alternate_content_fallback_is_not_mutable` | A selected fallback never yields a mutable child and its raw bytes remain unchanged |
| negative | `shape_mutation_indices_and_kinds_are_total` | Invalid indices return `None`, unsupported operations return errors, and neither path panics |
| unit | `preset_adjustment_setter_inserts_and_replaces_named_values` | A finite named value is inserted once, replaced in place, and serialized as a `val` guide |
| round-trip | `shape_name_mutation_escapes_xml_and_preserves_children` | Special characters are escaped and unmodelled non-visual children survive |

The backlog test gate is named explicitly: every setter round-trips through
save and reload.

## HLD impact

- `docs/hld/06-presentationml-model.md`

Replace the stale statement that the facade has no mutation surface with the
borrow-handle and supported-shape behavior that is true after this story.

## Risk routing

- Unit conversion, `Emu`, and `Angle`: read `docs/hld/01-glossary.md`, "Units",
  and `CLAUDE.md`, "Things that are deliberately wrong". Keep the existing
  truncating unit constructors, add exact transform-value assertions, and
  declare the hash harness unchanged.
- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-tolerant input,
  fixed-prefix output, schema-order, and raw-subtree preservation checks.
- Crate dependency graph and new cross-family `use`: read
  `docs/hld/03-architecture.md`. The new edges are from the `rpptx` facade to
  lower-level `oxml-core` and `oxml-drawing`, so no forbidden reverse edge is
  introduced. Run dependency inspection as part of the consolidated gate.

`rpptx` is `publish = false`, so the published-public-API rider does not match.

## Hash harness

Expected to be unchanged. The mutation surface is confined to unpublished
PresentationML crates and does not alter Word rendering output.

## Implementation checklist

- [x] Add `slide_mut`, `SlideMut`, and recursive `ShapeMut` access.
- [x] Re-export the shared unit, fill, and line types used by the facade.
- [x] Implement transform and name setters across supported concrete kinds.
- [x] Implement fill, line, and finite preset-adjustment setters.
- [x] Preserve raw XML and reject mutable `AlternateContent` projection.
- [x] Add the setter gate, negative cases, and schema-order round trips.
- [x] Update the listed HLD file to the current mutable facade contract.

## Open questions

None. The approved scope uses typed shared values, concrete borrow handles,
explicit unsupported-operation errors, and leaves `AlternateContent` read-only.
