# F-072, Pictures

**Status**: approved
**Sprint**: S17
**Size**: M
**Depends on**: F-060

## Problem

`ShapeTreeChild::Picture(Vec<u8>)` at
`crates/rpptx-oxml/src/shape_tree.rs:22` preserves a picture but exposes none of
its relationship, crop, shape-property, or placeholder information. The shared
DrawingML fill model already understands `a:srcRect`, yet its `BlipFill`
parser and writer are private and its writer fixes the outer tag to
`a:blipFill` instead of the `p:blipFill` wrapper required by `p:pic`.

The pinned corpus contains 240 pictures, including 198 crops, 98 pictures in
nested groups, 18 picture placeholders, and both embedded and linked images.
One valid picture carries its blip-fill choice inside opaque
`mc:AlternateContent`, so the direct typed fill must remain optional.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Media".
- `docs/hld/05-drawingml-model.md`, "What already exists", "Modules", "Two
  traps that are silent until PowerPoint refuses the file", and
  "Preservation".
- `docs/hld/06-presentationml-model.md`, "The shape tree", "Preservation
  strategy", and "Relationship remapping".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-072, Pictures".

## Approach

After F-071 provides the shared placeholder type, add
`rpptx-oxml/src/picture.rs` and export:

```rust
pub struct CT_Picture {
    pub placeholder: Option<CT_Placeholder>,
    pub blip_fill: Option<BlipFill>,
    pub shape_properties: CT_ShapeProperties,
}

impl CT_Picture {
    pub fn from_xml(xml: &[u8]) -> Result<Self>;
    pub fn to_xml(&self) -> Result<Vec<u8>>;
}
```

The concrete type also retains the required `p:nvPicPr`, root attributes,
optional `p:style`, extensions, and all unsupported children privately at
their schema boundaries. It requires `p:nvPicPr` first and `p:spPr` after the
blip-fill choice. A direct `p:blipFill` becomes typed. An
`mc:AlternateContent` blip-fill choice remains byte-for-byte opaque for F-076,
so `blip_fill` is optional.

Expose `BlipFill::from_xml` and a concrete `write_xml_as` method that can write
the existing `a:blipFill` or the PresentationML `p:blipFill` outer tag without
duplicating its children. Retain unknown root attributes that the current fill
model drops. Add the equivalent concrete root-name writer to
`CT_ShapeProperties` so its existing DrawingML children can be written under
`p:spPr`. These methods have two concrete roots today and introduce no trait or
generic abstraction.

Replace `ShapeTreeChild::Picture(Vec<u8>)` with `Picture(CT_Picture)` for root
and recursive groups. Relationship attributes are accepted only from the
office-document relationships namespace, so an unrelated `x:embed` or
`x:link` cannot shadow the real `r:` value.

Extend the existing integration test binary to traverse all root and nested
picture arms in the corpus, compare serialise and reparse structure, and assert
coverage for cropped, embedded, linked, nested, placeholder, and alternate
content cases.

## Rejected alternatives

- Leave pictures as raw XML. That does not expose the crop rectangle required
  by the story.
- Duplicate blip-fill parsing in `rpptx-oxml`. That creates two models for the
  same crop and relationship contract.
- Require a direct `p:blipFill`. A real corpus picture uses the markup
  compatibility choice instead.
- Parse the markup compatibility fallback now. F-076 owns fallback selection.
- Resolve relationship targets or media bytes here. OPC and later facade
  layers own package resolution, while this story models XML identifiers.
- Add convenience crop or relationship accessors. The public typed fields
  already expose the required values.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `cropped_picture_round_trips_with_relationship_and_source_rectangle` | Embedded relationship id and all four source-rectangle crop edges survive serialise and reparse |
| unit | `picture_reads_any_prefix_and_writes_fixed_prefixes_in_schema_order` | Alternate input prefixes work and output orders non-visual properties, blip fill, shape properties, style, and extensions |
| unit | `picture_requires_non_visual_and_shape_properties_in_schema_order` | Missing or misplaced required picture children are rejected |
| preservation | `picture_preserves_unknown_children_and_alternate_blip_choice_verbatim` | Unsupported picture content and the markup compatibility choice retain bytes and slots |
| regression | `qualified_picture_relationship_attributes_are_not_shadowed` | Only relationship-namespace `embed` and `link` attributes populate the typed fields |
| round-trip | `every_corpus_picture_round_trips_structurally` | All 240 corpus pictures, including crops and nested pictures, serialise and reparse equally |

The test gate is: a cropped picture round-trips with its crop rectangle.

## HLD impact

None.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Test schema order, alternate read
  prefixes, fixed write prefixes, namespace-qualified relationship attributes,
  prefix conflicts, and byte-for-byte opaque subtree preservation.
- Crate dependency graph and a new family `use`. Read
  `docs/hld/03-architecture.md`, keep the existing `rpptx-oxml` to
  `oxml-drawing` direction, and confirm it with
  `cargo tree -p rpptx-oxml --edges normal`.
- A new module or file. Read the structural rules in `CLAUDE.md` and obtain
  explicit approval before adding `crates/rpptx-oxml/src/picture.rs`.

The consolidated sprint gate adds `cargo test -p oxml-drawing`,
`cargo test -p rpptx-oxml`, and `RDOCX_PPTX_CORPUS_REQUIRED=1 cargo test -p
rpptx-oxml --test integration every_corpus_picture_round_trips_structurally`.

## Hash harness

Expected to be unchanged. The work is confined to unpublished 0.0.0
PowerPoint development crates and does not modify the released Word path.

## Implementation checklist

- [ ] Add and export the typed picture model.
- [ ] Reuse F-071 placeholder data inside non-visual picture properties.
- [ ] Expose root-name-aware concrete writers for blip fill and shape properties.
- [ ] Retain blip-fill root attributes and namespace-qualified relationship ids.
- [ ] Replace raw picture arms at the root and in recursive groups.
- [ ] Preserve markup compatibility and unsupported picture content verbatim.
- [ ] Add focused schema, crop, relationship, preservation, and corpus tests.
- [ ] Confirm every PowerPoint development crate remains version 0.0.0 and unpublished.
- [ ] Confirm all 28 deterministic hashes remain unchanged.

## Open questions

None. The user approved the new picture module and the F-071 to F-072 execution
edge for shared picture-placeholder data.
