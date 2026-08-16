# F-X016, correctness, pass 1

**Reviewed**: the uncommitted working tree. `oxml-layout/src/line.rs` for the
per-line reservations, `rdocx-layout/src/block.rs` for the reflow state,
`engine.rs` for populating and gating it, `paginator.rs` for alignment
placement and the reflow itself, plus a one-line field addition in
`rpptx-render`.
**Verdict**: 0 defects, 1 smell, 1 nitpick

## Defects

None outstanding.

### D1 found and fixed during the pass, wrapping missed the sample's own case
`crates/rdocx-layout/src/paginator.rs:512`

The design deliberately limited wrapping to drawings anchored to the current
paragraph or already placed, on the grounds that a later paragraph's position is
unknown. Rendering `sample1.docx` showed that limitation failing on the
contribution's own headline page. Its two arrows sit either side of one
paragraph, but the right-hand arrow is anchored to paragraph 282 while the text
is in paragraph 280, so the left arrow wrapped and the right arrow kept printing
over the text.

A bounded look-ahead now collects wrapping drawings from following blocks, but
only those whose vertical frame is the page or a margin. Those have a position
that does not depend on where their own paragraph lands, so no circularity is
introduced: the look-ahead reads nothing pagination has not already decided.

The plan predicted this limitation and accepted it. The render is what showed
the prediction was too weak to ship. Covered by
`a_drawing_anchored_to_a_later_paragraph_still_pushes_text_aside`, which fails
against the look-ahead reverted.

## Smells

### S1, a paragraph-relative anchor in a later block still does not wrap
`crates/rdocx-layout/src/paginator.rs:512`

The look-ahead deliberately excludes `ST_RelativeFromV::Paragraph` and `Line`.
Those genuinely need their own paragraph placed before their position is known,
and guessing would mean paginating twice.

Much narrower than the limitation D1 removed, and no sample or corpus document
hits it. Recorded rather than fixed, because closing it means a two-pass
paginator, which is a design change rather than a review-time patch. If it turns
out to matter, it wants its own F-ID.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs`, `reflow_around_wraps` decides which
  side a drawing is on by comparing its centre with the text area's centre. A
  drawing straddling the middle picks a side rather than splitting the line into
  two runs. Word does the same for square wrapping, so this matches rather than
  approximates.

## Not found

Checked and produced nothing:

- **correctness**. The reflow runs before the fitting decision, so the height it
  produces is the height that gets measured. `content_offset_top` is added both
  to `content_height` and to the render origin, so the two agree. The reflowed
  paragraph clears its own `reflow` field, so re-entering cannot reserve twice.
  Two passes, not a loop, so termination is structural.
- **panics**. No indexing or slicing on input-derived values. `resize` guards
  every write into the prefix and suffix vectors. `break_into_lines` failing
  returns `None` and the caller keeps the paragraph it already had.
- **ooxml**. No parser or serialiser touched. F-X015 did that.
- **structure**. One new struct in `block.rs` and one in `paginator.rs`, both
  with a single consumer that exists today. `LineBreakParams` gains two fields
  whose empty default reproduces existing behaviour exactly, which is what lets
  every other caller stay unchanged.
- **performance**. The reflow inputs are moved, not cloned:
  `inline_items` is finished with at that point and would otherwise be dropped.
  `Engine::layout` drops the state again unless the document holds a wrapping
  drawing, so a document without one carries nothing.
  `a_document_without_wrapping_carries_no_reflow_state` was dropped from the
  test plan because the field is private to the block module and the behaviour
  is better shown by the harness staying flat.
- **contract**. Matches the plan, with the look-ahead correction recorded in the
  plan itself.
- **tests**. The three golden tests and the look-ahead regression each fail
  against their own reverted change. `a_wrap_none_drawing_leaves_text_untouched`
  passes in both states by design: it is the identity guard that must never
  move, and the harness staying at 28 of 28 is the same claim at document scale.
