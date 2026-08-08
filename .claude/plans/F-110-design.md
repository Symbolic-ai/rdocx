# F-110, Shape constructors

**Status**: completed
**Sprint**: S27
**Size**: M
**Depends on**: F-109

## Problem

F-109 supplies mutable slide and shape handles, but ordinary shapes still have
no construction path. `CT_Shape` has only the placeholder constructor at
`crates/rpptx-oxml/src/shape_tree.rs:494`, while `CT_ConnectionShape` and
`CT_GroupShape` only parse and write at
`crates/rpptx-oxml/src/connector.rs:72` and
`crates/rpptx-oxml/src/shape_tree.rs:1009`. The facade cannot append a textbox,
preset shape, free-standing connector, or empty group.

Every constructor must allocate a tree-wide shape id, emit the required
non-visual and shape-property shells in schema order, and append at the end of
the shape tree for top z-order. A malformed shell makes PowerPoint repair the
deck even when this repository can parse its own output.

## Spec reference

- `docs/hld/01-glossary.md`, "Units" and "Geometry and text".
- `docs/hld/02-scope-and-non-goals.md`, "Presentation and slides".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and "Shape ids".
- `docs/hld/14-development-backlog.md`, "F-110, add_textbox, add_shape,
  add_connector, add_group_shape".

## Approach

Extend the F-109 mutable slide handle with four direct constructors:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorType {
    Straight,
    Elbow,
    Curve,
}

impl SlideMut<'_> {
    pub fn add_textbox(
        &mut self, left: Emu, top: Emu, width: Emu, height: Emu,
    ) -> Result<ShapeMut<'_>>;
    pub fn add_shape(
        &mut self, preset: &str,
        left: Emu, top: Emu, width: Emu, height: Emu,
    ) -> Result<ShapeMut<'_>>;
    pub fn add_connector(
        &mut self, connector: ConnectorType,
        begin_x: Emu, begin_y: Emu, end_x: Emu, end_y: Emu,
    ) -> Result<ShapeMut<'_>>;
    pub fn add_group_shape(&mut self) -> Result<ShapeMut<'_>>;
}
```

Re-scan the complete shape tree immediately before each append with
`ShapeIdAllocator`, allocate once, derive a deterministic name from that id,
and append the new child. Use narrow constructors in the existing
`shape_tree.rs` and `connector.rs` files.

An ordinary preset shape receives canonical non-visual properties, a typed
transform and preset geometry, plus a minimal text body. A textbox uses `rect`,
sets `txBox="1"`, has no fill, and contains `bodyPr`, `lstStyle`, and one
required paragraph. A connector is free-standing. Normalize its transform to
the componentwise minimum endpoint, absolute extents, and horizontal or
vertical flip flags. Map straight, elbow, and curve to `line`,
`bentConnector3`, and `curvedConnector3`. Horizontal and vertical connectors
may have one zero extent. An empty group contains required non-visual and group
property children but no invented bounds or members.

## Rejected alternatives

- Add a 187-variant preset enum. The current story needs one string value and
  no second representation exists today.
- Attach connector endpoints to shape ids. The story asks for connector
  construction, while connection ownership and routing are separate behavior.
- Transfer existing shapes into a new group. That adds ownership, z-order, and
  bounds semantics outside the story.
- Hand-write final slide XML. Narrow typed constructors reuse the existing
  schema-ordered writers and preservation model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `all_shape_constructors_open_in_powerpoint_without_repair` | One generated deck containing all four shapes opens in pinned PowerPoint without repair |
| unit | `ordinary_shape_and_textbox_constructors_emit_canonical_shells` | Required non-visual children, transform, geometry, textbox flag, and nonempty text body are in schema order |
| unit | `connector_constructor_normalizes_every_direction` | All endpoint directions, including zero-width and zero-height lines, produce correct offsets, extents, and flips |
| unit | `empty_group_constructor_has_required_children` | The empty group has `nvGrpSpPr` then `grpSpPr`, no members, and no invented bounds |
| integration | `four_appended_shapes_have_unique_ids_and_reopen` | All kinds append in top z-order, ids are tree-wide unique, validation is empty, and save plus reopen preserves kind and geometry |
| regression | `constructor_names_are_deterministic_from_allocated_ids` | Names are stable, unique, escaped, and survive reload |

The backlog test gate is named explicitly: each produces a shape PowerPoint
opens without repair.

## HLD impact

- `docs/hld/06-presentationml-model.md`

Document the public construction surface, tree-wide id allocation, top z-order
append rule, and canonical shells now emitted by the facade.

## Risk routing

- Unit conversion and `Emu`: read `docs/hld/01-glossary.md`, "Units", and
  `CLAUDE.md`, "Things that are deliberately wrong". Preserve exact EMU values
  and truncating constructors, test connector normalization exactly, and
  declare the hash harness unchanged.
- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add fixed-prefix, schema-order,
  reparse, and unmodelled-subtree preservation checks for every constructor.
- External oracle comparison: read `.claude/skills/differential-testing.md`.
  Pin the PowerPoint oracle to version `16.104.25121423` and record the native
  acceptance result.

`rpptx` and `rpptx-oxml` are unpublished. No new crate, module, file, trait,
generic, feature, or dependency edge is planned.

## Hash harness

Expected to be unchanged. Shape creation is confined to unpublished
PresentationML crates and assets.

## Implementation checklist

- [x] Add the concrete connector enum and four `SlideMut` methods.
- [x] Add narrow ordinary-shape, textbox, connector, and group constructors in
  existing OOXML files.
- [x] Allocate a tree-wide id immediately before each top z-order append.
- [x] Normalize connector transforms without rejecting a zero extent.
- [x] Add structural, direction, allocation, validation, and reopen tests.
- [x] Run and record the pinned native PowerPoint acceptance gate.
- [x] Update the listed HLD file to the current constructor contract.

## Open questions

None. The completed scope creates an empty group, free-standing connectors,
validated string preset names, and deterministic names derived from allocated
ids.
