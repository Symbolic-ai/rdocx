# F-107, add_slide

**Status**: completed
**Sprint**: S26
**Size**: L
**Depends on**: F-105, F-106

## Problem

`Presentation` owns only parsed slides and the presentation root at
`crates/rpptx/src/lib.rs:57`. It does not resolve layout records for mutation,
allocate slide parts and ids, synthesize placeholder shapes, or update the
three package structures required for a new slide. A naive layout deep copy
would retain relationship ids, transforms, and latent footer placeholders that
must not appear on the slide.

## Spec reference

- `docs/hld/01-glossary.md`, "The placeholder inheritance triangle".
- `docs/hld/02-scope-and-non-goals.md`, "Presentation and slides".
- `docs/hld/04-opc-and-packaging.md`, "Part naming".
- `docs/hld/06-presentationml-model.md`, "Placeholders" and "Adding a slide".
- `docs/hld/13-risks-and-open-questions.md`, the slide synthesis decision.
- `docs/hld/14-development-backlog.md`, "F-107, add_slide".

## Approach

Extend the facade's owned model with the layout records reachable from the
presentation's masters. Keep them as private concrete records beside the slide
records. Expose direct zero-based layout lookup rather than adding a forwarding
handle type, and add the mutation API:

```rust
impl Presentation {
    pub fn layout_count(&self) -> usize;
    pub fn layout_name(&self, index: usize) -> Option<&str>;
    pub fn add_slide(&mut self, layout_index: usize) -> Result<SlideRef<'_>>;
}
```

The index is stable for the lifetime of the loaded presentation because this
story does not add layout mutation. It also avoids an owned wrapper with no
behavior and avoids a self-borrow conflict between a layout reference and a
mutable presentation.

Implement the nine specified steps in order. Resolve the chosen layout,
allocate `slideN.xml` after the greatest existing positive suffix, build a
minimal `CT_Slide` shell, synthesize only non-latent layout placeholders, copy
their `type` and `idx` verbatim, and give every text body at least one
paragraph. Use `ShapeIdAllocator` for fresh `p:cNvPr` ids. Create exactly one
relative slide-to-layout relationship, add the content-type override, add the
presentation-to-slide relationship, and append a unique slide id using
`max(existing).max(255) + 1`. Return the new borrowed slide handle.

Add only the narrow constructors needed for the slide shell, shape tree, and
minimal placeholder shape to the existing `rpptx-oxml` files. Their writers
continue to enforce schema order. No raw layout shape is copied and no
relationship rewrite is needed.

## Rejected alternatives

- Accept `SlideLayoutRef<'_>` from the same presentation. Its immutable borrow
  conflicts with the mutable receiver and the wrapper would only forward.
- Accept a layout part-name string. That leaks OPC naming through the facade
  and makes cross-presentation misuse easy.
- Deep-copy the layout shape tree. It duplicates latent placeholders and
  imports stale transforms, relationship ids, and creation ids.
- Allocate slide names from part count. Sparse packages can already contain
  the candidate name.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `three_added_slides_have_unique_ids_and_reopen` | Three calls add ids at least 256, all ids and part names are unique, and serialized bytes reopen |
| regression | `add_slide_allocates_after_the_highest_existing_part_suffix` | A sparse package does not overwrite an existing slide part |
| unit | `add_slide_synthesizes_only_non_latent_layout_placeholders` | `dt`, `ftr`, and `sldNum` are absent while title, body, and object placeholders retain `type` and `idx` |
| round-trip | `synthesized_slide_uses_schema_order_and_one_relative_layout_relationship` | The new root reparses, has one relative layout target, and preserves required child order |
| regression | `synthesized_text_bodies_always_contain_a_paragraph` | Every cloned text placeholder satisfies DrawingML `minOccurs=1` |
| negative | `add_slide_rejects_an_unknown_layout_index_without_mutation` | An invalid index returns a contextual error and leaves package bytes unchanged |
| acceptance | `three_added_slides_open_in_powerpoint_without_repair` | The backlog deck opens natively without repair and contains three slides |

The backlog test gate is named explicitly: a deck with three added slides opens
without repair, and every `p:sldId/@id` is at least 256 and unique.

## HLD impact

None. The existing HLD already specifies the public capability, synthesis
choice, placeholder rules, nine-step package mutation, and acceptance gate.

## Risk routing

- Parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add fixed-prefix, schema-order,
  relationship-target, reparse, and raw-subtree preservation checks.
- Public API of an unpublished crate: read `docs/hld/10-bindings-spec.md` and
  the structural rules in `CLAUDE.md`. State that there is no released semver
  impact and keep the selector concrete with no wrapper type.

## Hash harness

Expected to be unchanged. Slide creation is confined to unpublished
PresentationML crates and assets.

## Implementation checklist

- [x] Resolve presentation layouts into private facade records.
- [x] Add direct layout count and name lookup plus index-based `add_slide`.
- [x] Add narrow OOXML constructors for the synthesized slide shell.
- [x] Implement the nine-step package mutation in specification order.
- [x] Preserve placeholder type and idx while excluding latent placeholders.
- [x] Add structural, sparse-allocation, negative, and native acceptance tests.

## Open questions

None. The index-based selector is the smallest concrete Rust API and does not
preclude later Python binding objects from resolving names to indices.
