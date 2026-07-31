# F-042, Rewrite the three collection passes on walk

**Status**: approved
**Sprint**: S09
**Size**: M
**Depends on**: F-035, F-040

## Problem

The font collector in `crates/oxml-pdf/src/font.rs:32`, image registration in
`crates/oxml-pdf/src/writer.rs:67`, and annotation allocation and writing in
`crates/oxml-pdf/src/writer.rs:97` and `crates/oxml-pdf/src/writer.rs:354`
iterate only `page.elements`. F-040 makes grouped content render recursively,
so each flat collection pass would now fail only for nested content. The
result would be missing grouped fonts, images, or live links.

This is the R3 regression gate. All three passes must use the already completed
`oxml_layout::walk` contract and must agree with recursive content emission on
stable leaf identity.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The recursion hazard".
- `docs/hld/12-testing-strategy.md`, "oxml-pdf".
- `docs/hld/13-risks-and-open-questions.md`, "R3, `Group`-blind collection
  passes".
- `docs/hld/14-development-backlog.md`, "F-042, Rewrite the three collection
  passes on walk".

## Approach

Replace each flat scan with `walk(&page.elements, ...)`, keeping its depth-first
document order. Font collection ignores the accumulated transform but sees
every nested `Text` leaf.

Key image and annotation references by `(page_index, leaf_index)`, where the
leaf index is the depth-first callback ordinal. Thread the same ordinal through
recursive content emission so image resource names and preallocated objects
remain aligned without pointer identity or a second tree.

For link annotations, apply the accumulated transform to all four rectangle
corners through `Transform::transform_rect_bbox` before the existing page
coordinate conversion. Use `walk` for allocation, page `/Annots` assembly, and
annotation dictionary writing so every pass uses the same key and ordering.

## Rejected alternatives

- Recurse separately in all three collectors. That recreates the exact
  divergence risk `walk` was introduced to remove.
- Key nested leaves by address. Pointer identity is unnecessary and obscures
  the document-order contract.
- Store indices inside public layout elements. Collection bookkeeping belongs
  to the PDF backend and needs no model change.
- Leave annotations untransformed. Annotation dictionaries do not inherit the
  content CTM, so nested links would be clickable in the wrong location.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `grouped_text_is_included_in_font_subsetting` | Text nested inside a group registers and uses its font. |
| regression, gate | `grouped_image_registers_and_uses_xobject` | A nested image receives a matching XObject resource and content name. |
| regression, gate | `grouped_link_emits_transformed_annotation` | A nested link is present in `/Annots` with the transformed rectangle. |
| unit | `nested_leaf_ordinals_match_content_order` | Collection and recursive emission use the same depth-first leaf keys. |
| regression | `top_level_collection_output_remains_stable` | Existing top-level fonts, images, and links retain their output behavior. |

The backlog test gate is three nested-target tests, one for font subsetting,
one for XObject registration, and one for transformed link annotations.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- No table row adds an external rider. Run focused `oxml-pdf` tests, exact
  seven-sample golden comparison, dependency inspection, and the consolidated
  workspace verification required by the normal sprint gate.

## Hash harness

Expected to remain unchanged. The staged backend is not a released rendering
consumer. Do not update `scripts/hash_baseline.json`.

## Implementation checklist

- [ ] Wait for integrated F-040 recursive content emission.
- [ ] Rewrite font collection on `walk`.
- [ ] Rewrite image registration on `walk` and depth-first leaf keys.
- [ ] Rewrite every annotation pass on `walk`, the same keys, and transformed
      rectangle bounds.
- [ ] Add all three R3 nested-target regression tests.
- [ ] Update exactly the declared HLD files and close the current R3
      mitigation wording.
- [ ] Prove the hash and exact golden baselines remain unchanged.

## Open questions

None. F-035 and F-040 define the required traversal and recursive rendering
seams.
