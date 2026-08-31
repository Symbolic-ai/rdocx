# F-219, SmartArt typed model

**Status**: approved
**Sprint**: S62
**Size**: L
**Depends on**: none

## Problem

`GraphicDataPayload::SmartArt` in
`crates/rpptx-oxml/src/graphic_frame.rs` stores only the raw `dgm:relIds`
element. The related diagram data, layout, quick style, colour, and drawing
parts remain opaque package entries. Callers cannot inspect or edit supported
nodes, and relationship remapping can see only the data-model id rather than
the complete owned diagram graph.

The tracked corpus contains several diagram layout families and producer
extensions. A typed projection must cover the bounded fields needed for model
editing and F-220 rendering while retaining every unsupported algorithm,
attribute, child, and part byte-for-byte across unrelated mutation.

## Spec reference

- ECMA-376 Part 1, DrawingML diagram data, layout, style, colour, and drawing
  schemas.
- ECMA-376 Part 2, OPC internal relationships and relationship target
  resolution.
- `docs/hld/02-scope-and-non-goals.md`, the SmartArt scope table.
- `docs/hld/03-architecture.md`, "The dependency rule", typed XML, package
  facade, resolver, and renderer seams.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  and "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Shape tree", "Preservation
  strategy", "Relationship remapping", and "Validation".
- `docs/hld/10-bindings-spec.md`, published native Rust API policy.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-219, SmartArt typed model".

## Approach

Add `crates/rpptx-oxml/src/diagram.rs` for the five diagram part roots and
their shared concrete values. Parsing is namespace-aware and prefix-tolerant.
Writing uses the fixed `dgm`, `a`, and `r` prefixes, schema child order, and
ordered raw-child capture at every typed boundary. The original raw bytes of a
part remain the serialization source until a supported field in that part is
mutated.

Expose the bounded model needed by the current corpus and F-220:

```rust
pub enum DiagramPointKind {
    Node,
    Assistant,
    Presentation,
    Other(String),
}

pub enum DiagramConnectionKind {
    ParentOf,
    PresentationOf,
    PresentationParentOf,
    Other(String),
}

pub enum DiagramLayoutFamily {
    List,
    Hierarchy,
    Cycle,
    Relationship,
    Matrix,
    Pyramid,
    Unsupported(String),
}

pub struct DiagramPoint {
    pub model_id: String,
    pub kind: DiagramPointKind,
    pub text: Option<CT_TextBody>,
}

pub struct DiagramConnection {
    pub model_id: String,
    pub source_id: String,
    pub destination_id: String,
    pub kind: DiagramConnectionKind,
    pub source_order: u32,
    pub destination_order: u32,
}

pub struct DiagramRelationshipIds {
    pub data: String,
    pub layout: String,
    pub style: String,
    pub colors: String,
    pub drawing: Option<String>,
}

pub struct CT_DiagramData { /* typed points, connections, background, raw */ }
pub struct CT_DiagramLayoutDefinition { /* identity, algorithms, constraints, raw */ }
pub struct CT_DiagramStyleDefinition { /* style labels and shape styles, raw */ }
pub struct CT_DiagramColorsDefinition { /* colour labels and choices, raw */ }
pub struct CT_DiagramDrawing { /* cached shape tree and raw */ }
```

`CT_GraphicData` replaces its raw SmartArt variant with a boxed typed
`DiagramRelationshipIds` that retains its exact raw subtree. The data model
types expose checked setters for supported node text and point order. They do
not expose arbitrary raw XML replacement.

The `rpptx` package assembly resolves every diagram relationship only in the
producing slide, layout, or master scope and loads a concrete resource set:

```rust
pub enum DiagramPart<T> {
    Parsed(T),
    External(String),
    MissingTarget(String),
    Invalid(String),
}

pub struct SmartArtInfo {
    pub slide_index: usize,
    pub shape_id: u32,
    pub relationships: DiagramRelationshipIds,
    pub data: DiagramPart<CT_DiagramData>,
    pub layout: DiagramPart<CT_DiagramLayoutDefinition>,
    pub style: DiagramPart<CT_DiagramStyleDefinition>,
    pub colors: DiagramPart<CT_DiagramColorsDefinition>,
}

impl Presentation {
    pub fn smart_art(&self, slide_index: usize) -> Result<Vec<SmartArtInfo>>;
    pub fn set_smart_art_node_text(
        &mut self,
        slide_index: usize,
        shape_id: u32,
        model_id: &str,
        text: &str,
    ) -> Result<()>;
}
```

The generic `DiagramPart<T>` has five concrete instantiations today, one for
each diagram part family, so it reduces repeated resource-state handling. Do
not add a trait or dynamic dispatch. The facade setter clones the package,
updates only the named data-model point text through DrawingML text types,
serializes and reparses that part, validates the complete relationship graph,
and commits only on success.

Unsupported algorithms remain `Unsupported` and their raw layout part remains
the sole serialization source. Unsupported point and connection kinds retain
their lexical value and raw siblings. Unrelated mutations do not parse and
rewrite diagram parts. Slide duplication and transfer remap all five retained
diagram relationship ids and copy or deduplicate their complete owned graph
without cross-scope aliasing.

Add the standard diagram relationship constants to `oxml-opc`. Add no new
dependency, feature, trait, crate, integration binary, or binary fixture. This
is additive native Rust API for the pre-1.0 `rpptx-oxml`, `oxml-opc`, and
`rpptx` crates. No Python, WASM, or CLI surface is added.

## Rejected alternatives

- Parsing only diagram data would leave layout, style, colours, and ownership
  opaque to the renderer and mutation validator.
- Rewriting every diagram part after an unrelated edit would lose producer
  prefixes, extensions, and unsupported algorithms.
- Putting the model in `rpptx-layout` would reverse the XML model and resolver
  dependency boundary.
- A trait for diagram parts would have no distinct behaviour implementers.
- Treating cached drawing shapes as the editable source would make node text
  diverge from the authoritative data model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `smartart_parts_read_aliases_write_schema_order_and_preserve_raw_children` | All five part roots accept alias prefixes, use fixed prefixes and child order after typed mutation, reparse structurally, and retain unmodelled subtrees byte-exactly. |
| round-trip | `supported_smartart_nodes_remain_editable_after_save_and_reopen` | Node identity, kind, text, connections, source order, layout family, style, colours, and relationship ownership survive a targeted text edit. |
| regression | `unsupported_smartart_algorithms_and_parts_remain_byte_preserved_after_unrelated_mutation` | An ordinary shape edit leaves every unsupported diagram part and relationship byte-identical. |
| integration | `smartart_relationships_resolve_only_in_their_producing_scope` | Equal relationship ids on slide, layout, and master do not alias, and missing or external targets remain typed failures. |
| regression | `duplicated_smartart_remaps_the_complete_diagram_relationship_graph` | Data, layout, style, colour, drawing, image, and nested diagram relationships resolve to the copied or deduplicated targets. |
| regression | `failed_smartart_node_mutation_leaves_the_package_unchanged` | Missing shape, point, target, invalid XML, and graph-validation failures are atomic. |

The exact backlog **test gate is round-trip**: "Supported nodes remain
editable and unsupported diagram parts remain byte-preserved after unrelated
mutations."

Use the tracked SmartArt corpus for read and preservation coverage. Construct
prefix, schema-order, malformed-target, duplicate-id, and targeted-mutation
fixtures in the existing `crates/rpptx-oxml/tests/integration.rs` and
`crates/rpptx/tests/integration.rs` binaries. Do not add a binary fixture or
integration binary.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add namespace-alias, fixed-prefix,
  schema-order, structural-reparse, and byte-exact unmodelled-subtree checks
  for every typed part root.
- Crate dependency graph and cross-family uses: keep diagram XML in
  `rpptx-oxml`, relationship constants in `oxml-opc`, and package assembly in
  `rpptx`. Run `cargo tree -p rpptx -e normal` and the shared-crate
  dependency-direction test.
- Public API of a published crate: state the additive pre-1.0 impact. Run
  publish dry runs for `rpptx-oxml`, `oxml-opc`, and `rpptx`, then assert every
  archive remains below 10 MiB.
- New module or file: explicit approval is required for
  `crates/rpptx-oxml/src/diagram.rs`. The five related schemas share ordered
  raw preservation and would make `graphic_frame.rs` harder to understand if
  inlined there.

## Hash harness

Expected unchanged, 49 of 49. Typed projection and opt-in node mutation do not
change ordinary saved or rendered samples. Any delta is unexplained and blocks
integration.

## Implementation checklist

- [ ] Add the approved diagram model module and relationship constants.
- [ ] Parse and write relationship ids and all five diagram part roots.
- [ ] Type bounded point, connection, text, algorithm, style, and colour data.
- [ ] Preserve unsupported algorithms, lexical values, attributes, and child
  subtrees through raw capture.
- [ ] Resolve complete diagram resources in their producing relationship scope.
- [ ] Add inspection and atomic supported-node text mutation to the facade.
- [ ] Extend slide duplication and transfer to the complete diagram graph.
- [ ] Add corpus and source-built round-trip and preservation tests to existing
  targets.
- [ ] Run focused `oxml-opc`, `rpptx-oxml`, and `rpptx` checks plus every rider.

## Open questions

None. The diagram model module and bounded node-text and point-order mutation
surface are approved. Every other field remains inspectable and raw-preserved.
