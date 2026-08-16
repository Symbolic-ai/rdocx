# F-X013a, Footnote line advance

**Status**: completed
**Sprint**: S41
**Size**: S
**Depends on**: none

## Problem

`render_page_footnotes` in `crates/rdocx-layout/src/engine.rs` draws every
segment of a footnote line at the same horizontal position. The origin is
`geometry.margin_left + indent` at `engine.rs:412`, computed once outside the
loop over `line.items` and never advanced. A footnote whose line holds one
segment renders correctly by accident. A footnote whose line holds several,
which is what any note carrying mixed formatting or a hyperlink produces,
renders every segment stacked on the same x and is unreadable.

The body text path does not have this defect. `render_paragraph_lines` in
`crates/rdocx-layout/src/paginator.rs` advances an `x` cursor by `seg.width` for
each item it places. The footnote path was written separately and never gained
the advance.

This is the contained half of the footnote work carried over from the external
PR 2 contribution. That contribution fixed the same line with the same
mechanism.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put", for the `rdocx-layout`
  ownership boundary that puts note rendering in the flow engine.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the regression
  category, and "The hash harness" for the labelled-delta rule.
- `docs/hld/14-development-backlog.md`, "F-X013a, Footnote line advance".

## Approach

Hoist a mutable `x` cursor to the top of the per-line loop in
`render_page_footnotes`, initialised to `geometry.margin_left + indent`, and
advance it by `seg.width` after each segment is pushed. This mirrors
`render_paragraph_lines` exactly.

```rust
for line in &pb.lines {
    let line_baseline = cursor_y + line.ascent;
    let mut x = geometry.margin_left + indent;
    for item in &line.items {
        if let LineItem::Text(seg) | LineItem::Marker(seg) = item {
            // ... origin: Point { x, y: line_baseline - seg.baseline_offset }
            x += seg.width;
        }
    }
    cursor_y += line.height;
}
```

`seg.width` is the width line breaking already computed and is the same
quantity `render_paragraph_lines` advances by, so no new measurement or shaping
is introduced.

**Corrected after review.** This section originally claimed that items which are
neither `Text` nor `Marker` do not advance the cursor in the body path either.
That is false. `paginator.rs:946` and `paginator.rs:967` advance for `Tab` and
`Image`. Microscope pass 1 caught it as D1. The implementation matches on all
four `LineItem` variants and advances for each, drawing only the two that carry
glyphs.

**Extended after review.** Notes were also being line-broken at
`geometry.content_width()` while drawn at `margin_left + 12.0`, so every note
line overran the right margin by exactly the indent. The stacking defect hid it.
A single `FOOTNOTE_INDENT` constant now feeds both, so they cannot disagree. The
reasoning for taking this into the story rather than deferring it is recorded in
`.claude/reviews/F-X013a-correctness-pass-2.md`.

The story still does not touch the marker position, the separator, or where the
note area sits on the page. Those belong to F-X013b.

## Rejected alternatives

- **Re-shape the whole line as one string.** Would discard per-segment
  formatting, which is the thing that produces several segments in the first
  place.
- **Justify or otherwise re-break footnote lines here.** Out of scope. Line
  breaking already ran for these paragraphs and this story only places what it
  produced.
- **Fix the note indent at the same time.** It is a separate defect with a
  separate baseline consequence. Folding two deltas into one commit is what the
  harness rule exists to prevent.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_multi_segment_footnote_does_not_stack_its_segments_at_one_x` | A note whose paragraph holds three differently formatted runs places its segments at strictly increasing x, and each x equals the previous x plus the previous segment width |
| regression | `a_single_segment_footnote_keeps_its_original_position` | A note with one segment renders at `margin_left + indent`, unchanged from before the fix |
| unit | `footnote_segment_advance_matches_body_segment_advance` | The x positions a footnote line produces match those the body path produces for the same segment widths and starting offset |
| regression | `a_tab_inside_a_footnote_still_advances_the_text_after_it` | Added after review D1. The same note laid out with and without a tab between its runs places the trailing run further right when the tab is present |
| regression | `a_long_footnote_does_not_overrun_the_right_margin` | Added with the break-width fix. No note glyph reaches past the right margin, and the note is asserted to have wrapped so the test cannot pass vacuously |

**Test gate**, from the backlog: the two regression tests, each named as a
sentence describing the failure it prevents.

New tests join the existing integration entrypoint in `crates/rdocx-layout`
rather than adding a file under `tests/`, per the build note in `CLAUDE.md`.

## HLD impact

None. The story fixes a defect in rendering a construct the spec set already
describes. It adds no surface and changes no documented behaviour.

## Risk routing

Matched row: **Layout, pagination, line breaking, text shaping**.

- Read `docs/hld/08-rendering-spec.md` before editing. Done. Its text covers
  slide text boxes rather than Word flow notes, so it constrains nothing here,
  but the deterministic font rule below is what the row actually adds.
- Deterministic font mode for every baseline. Any baseline re-recorded for this
  story is recorded under deterministic fonts, never system fonts.
- The re-record is deliberate and is its own labelled commit, not incidental.

No other row matches. There is no unit conversion, no theme colour, no parser or
serialiser, no dependency-graph change, no public API surface, and no new trait,
module or file.

## Hash harness

**Unchanged, 28 of 28.** The plan predicted a delta before implementation. The
measured result is no delta, and the reason matters more than the prediction.

None of the seven corpus documents contains a footnote. The baseline covers
`contract`, `feature_showcase`, `invoice`, `letter`, `proposal`, `quote` and
`report`, and no sample generator under `crates/rdocx/examples/` emits a
footnote reference at all. The harness therefore exercises no part of
`render_page_footnotes`.

So 28 of 28 matching is **not** evidence that this fix is correct. It is only
evidence that the fix touched nothing outside the note path. The evidence for
correctness is the three regression tests, which reproduce the defect exactly
as `[72.0, 84.0, 84.0, 84.0]` before the change.

This is a real coverage gap and it applies to F-X013b and F-X013c as well. Both
will report a flat harness for the same reason, which must not be read as those
stories having no output effect. Closing the gap means adding a corpus document
with notes, which changes the baseline set and is its own decision rather than
something to fold into a defect fix. Recorded as a follow-up rather than done
here.

## Implementation checklist

- [x] Record the pre-change harness state so the delta set is attributable
- [x] Hoist the `x` cursor in `render_page_footnotes` and advance by `seg.width`
- [x] Add the two named regression tests and the unit comparison test
- [x] Run `cargo test -p rdocx-layout`, 54 passed
- [x] Run the harness. No delta, because the corpus holds no footnote. Recorded
      above as a coverage gap rather than as a pass
- [x] No baseline re-record needed, since nothing moved
- [x] `/microscope F-X013a --working`, pass 1 found 1 defect and 2 smells,
      pass 2 clean
- [x] Fix D1, advance for `Tab` and `Image` as well as `Text` and `Marker`
- [x] Fix the break-width and indent disagreement the advance fix exposed
- [x] Validate end to end against `sample1.docx`, the external PR's own sample
- [x] `/verify`

## Open questions

None. The two scope questions raised during design, endnote placement and
oversized-note splitting, were both taken into scope and belong to F-X013c and
F-X013b respectively. Neither affects this story.
