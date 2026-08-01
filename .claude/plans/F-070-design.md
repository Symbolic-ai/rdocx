# F-070, The shape tree

**Status**: approved
**Sprint**: S16
**Size**: L
**Depends on**: F-063

## Problem

`p:spTree` is the ordered z-axis of every common slide, but there is no type
that enforces its required `p:nvGrpSpPr` and `p:grpSpPr` prefix or preserves its
six child variants in document order. The repair-sensitive structure is defined
at `docs/hld/06-presentationml-model.md:39`.

Detailed placeholder, picture, graphic-frame, and connector content belongs to
later stories. F-070 therefore needs a stable union boundary that preserves
those payloads without speculatively implementing F-071 through F-074.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Transforms", "Shape properties", and
  "Text".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and "Preservation
  strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-070, The shape tree".

## Approach

Add a shape-tree module exporting `CT_ShapeTree`, `CT_GroupShape`, and
`ShapeTreeChild`. `CT_ShapeTree` requires its non-visual group properties and
group properties before children, retains raw attributes, and writes child
variants in their original z-order. `CT_GroupShape` uses the same recursive
child list for nested groups.

The approved payload decision makes `Shape`, `Picture`, `GraphicFrame`,
`Connector`, and `AlternateContent` own their captured XML bytes directly,
while `GroupShape` alone is recursive. That avoids forwarding-only shell
structs and preserves all content until the named later stories model each
variant. `p:grpSpPr` is parsed enough to expose the existing DrawingML group
transform while preserving unsupported children.

F-070 replaces F-069's raw `p:spTree` field with `CT_ShapeTree`. It adds a
code-built nested-group fixture and runs the model over every corpus shape tree
to prove required prefix children, document order, recursion, and byte
preservation.

## Rejected alternatives

- Implement placeholders, pictures, tables, charts, and connectors now. Those
  have named later stories and would make F-070 exceed its contract.
- Add forwarding-only wrapper structs for every opaque child kind. They add
  places to look without reducing invalid states.
- Store the whole tree as one raw blob. That cannot enforce required group
  children or expose recursion and z-order.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `shape_tree_requires_non_visual_and_group_properties_in_order` | Missing or reordered required prefix children are rejected or canonicalised correctly |
| unit | `all_six_child_variants_keep_document_order` | The union retains variant identity, payload bytes, and z-order |
| round-trip | `nested_group_shape_tree_round_trips_with_tree_shape_preserved` | Recursive groups retain structure and their DrawingML transforms |
| round-trip | `every_corpus_shape_tree_round_trips_structurally` | Every corpus tree serialises and reparses with equal structure |

The test gate is: a deck with nested groups round-trips with tree shape
preserved.

## HLD impact

None.

## Risk routing

- Any parser or serialiser. Test required schema order, fixed prefixes,
  alternate read prefixes, recursion, and byte-for-byte opaque child payloads.
- Crate dependency graph and a new family `use`. Keep the dependency from
  `rpptx-oxml` to DrawingML types and confirm it with
  `cargo tree -p rpptx-oxml`.
- A new module or file. Obtain explicit approval for the shape-tree module
  before implementation.

## Hash harness

Expected to be unchanged. Shape-tree parsing is isolated from the Word facade
and renderers.

## Implementation checklist

- [ ] Add the recursive shape-tree and group-shape models.
- [ ] Enforce required non-visual and group-property children in schema order.
- [ ] Represent and preserve all six child variants in document order.
- [ ] Replace F-069's raw tree boundary with the typed tree.
- [ ] Add nested-group and all-corpus round-trip coverage.
- [ ] Run crate, dependency-tree, prose, and hash checks.

## Open questions

None. The user approved raw XML payloads for non-group variants until F-071
through F-074, the shape-tree module, and the shared F-067 corpus decision.
