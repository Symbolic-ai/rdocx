# F-076, mc:AlternateContent

**Status**: approved
**Sprint**: S18
**Size**: M
**Depends on**: F-070

## Problem

The shape-tree union currently stores `mc:AlternateContent` only as raw bytes
at `crates/rpptx-oxml/src/shape_tree.rs:23`. That preserves its alternatives
but gives a renderer no branch to inspect. The story requires fallback
selection without weakening the existing byte-preservation guarantee.

Selection must remain separate from serialisation. Parsing and re-emitting an
alternative would normalise producer XML, while evaluating `mc:Choice` would
require a supported-extension policy that this story and the current renderer
do not define.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Preservation discipline".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and
  "Preservation strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-076, mc:AlternateContent".

## Approach

Keep the model beside recursive shape-tree dispatch in
`crates/rpptx-oxml/src/shape_tree.rs`:

```rust
pub struct CT_AlternateContent {
    raw_xml: Vec<u8>,
    selected_fallback: Option<Vec<ShapeTreeChild>>,
}

impl CT_AlternateContent {
    pub fn raw_xml(&self) -> &[u8];
    pub fn selected_fallback(&self) -> Option<&[ShapeTreeChild]>;
}

pub enum ShapeTreeChild {
    // existing arms
    AlternateContent(Box<CT_AlternateContent>),
}
```

The full captured subtree remains the only serialisation source. The parser
finds at most one immediate child whose expanded name is `mc:Fallback`, using
inherited and local namespace bindings. It parses direct fallback members
through the same private shape-tree dispatch used by `p:spTree` and
`p:grpSp`, retaining their order. Unknown direct content is not exposed as a
render child but remains in `raw_xml`.

Selection is fallback-only. Every `mc:Choice` stays opaque and is not evaluated.
No fallback is accepted and returns `None`, preserving the existing Choice-only
fixture. An empty fallback returns `Some(&[])`. More than one immediate MC
fallback is invalid. A producer element with local name `Fallback` but another
namespace does not select.

F-075 integrates first. Its typed connector arm will then be available through
the shared fallback dispatch without a second connector parser.

## Rejected alternatives

- Add `alternate_content.rs`. The type and shape-tree member parser are
  mutually recursive, so keeping them together makes the dispatch readable in
  one place and avoids a new module.
- Return only raw fallback XML. That does not provide renderable members and
  makes later shape-id scanning parse the same branch again.
- Evaluate `mc:Choice/@Requires`. No extension capability registry or renderer
  exists, and the story explicitly selects the fallback.
- Serialise from the parsed fallback. That would discard or normalise choices
  and violate the byte-identical subtree gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `alternate_content_selects_fallback_without_parsing_choices` | Only ordered typed members from the immediate fallback are exposed and Choice members are ignored |
| unit | `alternate_content_without_fallback_preserves_choices_and_selects_none` | A Choice-only subtree remains valid, exact, and has no selected render members |
| unit | `alternate_content_resolves_branch_namespaces_and_rejects_duplicate_fallbacks` | Namespace aliases work, wrong-namespace local names do not select, and duplicate MC fallbacks fail |
| preservation | `alternate_content_fallback_selection_does_not_change_stored_alternatives` | Attributes, entities, comments, processing instructions, every Choice, and the Fallback remain byte-identical |
| round-trip | `alternate_content_fallback_keeps_shape_tree_order_inside_nested_groups` | Outer z-order and selected fallback member order survive in a recursive group |
| round-trip | `every_corpus_alternate_content_subtree_round_trips_byte_identically` | Every MC AlternateContent subtree found across the pinned corpus remains exact and coverage is nonzero |

The test gate is: a deck with `AlternateContent` round-trips byte-identically in
that subtree.

The corpus currently contains AlternateContent in transition, chart style,
OLE, and picture payloads rather than direct shape-tree children. The corpus
test proves the preservation gate, while code-built fixtures prove typed
shape-fallback selection.

## HLD impact

- `docs/hld/06-presentationml-model.md`, replace the opaque enum arm with the
  raw-plus-selected model and state fallback-only selection, Choice opacity,
  no-fallback behaviour, and raw-only serialisation.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Check alternate namespace prefixes,
  nested shape order, duplicate fallbacks, and complete byte equality including
  choices, comments, and processing instructions.

The consolidated sprint gate adds `cargo test -p rpptx-oxml` and
`RDOCX_PPTX_CORPUS_REQUIRED=1 cargo test -p rpptx-oxml --test integration
every_corpus_alternate_content_subtree_round_trips_byte_identically`.

## Hash harness

Expected to be unchanged. Alternate-content inspection remains inside the
unpublished PowerPoint development model and does not affect Word rendering.

## Implementation checklist

- [ ] Replace the raw alternate-content arm with the raw-plus-selected model.
- [ ] Reuse one shape-tree member parser for ordinary and fallback members.
- [ ] Select only an immediate MC fallback using namespace URI resolution.
- [ ] Preserve every alternative and serialise only the original raw subtree.
- [ ] Add focused fallback, namespace, recursive-order, and preservation tests.
- [ ] Add the required pinned-corpus byte-identity gate.
- [ ] Update the approved HLD impact file.
- [ ] Confirm every PowerPoint development crate remains version 0.0.0 and unpublished.
- [ ] Confirm all deterministic hashes remain unchanged.

## Open questions

None. The user approved ordered typed `ShapeTreeChild` fallback members while
the original raw subtree remains the sole serialisation source.
