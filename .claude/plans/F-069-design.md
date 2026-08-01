# F-069, Slide, layout and master parts

**Status**: completed
**Sprint**: S16
**Size**: L
**Depends on**: F-064

## Problem

The required slide, layout, and master parts have no PresentationML models.
Their structural relationships and colour-map roles are specified at
`docs/hld/06-presentationml-model.md:7`, but the current workspace can only
expose their raw OPC bytes.

These roots share `p:cSld` but carry different ordered children. They also
contain unsupported timing, transition, extension, and producer-specific XML
that must remain in the original schema slots instead of being dropped or
normalised into a catch-all tail.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "The package" and "Relationship types".
- `docs/hld/05-drawingml-model.md`, "Colour resolution".
- `docs/hld/06-presentationml-model.md`, "Parts", "The shape tree", and
  "Preservation strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-069, Slide, layout and master
  parts".

## Approach

Add a slide-parts module with `CT_Slide`, `CT_SlideLayout`,
`CT_SlideMaster`, and `CT_CommonSlideData`. Each root has `from_xml(&[u8])` and
`to_xml()`, reads alternate prefixes, captures root attributes, emits fixed
prefixes in its own schema sequence, and stores unsupported children in ordered
raw boundaries.

`CT_CommonSlideData` types its name and optional background while retaining the
`p:spTree` subtree as raw XML until F-070 replaces that one field with the shape
tree model. `p:txStyles` models its three ordered `a:lstStyle` children with
`CT_TextListStyle`. PresentationML `p:clrMap` and `p:clrMapOvr` parse their
twelve scheme-slot mappings into `oxml_drawing::color::ColorMap`, retain raw
attributes and extensions, and serialise the correct master or override form.

The package-facing corpus test discovers slides, layouts, and masters from
their content types and relationships, round-trips each part structurally, and
checks the hard relationship counts without embedding OPC target resolution in
the XML structs.

## Rejected alternatives

- One generic root type parameterised by part kind. The roots have different
  schema sequences and only one concrete instantiation per kind.
- Parse all producer-specific children. The preservation rule says to model
  only what later rendering and editing consume.
- Move `p:clrMap` into `oxml-drawing`. It is a PresentationML element even
  though its value maps to DrawingML theme slots.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `slide_layout_and_master_write_their_own_schema_order` | Each root emits typed and raw children at the correct boundaries |
| unit | `colour_maps_read_any_prefix_and_preserve_extensions` | All twelve slots and raw extension XML survive |
| unit | `common_slide_data_preserves_the_shape_tree_until_f070` | The complete tree remains byte-identical inside the F-069 model |
| round-trip | `every_corpus_slide_layout_and_master_round_trips_structurally` | Every relevant corpus part parses, serialises, and reparses equally |
| integration | `corpus_part_relationship_counts_are_valid` | Slides, layouts, and masters have the required single parent and theme edges |

The test gate is: every corpus slide, layout and master round-trips
structurally.

## HLD impact

None.

## Risk routing

- Theme colour and colour mapping. Keep the spec-correct mapping in the shared
  DrawingML value model and do not alter the released Word tint and shade path.
- Any parser or serialiser. Test fixed write prefixes, per-root schema order,
  and byte-for-byte preservation of unsupported children.
- Crate dependency graph and a new family `use`. Confirm `rpptx-oxml` depends
  on `oxml-drawing`, never the reverse, with `cargo tree -p rpptx-oxml`.
- A new module or file. Obtain explicit approval for the slide-parts module
  before implementation.

## Hash harness

Expected to be unchanged. The new PresentationML models have no path into Word
sample generation.

## Implementation checklist

- [x] Add slide, layout, master, and common-slide-data root models.
- [x] Add typed PresentationML colour-map and colour-map-override forms.
- [x] Add typed master text styles with ordered raw preservation.
- [x] Retain the shape tree as one raw subtree for the F-070 boundary.
- [x] Add focused fixtures and corpus-wide structural tests.
- [x] Validate required relationship counts through the OPC layer.
- [x] Run crate, dependency-tree, prose, and hash checks.

## Open questions

None. The user approved the slide-parts module and the shared F-067 corpus
decision.
