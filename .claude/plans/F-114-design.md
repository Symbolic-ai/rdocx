# F-114, remove_slide, move_slide, duplicate_slide

**Status**: approved
**Sprint**: S28
**Size**: M
**Depends on**: F-078, F-107

## Problem

`Presentation` keeps slide records, `p:sldIdLst`, presentation relationships,
slide relationship scopes, package parts, content types, media, notes, and
custom-show references. The facade currently supports append-only slide
creation. Collection edits must keep all of those representations consistent
while preserving unmodelled XML and raw child boundaries.

Deep copy cannot reuse source relationship ids. Relationship ids may occur in
typed or preserved slide content, and notes require a fresh back relationship.
The exact story gate is: a duplicated slide's images resolve to the new slide's
own relationships.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Presentation and slides".
- `docs/hld/04-opc-and-packaging.md`, part naming, media, and integrity.
- `docs/hld/06-presentationml-model.md`, presentation order, preservation,
  relationship remapping, notes, custom shows, and validation.
- `docs/hld/13-risks-and-open-questions.md`, schema order and relationship ids
  in preserved XML.
- `docs/hld/14-development-backlog.md`, "F-114, remove_slide, move_slide,
  duplicate_slide".

## Approach

Add the collection methods to the existing facade:

```rust
pub fn remove_slide(&mut self, index: usize) -> Result<()>;
pub fn move_slide(&mut self, from_index: usize, to_index: usize) -> Result<()>;
pub fn duplicate_slide(&mut self, index: usize) -> Result<SlideRef<'_>>;
```

Indices are zero-based and must already exist. `to_index` is the final index.
Moving to the same index is a no-op. A duplicate is inserted immediately after
its source. It is not automatically enrolled in any custom show.

Move the same record in `self.slides` and `presentation.slide_ids` without
changing ids, relationships, parts, or custom shows. Record original slide
relationship ids inside `CT_Presentation` and serialize the list through
reconciled raw boundaries keyed by relationship id. This keeps comments and
unmodelled children anchored to surviving producer entries after move, remove,
and insertion.

Stage removal before commit. Remove the selected slide id, record,
presentation relationship, slide part, relationship scope, and content-type
override. Remove its notes part and scope when present. Remove only matching
`p:sld` entries from preserved custom-show XML while retaining the containers
and unrelated bytes. Delete media reachable from the removed scopes only when
no remaining internal relationship reaches that part, then rebuild the media
index. Do not clean unrelated pre-existing orphan parts.

Stage duplication as a complete new graph. Allocate a fresh slide part,
producer slide id, and presentation relationship. Clone the source
relationship scope in source order with destination-scope ids, preserving
external target mode and recomputing internal relative targets. Reuse equal
image bytes through the existing package-wide `MediaStore`. Deep-copy notes to
a fresh notes part and point its back relationship at the new slide.

Serialize the typed source slide, rewrite all numeric relationship ids through
the existing `rewrite_rel_ids`, assign fresh shape ids across ordinary,
grouped, and compatibility content, rewrite connector endpoints through the
same shape-id map, then parse the result as the destination slide. Apply the
corresponding relationship and shape-id rewrite to copied notes. Add a narrow
shape-tree behavior in the existing module because slides and notes are two
present consumers. Commit the staged graph only after every allocation and XML
rewrite succeeds.

Use existing error variants. Add no new module, file, trait, generic, feature,
dependency, or owned-slide wrapper.

## Rejected alternatives

- Mutate only the facade vector. That diverges from `p:sldIdLst` and the OPC
  graph.
- Retain removed slide parts or stale custom-show entries. Validation would
  expose the inconsistent graph.
- Reuse source-relative targets or relationship ids in a duplicated scope.
  Both are scoped to the source part.
- Share notes between source and duplicate. One notes back relationship cannot
  identify two source slides.
- Add the duplicate to custom shows. Membership is explicit producer state and
  must not be inferred.
- Parse the full custom-show model. One narrow entry-removal behavior preserves
  more source XML.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| negative | `invalid_slide_collection_indices_do_not_mutate_the_presentation` | Invalid remove, move, and duplicate preserve exact bytes |
| integration | `move_slide_reorders_the_slide_id_list_without_rewriting_relationships` | Facade order and `p:sldIdLst` agree while ids and targets remain stable |
| integration | `remove_slide_removes_its_part_relationship_notes_and_custom_show_entries` | Selected graph members disappear and unrelated custom-show bytes remain |
| regression | `removing_shared_and_last_image_users_prunes_only_new_orphans` | Shared media remains and the final removed user prunes its media |
| integration, gate | `duplicated_slides_images_resolve_to_the_new_slides_own_relationships` | Source and copy use distinct slide scopes that resolve to equal deduplicated bytes |
| round-trip | `duplicate_slide_rewrites_typed_and_preserved_relationship_ids_without_other_byte_changes` | Typed and preserved relationship attributes are mapped and other bytes survive |
| regression | `duplicate_slide_assigns_fresh_shape_ids_and_rewrites_connector_endpoints` | Shape ids are unique and connectors target copied shapes |
| integration | `duplicate_slide_copies_notes_with_a_fresh_back_relationship` | Notes survive and point to the duplicate |
| integration | `remove_move_and_duplicate_round_trip_with_clean_validation` | Combined operations reopen in order with zero validation issues |

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`

Document candidate-only orphan-media pruning, the three facade methods and
their index semantics, synchronized order, fresh ids, relationship remapping,
notes copying, custom-show behavior, and raw-boundary reconciliation.

## Risk routing

- Parser or serialiser: run focused tests for reconciled slide-list raw
  children and typed plus preserved relationship-id rewrites. Assert schema
  order, prefix tolerance, fixed-prefix output, and byte preservation outside
  mapped ids and removed custom-show entries.
- Run the shared corpus identity test with an empty relationship map when
  `RDOCX_PPTX_CORPUS_DIR` is available.

No dependency edge, published API, external oracle, file, module, crate,
trait, generic, or feature rider applies.

## Hash harness

Expected unchanged. This story changes only unpublished PowerPoint package
mutation. All 28 deterministic hashes must match.

## Implementation checklist

- [ ] Add the three atomic facade methods.
- [ ] Reconcile raw slide-list children against original relationship ids.
- [ ] Synchronize facade records and slide ids.
- [ ] Remove slide, notes, relationship, content-type, custom-show, and newly
  orphaned media graph members.
- [ ] Deep-copy slides and notes with fresh ids and remapped relationships.
- [ ] Freshen shape ids and connector endpoints.
- [ ] Add the exact gate and all negative, preservation, and graph tests to
  existing binaries.
- [ ] Update exactly HLD 04 and HLD 06.
- [ ] Run focused checks, risk riders, `/verify --full`, and the hash harness.

## Open questions

None. The approved behavior inserts a duplicate immediately after its source,
deep-copies notes, does not copy custom-show membership, and interprets
`to_index` as the final existing index.
