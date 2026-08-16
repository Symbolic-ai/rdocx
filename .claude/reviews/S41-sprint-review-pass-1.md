# S41 sprint review, pass 1

**Reviewed**: `sprint/s41` against `1991e37`, 39 files, 5563 insertions and 347
deletions, of which 16 files and 3106 insertions are product code. Crates:
`oxml-layout`, `oxml-pdf`, `rdocx`, `rdocx-layout`, `rdocx-oxml`,
`rpptx-chart`, `rpptx-render`.
**Verdict**: 0 blocking, 3 should-fix, 2 nice-to-have

The sprint carried six F-IDs, all completed. The two that interact most are
F-X013b, which took height out of a page for the note area, and F-X016, which
changes a paragraph's height by reflowing it. Both findings below sit exactly on
that seam, which is what a sprint review is for: each story's own review passed
clean, and neither could see the other.

## Blocking

None.

## Should-fix

### S1, the fitting decision ignores a paragraph's wrap offset
`crates/rdocx-layout/src/paginator.rs:1291` and
`crates/rdocx-layout/src/paginator.rs:1323`

F-X016 added `content_offset_top`, the vertical space a `wrapTopAndBottom`
drawing pushes a paragraph's content down by. `content_height` counts it and
`render_paragraph_lines` honours it, so the whole-paragraph path is consistent.

Both split paths are not. `count_lines_that_fit_with_notes` is handed
`pager.cursor_y + space_before` and `0.0`, neither of which includes the offset,
while the lines it is counting actually begin at that point plus the offset. The
count therefore overestimates by roughly `content_offset_top / line_height`
lines.

Trigger: a paragraph carrying a top-and-bottom wrapped drawing that also has to
split across a page boundary. The last lines placed run past the bottom margin
by up to the offset.

Not reachable by any current test, corpus document or `sample1.docx`, because
the sample's only top-and-bottom drawing sits on a paragraph that does not
split. A fix must make the start position handed to the counter agree with the
position the lines are rendered at, and should be pinned by a regression that
splits a top-and-bottom wrapped paragraph.

### S2, a split paragraph's ink bottom ignores the same offset
`crates/rdocx-layout/src/paginator.rs:1417`

`render_para_split` sets `ink_bottom` to `cursor_y` plus the heights of the
lines it placed, omitting `content_offset_top`. The full-render path at line
1394 uses `content_height()`, which includes it, so the two disagree.

`ink_bottom` is what F-X013b measures the note area from. Underestimating it
makes the note area larger than the body actually left, so a footnote can be
drawn over the last line of a split paragraph. This is the same class of defect
as F-X013b's own D1, reintroduced at a site F-X016 created.

Trigger: a page holding a footnote and a split paragraph carrying a
top-and-bottom wrapped drawing. Narrow, and not reachable today, but the failure
mode is text over text.

### S3, the spec still claims `oxml-layout` is free of Word concepts
`docs/hld/03-architecture.md:74`

The spec says the crate's "output, font, and line modules are 100 percent
docx-free". F-X013c added `NoteStream { Footnote, Endnote }` and `NoteRef` to
`crates/oxml-layout/src/line.rs:102` and `output.rs`, which are two of the three
modules named. Footnotes and endnotes are WordprocessingML constructs and
PresentationML has neither.

The claim was already imprecise, since `footnote_id: Option<i32>` predates this
sprint. F-X013c turned an untyped integer into a named Word concept and did not
revisit the sentence, so the sprint made the gap larger and left the spec
asserting the opposite.

Either the sentence is narrowed to say what is actually true, or the note
reference moves behind a format-neutral name. The former is the honest and much
smaller change. This is a documentation finding, not a layering one: no
dependency edge was added, and the check below confirms it.

## Nice-to-have

### N1, `NoteRegistry::is_empty` has no caller
`crates/rdocx-layout/src/notes.rs:153`

Public API that no story asked for and nothing uses. It came from the F-X013b
design plan's sketch of the type and survived into the implementation. Delete it
or find it a consumer.

### N2, `NOTE_FONT_SIZE` is public but used only within its own module
`crates/rdocx-layout/src/notes.rs:23`

The other three constants are read by the paginator and need to be public. This
one is not. Narrowing it costs nothing.

## Milestone gate

Quoted from `docs/sprints/CURRENT_SPRINT.md`, "Definition of done for this
sprint", each with its evidence.

1. **"A footnote assembled from several runs renders its segments at strictly
   increasing x, and a page whose body fills the text area leaves the reserved
   footnote area clear."** Holds.
   `a_multi_segment_footnote_does_not_stack_its_segments_at_one_x` and
   `a_page_whose_body_fills_the_text_area_does_not_overlap_its_notes`, both
   proven to fail against their own reverted change.

2. **"The three kashida justification values parse to the justified variant, and
   an unknown justification string is still rejected."** Holds.
   `kashida_justification_maps_to_both` and
   `an_unknown_justification_is_still_rejected`. The stronger claim, that such a
   document opens at all, is covered by
   `a_document_using_kashida_justification_still_opens`.

3. **"Wrap mode, the four text distances and both alignment axes round-trip
   through `CT_Anchor` and reach `AnchoredDrawing`, with the hash harness
   unchanged."** Holds.
   `an_anchor_round_trips_its_wrap_distances_and_alignments` covers the round
   trip. "Reach `AnchoredDrawing`" is evidenced end to end rather than directly:
   `text_wraps_beside_a_left_aligned_square_drawing` cannot pass unless the wrap
   mode, the distance and the alignment all arrive in the layout model. Harness
   28 of 28.

4. **"Body text flows around a `wrapSquare` drawing and clears a
   `wrapTopAndBottom` one, while every baseline without a wrapped drawing stays
   byte-identical."** Holds. The three golden tests, plus
   `a_wrap_none_drawing_leaves_text_untouched` and the harness at 28 of 28. The
   corpus does contain floating drawings and they all use `wrapNone`, so the
   flat harness is evidence here rather than a blind spot.

5. **"Every harness delta in the sprint is stated and justified in the commit
   that causes it. No delta is folded into an unrelated change."** Holds, and
   the interesting case is F-X015. Its first implementation moved
   `report:word/document.xml` by writing four zero-valued attributes. The delta
   was caught before commit, the cause identified, the serialiser changed to
   omit a zero distance, and both the mistake and the fix are recorded in the
   commit message and in `.claude/plans/F-X015-design.md`. No commit in the
   sprint carries an undeclared delta, because no commit carries a delta at all.

**Verified end to end** against `sample1.docx`, the document the external
contribution used for its own screenshots. Page 5's footnote is legible and
clear of the body, page 7's text flows between both arrows with neither
overprinting, and a ninth page carries the endnote. Pages 1 to 6 and 8 are byte
identical to their pre-sprint renders except page 5 and 7, each changed by the
story that claims it.

## Not found

Aspects checked that produced nothing:

- **layering**. No `Cargo.toml` in the workspace changed, so no crate gained a
  dependency of any kind, and no `oxml-*` crate gained an edge to `rdocx-*` or
  `rpptx-*`. The `oxml-layout` concern in S3 is about a spec sentence, not a
  dependency.
- **deps**. No new dependency, workspace or otherwise.
- **duplication**. The sprint's one real risk here was note drawing, which
  F-X013b wrote for the page foot and F-X013c needed again for the document end.
  F-X013c extracted `draw_note` rather than copying it, which is the correct
  outcome given that two diverging placement routines are what caused F-X013a's
  original defect.
- **harness**. Covered under the gate above. All six AS_BUILT entries record
  "unchanged, 28 of 28", and all six agree with their commit messages. Three of
  them additionally explain that a flat harness proves nothing for notes,
  because no corpus document has one, which is an honest qualification rather
  than a claim of coverage.
- **surface**, beyond N1 and N2. Every other public item added traces to a story
  that needed it. `oxml-layout`'s breaking rename of `footnote_id` to `note` is
  declared in F-X013c's plan and AS_BUILT, the crate is incubating at 0.2.0 with
  no consumer outside this workspace, and `rdocx`'s own public API is unchanged.
- **interaction**, beyond S1 and S2. The reflow runs before the fitting
  decision, so notes are claimed from the reflowed lines rather than the
  original ones, which is the ordering that keeps F-X016 and F-X013b consistent
  in the whole-paragraph path.
