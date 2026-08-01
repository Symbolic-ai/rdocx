# F-071, Placeholders

**Status**: completed
**Sprint**: S17
**Size**: M
**Depends on**: F-070

## Problem

F-070 deliberately leaves `p:sp` opaque in
`crates/rpptx-oxml/src/shape_tree.rs:21`, so the `p:ph` nested under
`p:nvSpPr/p:nvPr` cannot be inspected or matched. The inheritance resolver in
F-081 needs the attribute-presence-sensitive placeholder identity described in
the PresentationML model, while the current HLD signature loses that presence
by declaring `PlaceholderKey.idx` as an unconditional `u32`.

Placeholder matching is not ordinary equality. A shared `idx` takes priority,
otherwise types are compared after defaulting an absent type to body and
applying PowerPoint's title and body equivalence classes.

## Spec reference

- `docs/hld/06-presentationml-model.md`, "The shape tree", "Placeholders",
  and "Preservation strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-071, Placeholders".

## Approach

Add `rpptx-oxml/src/placeholder.rs` and export:

```rust
pub enum PhType {
    Title,
    Body,
    CenteredTitle,
    Subtitle,
    DateTime,
    SlideNumber,
    Footer,
    Header,
    Object,
    Chart,
    Table,
    ClipArt,
    Diagram,
    Media,
    SlideImage,
    Picture,
    VerticalTitle,
    VerticalBody,
    VerticalObject,
    Other(String),
}

pub struct CT_Placeholder {
    pub ph_type: Option<PhType>,
    pub idx: Option<u32>,
}

pub struct PlaceholderKey {
    pub ph_type: PhType,
    pub idx: Option<u32>,
}

impl CT_Placeholder {
    pub fn effective_type(&self) -> PhType;
    pub fn key(&self) -> PlaceholderKey;
}

impl PlaceholderKey {
    pub fn matches(&self, other: &Self) -> bool;
}
```

The concrete placeholder also retains `orient`, `sz`, `hasCustomPrompt`,
unknown attributes, and ordered raw children so `p:ph` round-trips without
discarding producer extensions. If both keys have an `idx`, `matches` compares
only those indices. If either lacks one, it compares effective types. `Title`
and `CenteredTitle` share one class. `Body`, `Subtitle`, and `Object` share a
second class. Other types match exactly. This domain predicate remains a named
method instead of overloading `PartialEq` with non-transitive semantics.

Replace `ShapeTreeChild::Shape(Vec<u8>)` with `Shape(CT_Shape)`. The partial
`CT_Shape` model owns its typed `CT_Placeholder` inside the required
`p:nvSpPr/p:nvPr` path, enforces the required `p:spPr` shell, and preserves all
unrelated non-visual, shape-property, style, text, and extension content in
schema slots. It does not speculatively model shape rendering fields owned by
later stories.

The existing integration test binary gains focused nested-tree coverage and a
corpus traversal that parses every placeholder-bearing shape and compares its
serialise and reparse model.

## Rejected alternatives

- Keep `p:sp` raw and scan `p:ph` ad hoc. That gives F-081 no durable typed
  placeholder boundary and duplicates parsing at every consumer.
- Collapse an absent `idx` to zero. Presence selects the matching algorithm,
  so a sentinel changes behaviour.
- Implement matching through `PartialEq`. The `idx` preference and type
  fallback make this a domain predicate, not structural equality.
- Parse all shape properties, styles, and text in this story. Those completed
  DrawingML models can be connected when a named consumer needs them.
- Put placeholder logic in `shape_tree.rs`. A dedicated module keeps the
  matching contract in one discoverable file and is reused by picture
  placeholders in F-072.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `placeholders_match_by_idx_before_type` | Two present indices control the match even when types agree or disagree |
| unit | `placeholders_match_by_type_when_either_idx_is_absent` | Type matching is used unless both indices are present |
| unit | `absent_placeholder_type_defaults_to_body` | Missing `type` produces the body effective type |
| unit | `title_and_centered_title_are_equivalent_placeholders` | Both title members match each other |
| unit | `body_subtitle_and_object_are_equivalent_placeholders` | All three body members match each other |
| preservation | `placeholder_attributes_and_unknown_children_round_trip_in_place` | Known attributes and opaque extension XML retain their values and slots |
| round-trip | `typed_shape_placeholder_round_trips_inside_nested_groups` | The shape-tree arm becomes typed without changing tree order or recursion |
| round-trip | `every_corpus_shape_placeholder_round_trips_structurally` | Placeholder-bearing shapes from all 50 pinned decks serialise and reparse equally |

The test gate is: matching by idx, by type, absent type defaulting to body, and
both equivalence classes.

## HLD impact

- `docs/hld/06-presentationml-model.md`, make `PlaceholderKey.idx` optional so
  the documented type retains the presence needed by its matching rule.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Test `p:sp`, `p:nvSpPr`, `p:nvPr`,
  and `p:ph` schema order, alternate read prefixes, fixed prefixes for modelled
  output, and byte-for-byte opaque subtree preservation.
- A new module or file. Read the structural rules in `CLAUDE.md` and obtain
  explicit approval before adding `crates/rpptx-oxml/src/placeholder.rs`.

The consolidated sprint gate adds `cargo test -p rpptx-oxml`,
`RDOCX_PPTX_CORPUS_REQUIRED=1 cargo test -p rpptx-oxml --test integration
every_corpus_shape_placeholder_round_trips_structurally`, and
`cargo tree -p rpptx-oxml --edges normal`.

## Hash harness

Expected to be unchanged. The model remains in unpublished `rpptx-oxml` and
does not enter the released Word path.

## Implementation checklist

- [x] Add and export the placeholder types and matching predicate.
- [x] Preserve attribute presence and default absent type to body.
- [x] Implement both equivalence classes and the `idx` priority rule.
- [x] Replace the raw shape arm with the partial typed `CT_Shape` model.
- [x] Preserve unrelated shape content and enforce the required shape shell.
- [x] Add focused matching, prefix, order, preservation, nested-tree, and corpus tests.
- [x] Update the approved HLD impact file with the presence-sensitive key type.
- [x] Confirm every PowerPoint development crate remains version 0.0.0 and unpublished.
- [x] Confirm all 28 deterministic hashes remain unchanged.

## Open questions

None. The user approved the presence-sensitive `Option<u32>` key and the new
placeholder module.
