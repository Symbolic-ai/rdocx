# F-073, Graphic frames

**Status**: approved
**Sprint**: S17
**Size**: M
**Depends on**: F-070

## Problem

`ShapeTreeChild::GraphicFrame(Vec<u8>)` at
`crates/rpptx-oxml/src/shape_tree.rs:23` preserves frame XML but cannot expose
the frame transform or classify the payload carried by
`a:graphic/a:graphicData`. The payload kind is selected by
`a:graphicData@uri`, not reliably by its first child. This matters for the four
OLE corpus frames whose `p:oleObj` is wrapped in `mc:AlternateContent`.

The pinned corpus contains 86 graphic frames and covers every required kind:
26 tables, 26 charts, 18 SmartArt diagrams, and 16 OLE payloads. The model must
recognise all four while retaining unsupported content verbatim.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and "Crate-level
  conventions".
- `docs/hld/05-drawingml-model.md`, "Two traps that are silent until
  PowerPoint refuses the file" and "Preservation".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and "Preservation
  strategy".
- `docs/hld/09-charts-spec.md`, "Why a chart needs three parts".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-073, Graphic frames" and "F-074,
  DrawingML tables".

## Approach

After F-074 provides `CT_Table`, add
`rpptx-oxml/src/graphic_frame.rs` and export:

```rust
pub enum GraphicDataPayload {
    Table(Box<CT_Table>),
    Chart(Vec<u8>),
    SmartArt(Vec<u8>),
    Ole(Vec<u8>),
    Other(Vec<u8>),
}

pub struct CT_GraphicData {
    pub uri: String,
    pub payload: GraphicDataPayload,
}

pub struct CT_GraphicFrame {
    pub transform: CT_Transform2D,
    pub graphic_data: CT_GraphicData,
}

impl CT_GraphicFrame {
    pub fn from_xml(xml: &[u8]) -> Result<Self>;
    pub fn to_xml(&self) -> Result<Vec<u8>>;
}
```

Dispatch uses the exact standard table, chart, diagram, and OLE URI values.
The table branch parses the payload through F-074's `CT_Table`. Chart,
SmartArt, OLE, and unknown payload subtrees stay as captured XML bytes. The
concrete structs also retain raw attributes, required non-visual frame
properties, and ordered unsupported children privately.

The parser enforces required `p:nvGraphicFramePr`, `p:xfrm`, and
`a:graphic/a:graphicData` in schema order, followed by optional `p:extLst`.
Add a concrete root-name-aware writer to `CT_Transform2D` so the existing type
writes `p:xfrm` here and keeps `a:xfrm` for DrawingML call sites. Those are two
real roots today and do not require a trait.

Replace `ShapeTreeChild::GraphicFrame(Vec<u8>)` with
`GraphicFrame(Box<CT_GraphicFrame>)` for root and recursive groups. The box
keeps the child union compact now that the frame contains a typed table.

Extend the existing integration test binary to traverse every corpus frame,
assert all four required URI kinds occur, and compare each frame after
serialise and reparse. Opaque payload tests compare the captured subtree bytes
directly, including markup compatibility wrappers.

## Rejected alternatives

- Dispatch on the payload child QName. That fails for OLE payloads wrapped in
  markup compatibility content and is not the format's dispatch contract.
- Keep table bytes opaque. F-074 supplies the typed table model in this sprint,
  and connecting it here fulfils the sprint goal of tables inside the ordered
  shape tree.
- Model charts, SmartArt, or OLE. Those have separate ownership or are
  explicitly preserved opaque in v1.
- Add a graphic-data payload trait. No second behavioural implementation
  exists, and a concrete enum expresses the closed dispatch set directly.
- Duplicate transform parsing in `rpptx-oxml`. The shared concrete transform
  already models the required fields and only needs the correct outer tag.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `graphic_data_uri_dispatch_recognises_table_chart_smartart_and_ole` | Every standard URI selects the correct branch, including OLE wrapped in alternate content |
| unit | `graphic_frame_requires_children_in_schema_order_and_writes_fixed_prefixes` | Required frame children are enforced and output uses fixed `p:` and `a:` prefixes |
| preservation | `graphic_frame_preserves_unknown_payload_and_extension_xml_byte_for_byte` | Chart, SmartArt, OLE, unknown payloads, and unsupported frame content retain bytes and slots |
| unit | `transform_writer_uses_the_requested_root_name` | One transform model writes `a:xfrm` and `p:xfrm` without changing child order |
| round-trip | `every_corpus_graphic_frame_round_trips_structurally` | All 86 corpus frames reparse equally and all four required kinds are observed |

The test gate is: each payload kind is recognised and its unmodelled forms
preserved.

## HLD impact

None.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Test required schema order, alternate
  read prefixes, fixed write prefixes, URI dispatch, and byte-for-byte opaque
  payload preservation.
- Crate dependency graph and a new family `use`. Read
  `docs/hld/03-architecture.md`, keep the existing `rpptx-oxml` to
  `oxml-drawing` direction, and confirm it with
  `cargo tree -p rpptx-oxml --edges normal`.
- A new module or file. Read the structural rules in `CLAUDE.md` and obtain
  explicit approval before adding `crates/rpptx-oxml/src/graphic_frame.rs`.

The consolidated sprint gate adds `cargo test -p oxml-drawing`,
`cargo test -p rpptx-oxml`, and `RDOCX_PPTX_CORPUS_REQUIRED=1 cargo test -p
rpptx-oxml --test integration every_corpus_graphic_frame_round_trips_structurally`.

## Hash harness

Expected to be unchanged. Graphic-frame and table modelling remains inside
unpublished 0.0.0 PowerPoint development crates and does not modify the
released Word path.

## Implementation checklist

- [ ] Add and export typed graphic-frame and graphic-data payload types.
- [ ] Dispatch the four standard kinds by exact `a:graphicData@uri`.
- [ ] Parse F-074 tables and preserve chart, SmartArt, OLE, and unknown payloads.
- [ ] Enforce the frame shell and preserve unsupported content in schema slots.
- [ ] Add the concrete `p:xfrm` writer path to the shared transform type.
- [ ] Replace raw graphic-frame arms at the root and in recursive groups.
- [ ] Add focused dispatch, schema, preservation, and all-corpus tests.
- [ ] Confirm every PowerPoint development crate remains version 0.0.0 and unpublished.
- [ ] Confirm all 28 deterministic hashes remain unchanged.

## Open questions

None. The user approved the new graphic-frame module and the typed table
boundary after F-074, with every other payload retained opaque.
