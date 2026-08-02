# F-082, Effective transform and body properties

**Status**: completed
**Sprint**: S20
**Size**: M
**Depends on**: F-081, F-057

## Problem

An ordinary `CT_Shape` retains its required `p:spPr` as private bytes at
`crates/rpptx-oxml/src/shape_tree.rs:172` and
`crates/rpptx-oxml/src/shape_tree.rs:187`, although
`CT_ShapeProperties.transform` is already typed at
`crates/oxml-drawing/src/shape_props.rs:100`. The resolver therefore cannot
inherit a missing slide transform from a layout or master. Body properties are
typed at `crates/oxml-drawing/src/text/body.rs:303`, but there is no
property-level cascade or application of the documented defaults.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Transforms" and "Text".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and
  "Preservation strategy".
- `docs/hld/07-inheritance-and-resolution.md`, "Position and size" and "Body
  properties".
- `docs/hld/14-development-backlog.md`, "F-082, Effective transform and body
  properties".

## Approach

Type the required ordinary-shape `p:spPr` as public
`CT_ShapeProperties`, following the existing picture and connector models.
Parse it with `CT_ShapeProperties::from_xml` and write it as `p:spPr` in its
existing schema position. Preserve unmodelled children through the typed
shape-properties raw-child store. `CT_Shape` becomes `PartialEq` rather than
`Eq`, matching the contained type.

Extend `ResolveCtx` in `context.rs` with:

```rust
pub fn effective_xfrm(&self, shape: &CT_Shape) -> Option<CT_Transform2D>;
pub fn effective_body_pr(&self, shape: &CT_Shape) -> CT_TextBodyProperties;
```

Return an owned transform clone from the first present source in slide,
layout, master order. Build body properties from exact defaults, then overlay
master, layout, and slide values per field so later sources win without
erasing unrelated inherited fields. Defaults are 91,440 EMU left and right,
45,720 EMU top and bottom, top anchor, square wrap, horizontal text, and no
autofit. Returning the existing typed body value keeps the F-082 surface small.
The fully renderer-specific body contract remains owned by F-087.

## Rejected alternatives

- Select the first whole `bodyPr`. OOXML inheritance is property-level and a
  partial slide value must not erase layout or master values.
- Parse `p:spPr` on every resolver call. The ordinary shape should expose the
  same typed model already used by pictures and connectors.
- Convert EMU through floating point. The specified defaults have exact
  integer representations.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `slide_placeholder_without_transform_inherits_layout_position` | The backlog case inherits the layout transform |
| unit | `effective_transform_uses_slide_layout_master_precedence` | Own, layout, master, and absent cases follow the chain |
| unit | `body_properties_merge_per_field_across_the_chain` | Values from all three hierarchy levels survive and later values win |
| unit | `body_property_defaults_use_exact_emu_values` | Every absent property receives the documented exact default |
| round-trip | `ordinary_shape_properties_round_trip_in_schema_order` | Typed `p:spPr` writes in place and preserves unmodelled children |
| corpus | `all_corpus_modelled_parts_reparse_structurally` | The existing required corpus gate remains green after the parser change |

The backlog test gate is named explicitly:
`slide_placeholder_without_transform_inherits_layout_position`.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`

## Risk routing

- Unit conversion. Use exact EMU constants and retain the pinned truncating
  constructor behavior. The 28 deterministic hashes must remain unchanged.
- Any parser or serialiser. Recheck schema order, prefix-tolerant reads,
  fixed-prefix writes, and byte preservation for unmodelled `p:spPr` children.
  Run the required PowerPoint corpus structural round-trip gate.
- Crate dependency graph. Confirm F-081 established only
  `rpptx-layout -> rpptx-oxml` and `rpptx-layout -> oxml-drawing` edges.
- Layout. Any visual baseline would require deterministic font mode. This
  story adds no baseline and declares all hashes unchanged.

## Hash harness

Expected to be unchanged. The PowerPoint resolver is outside the Word hash
surface.

## Implementation checklist

- [x] Type ordinary-shape `p:spPr` without changing schema position.
- [x] Add transform resolution in slide, layout, master order.
- [x] Add per-property body cascade with exact defaults.
- [x] Add focused parser, transform, body, and corpus regressions.
- [x] Run the unit, parser, dependency, and deterministic-hash riders.
- [x] Update the two HLD files during sprint finalisation.

## Open questions

None. The plan uses the existing typed body-properties output. The shared
F-081 crate and file approval remains recorded in F-081.
