# F-X019, Paragraph-relative drawings in later blocks should wrap

**Status**: approved
**Sprint**: S43
**Size**: M
**Depends on**: F-X016

## Problem

`Pager::lookahead_wraps` at `crates/rdocx-layout/src/paginator.rs:515-541` lets
a paragraph flow around a wrapping drawing that is anchored to a **later**
paragraph, which is what Word documents do routinely when the arrow beside a
paragraph is anchored to the paragraph after it. It does that only for drawings
whose vertical anchor is the page or a margin:

```rust
let absolute = para.anchored.iter().filter(|a| {
    a.wrap != WrapType::None
        && !matches!(a.rel_v, ST_RelativeFromV::Paragraph | ST_RelativeFromV::Line)
});
```

The exclusion is honest rather than lazy, and the comment at lines 505-514 says
why: a paragraph-relative anchor has no vertical position until its own
paragraph has been placed, and its own paragraph cannot be placed until the
earlier text that would flow around it has been laid out. The look-ahead
therefore has nothing to look at, and text runs straight through where the
drawing will land.

`resolve_anchor_v` at `paginator.rs:1138` shows the same thing from the other
side: for `Paragraph` and `Line` it measures from `para_top`, and the
look-ahead has no `para_top` to give it, so it passes `0.0` for the absolute
cases where the argument is ignored.

## Spec reference

- `docs/hld/03-architecture.md`, the `rdocx-layout` paragraph beginning "The
  paginator also reflows a paragraph around any floating drawing that wraps".
  That paragraph states the reflow rule and the "only for a document that
  actually holds a drawing whose wrap is not `none`" scoping this story extends
  to a second pass.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", for the `regression`
  category the gate uses.
- `docs/hld/14-development-backlog.md`, "F-X019, Paragraph-relative drawings in
  later blocks should wrap".

## Approach

Paginate the section twice, and only when the document contains the case that
needs it. Pass one learns where each paragraph-relative wrapping drawing lands.
Pass two lays the section out again with those positions known, so the
look-ahead can offer them exactly as it already offers page-relative ones.

The drawing is identified by where it is anchored, which is stable across the
two passes because both passes walk the same block list:

```rust
/// Where the first pass placed a wrapping drawing whose vertical anchor is its
/// own paragraph, keyed by block index and the drawing's index within it.
type ResolvedWraps = HashMap<(usize, usize), (usize, PlacedWrap)>;
//                                             ^ the page number it landed on
```

Three edits carry it:

1. `Pager` gains `resolved_in: &'a ResolvedWraps` and
   `resolved_out: ResolvedWraps`. `place_anchored` takes the owning
   `block_idx`, and records into `resolved_out` for each drawing it places whose
   `wrap != WrapType::None` and whose `rel_v` is `Paragraph` or `Line`, together
   with `self.page_number`. Both call sites at `paginator.rs:1387` and
   `paginator.rs:1410` already have `block_idx` in scope.

2. `lookahead_wraps` keeps its existing absolute branch untouched and adds a
   second one: for each later block's paragraph-relative wrapping drawing, look
   `(block_idx, anchor_idx)` up in `resolved_in` and include its rect when the
   recorded page number equals `self.page_number`. On pass one the map is empty
   and every lookup misses, so pass one is byte-identical to today.

3. `paginate_with_media` becomes the two-pass driver:

   ```rust
   fn paginate_with_media(...) -> (Vec<PageFrame>, Vec<OutlineEntry>) {
       let first = paginate_pass(blocks, .., &ResolvedWraps::new());
       if !has_paragraph_relative_wrap(blocks) {
           return (first.pages, first.outlines);
       }
       let second = paginate_pass(blocks, .., &first.resolved);
       (second.pages, second.outlines)
   }
   ```

   `has_paragraph_relative_wrap` is the same predicate the look-ahead filters
   on, over every block. A document without such a drawing, which is every
   sample and every corpus document today, runs `paginate_pass` once and
   produces exactly what it produces now.

`paginate_sections` calls `paginate_with_media` per section, so the second pass
is per section and page numbering, outlines and endnote pages are untouched.

**Two passes, not a fixed point.** Pass two reflows earlier text, which can move
the drawing's own paragraph relative to where pass one put it, so the rect the
look-ahead offered can be slightly stale. Iterating to a fixed point is not
guaranteed to terminate: growing a paragraph can push a drawing to the next
page, which shrinks the paragraph, which pulls it back. Word bounds the same
problem the same way. The limit is stated in the HLD paragraph this story
updates rather than left for a reader to discover.

## Rejected alternatives

- **Predict the paragraph's top in a single pass, from the running height the
  look-ahead already accumulates.** Cheaper, and wrong in the case the story is
  about: the accumulator is computed before the current paragraph reflows, and
  the reflow is what changes the current paragraph's height, so the prediction
  is stale exactly when the feature does something. It also cannot see a page
  break the later block will take.
- **Iterate until placements stop moving, with a pass cap.** No termination
  argument, and a cap turns non-convergence into an arbitrary answer that
  depends on the cap. Two passes give one answer, always.
- **Always run both passes.** Doubles pagination for every document to serve a
  case no sample or corpus document has. The predicate is three lines.
- **Resolve paragraph-relative anchors during layout, before pagination.** Their
  position depends on where the paragraph lands on a page, which is pagination's
  output, not layout's.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_paragraph_relative_wrapping_drawing_pushes_earlier_text_aside` | An earlier paragraph's lines are narrowed by a wrapping drawing anchored to a later paragraph with `rel_v` of `Paragraph`, and the drawing is placed once, at the same rect the earlier text flowed around |
| regression | `a_page_relative_drawing_in_a_later_block_still_wraps` | F-X016's case is untouched by the second pass. This document has no paragraph-relative wrap, so the predicate is false and it paginates once |
| regression | `a_second_pass_is_stable_for_the_document_that_earns_it` | Two layouts of the same document produce identical page elements. Two passes are not a fixed point, so the guarantee is that the answer is the same answer every time |
| unit | `the_lookahead_offers_a_resolved_rect_only_on_its_own_page` | A resolved drawing recorded on page two is not offered to a paragraph being laid out on page one |
| unit | `pass_one_ignores_paragraph_relative_anchors` | With an empty resolved map the look-ahead returns exactly the absolute set it returns today |
| unit | `the_two_pass_predicate_matches_only_paragraph_relative_wraps` | The predicate deciding the pass count is true for a paragraph or line frame that wraps, and false for a page frame, a plain paragraph, and a paragraph-relative drawing whose wrap is `none` |
| unit | `a_placed_paragraph_relative_wrap_is_recorded_for_the_next_pass` | A pass records the paragraph-relative drawing it placed, with its page, and records the page-relative one not at all |

**Test gate**, from the backlog: the first regression, plus the single-pass half
of the same gate, which is the second row.

## HLD impact

- `docs/hld/03-architecture.md`. The paragraph on reflowing around floating
  drawings gains the two-pass rule: a document holding a wrapping drawing
  anchored to its own paragraph paginates twice, the first pass resolving
  positions and the second flowing around them, and the stated limit that two
  passes are not iterated to a fixed point.

## Risk routing

Matched row: **Layout, pagination, line breaking, text shaping**.

- Read `docs/hld/08-rendering-spec.md` before editing.
- Deterministic font mode for every baseline. The regressions construct their
  documents in code and assert on line widths and placed rects, so no new
  recorded baseline is created.
- Re-record deliberately, never incidentally. This story expects no delta, so
  any harness movement is a defect.

No other row matches. `ResolvedWraps` is a type alias over `HashMap`, not a new
trait, generic or module, and it lives in `paginator.rs` beside its only user.

## Hash harness

**Expected unchanged.** No sample defines an anchored drawing with a
paragraph-relative vertical anchor and a wrap other than `none`, so
`has_paragraph_relative_wrap` is false for all seven and each paginates in one
pass through code that is unchanged. Any delta means pass one stopped matching
today's single pass.

## Implementation checklist

- [x] Record the pre-change harness state, 49 of 49
- [x] `ResolvedWraps` and the two `Pager` fields
- [x] `place_anchored` taking `block_idx` and recording resolved rects
- [x] `lookahead_wraps` second branch, page-scoped by the recorded page number
- [x] `has_paragraph_relative_wrap` and the two-pass driver in
      `paginate_with_media`
- [x] `PassContext`, so the two passes differ in one argument rather than eight,
      per microscope S1
- [x] The tests, added to the existing modules rather than a new binary
- [x] Update `03-architecture.md`, including the stated two-pass limit
- [x] `cargo test -p rdocx-layout`, `/microscope F-X019 --working`, `/verify`

## Open questions

None material. The two-pass shape is the one the backlog entry names, and the
convergence limit is recorded rather than engineered around.
