# S13 sprint review, pass 1

**Reviewed**: `sprint/s13` at
`87a1e4412c4ac9b04e903adec9c9ec84bd2deb63` against
`e7d95607dc57238937b3195d3fdc26870e736bd3`, 20 files with 4,573 insertions
and 58 deletions, crates: `oxml-drawing`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M7 end-of-milestone gate is:

> every `a:txBody` and `a:spPr` in the deck corpus parses, serialises and
> reparses to a structurally equal value.

S13 does not claim that end-of-milestone gate. F-061 through F-066 remain
pending, and the fetched deck corpus is not present. The approved F-059 plan
uses an inline schema-valid custom geometry fixture while retaining the real
corpus gate at the M7 boundary.

The S13 slice gate holds. Evidence includes
`nested_group_transform_composes_to_the_hand_computed_matrix`,
`hand_written_custom_geometry_guides_produce_expected_path_coordinates`,
`corpus_custom_geometry_round_trips_and_evaluates_to_a_closed_path`, and
`every_fill_form_round_trips_and_gradient_stops_keep_document_order`. The
integrated full gate passed with all 28 deterministic hashes unchanged.

## Not found

- Interaction: transform, guide, custom geometry, and fill models compose
  without conflicting ownership or schema boundaries.
- Duplication: repeated XML helpers remain local and concrete. Moving them
  would add indirection without reducing the cases a reader must consider.
- Layering: `oxml-drawing` depends only on `oxml-core` and `quick-xml` at
  runtime. No `rdocx-*` or `rpptx-*` edge was added.
- Harness: every S13 plan declared no expected delta, and all 28 entries match.
- Gate: every S13 story gate exists and passes on the integrated tree.
- Docs: the only corrected contract is the guide formula set and arc semantics
  in `docs/hld/05-drawingml-model.md`, exactly as listed by F-058.
- Dependencies: no manifest or lockfile changed.
- Surface: the public transform, geometry, and fill types are called for by the
  four approved story contracts. No unrelated public API was added.
