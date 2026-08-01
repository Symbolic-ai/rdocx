# F-075, Connectors

**Status**: completed
**Sprint**: S18
**Size**: S
**Depends on**: none

## Problem

The HLD requires a typed `CT_ConnectionShape`, but the shape-tree union still
stores `p:cxnSp` as opaque bytes at `crates/rpptx-oxml/src/shape_tree.rs:22`.
The dispatch at `crates/rpptx-oxml/src/shape_tree.rs:722` therefore cannot
expose the start or end connection used to attach a connector to shapes.

Corpus connectors include free-standing, start-only, and fully connected forms,
including connectors nested in group shapes. Both endpoints must be optional,
while any endpoint that exists must carry its required shape id and connection
site index.

## Spec reference

- `docs/hld/03-architecture.md`, "Crate-level conventions".
- `docs/hld/04-opc-and-packaging.md`, "Preservation discipline".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and
  "Preservation strategy".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-075, Connectors".

## Approach

Add `crates/rpptx-oxml/src/connector.rs` and export concrete schema types:

```rust
pub struct CT_Connection {
    pub id: u32,
    pub idx: u32,
    // private raw attributes
}

pub struct CT_ConnectionShape {
    pub start_connection: Option<CT_Connection>,
    pub end_connection: Option<CT_Connection>,
    pub shape_properties: CT_ShapeProperties,
    // private root, non-visual, style, extension, and raw slot state
}

impl CT_ConnectionShape {
    pub fn from_xml(xml: &[u8]) -> Result<Self>;
    pub fn to_xml(&self) -> Result<Vec<u8>>;
}
```

The root parser enforces required `p:nvCxnSpPr` then required `p:spPr`, followed
by optional `p:style` and `p:extLst`. The non-visual shell enforces
`p:cNvPr`, `p:cNvCxnSpPr`, then `p:nvPr`. Inside `p:cNvCxnSpPr`, optional
`a:stCxn` precedes optional `a:endCxn`. Each endpoint requires unqualified
`id` and `idx` attributes parsed as `u32`, so qualified lookalikes cannot shadow
them.

Reuse `oxml_drawing::shape_props::CT_ShapeProperties` for required `p:spPr`
and its wrapper-aware writer. Unsupported locks, style, extensions, attributes,
and children remain in ordered raw slots at their schema boundary. Reads
resolve namespaces by URI and writes use fixed `p:` and `a:` prefixes.

Replace `ShapeTreeChild::Connector(Vec<u8>)` with
`ShapeTreeChild::Connector(CT_ConnectionShape)` and use it in both root and
recursive group dispatch. F-075 integrates before F-076 so alternate-content
fallback selection can reuse the typed connector arm.

## Rejected alternatives

- Reparse raw connector bytes from endpoint accessors. That duplicates parser
  state and leaves invalid typed states representable.
- Put connector schema details into `shape_tree.rs`. That already large module
  owns tree recursion and dispatch, while the connector is a cohesive concrete
  schema type.
- Model routing, locks, style, and application properties now. The story needs
  connection endpoints, while ordered raw slots retain the rest losslessly.
- Require both endpoints. Valid corpus connectors can be free-standing or have
  only one endpoint.
- Add a connector trait or generic abstraction. No second implementation or
  instantiation exists today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `connector_with_start_and_end_connections_round_trips` | Both endpoint ids and indices remain typed and equal after serialise and reparse |
| unit | `connector_reads_any_prefix_and_writes_fixed_prefixes_in_schema_order` | Namespace aliases parse and root, non-visual, and endpoint children write in schema order |
| unit | `connector_requires_non_visual_and_shape_properties_in_schema_order` | Missing, duplicate, or misordered required children fail, while free-standing and one-ended connectors remain valid |
| regression | `connector_connection_attributes_cannot_be_shadowed_by_qualified_names` | Only unqualified required `id` and `idx` attributes populate an endpoint |
| preservation | `connector_preserves_unknown_children_in_their_schema_slots` | Unsupported root and non-visual content remains byte-identical while endpoints are editable |
| round-trip | `every_corpus_connector_round_trips_structurally` | Root and recursively grouped corpus connectors serialise and reparse to equal typed models with nonzero endpoint coverage |

The test gate is: a corpus connector round-trips.

The existing six-variant shape-tree fixture is updated to use a schema-valid
typed connector while retaining its z-order assertion.

## HLD impact

- `docs/hld/06-presentationml-model.md`, define optional typed start and end
  connections and preservation of unsupported connector content on the already
  specified `CT_ConnectionShape` arm.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Check prefix-tolerant reads,
  fixed-prefix writes, required schema order, recursive groups, and exact
  preservation of unsupported subtrees.
- A new module or file. Read the structural rules in `CLAUDE.md` and obtain
  explicit approval before adding `crates/rpptx-oxml/src/connector.rs`.

The consolidated sprint gate adds `cargo test -p rpptx-oxml` and
`RDOCX_PPTX_CORPUS_REQUIRED=1 cargo test -p rpptx-oxml --test integration
every_corpus_connector_round_trips_structurally`.

## Hash harness

Expected to be unchanged. Connector modelling remains inside the unpublished
PowerPoint development model and does not affect Word rendering.

## Implementation checklist

- [x] Add and export concrete connector and endpoint types.
- [x] Enforce root, non-visual, and endpoint child order.
- [x] Parse optional endpoints with required unqualified id and index attributes.
- [x] Preserve unsupported connector content in its schema slots.
- [x] Replace root and recursive raw connector arms with the typed model.
- [x] Add focused namespace, order, endpoint, and preservation tests.
- [x] Add the required pinned-corpus connector round-trip gate.
- [x] Update the approved HLD impact file.
- [x] Confirm every PowerPoint development crate remains version 0.0.0 and unpublished.
- [x] Confirm all deterministic hashes remain unchanged.

## Open questions

None. The user approved `crates/rpptx-oxml/src/connector.rs`.
