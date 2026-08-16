# F-X013c, correctness, pass 1

**Reviewed**: the uncommitted working tree. `oxml-layout/src/line.rs` and
`output.rs` for the tagged reference, `rdocx-layout/src/engine.rs`,
`notes.rs` and `paginator.rs` for the split, plus mechanical field renames in
`rpptx-render`, `rpptx-chart`, `oxml-pdf` and `rdocx-layout/src/convert.rs`.
**Verdict**: 0 defects, 1 smell, 1 nitpick

## Defects

None.

The one thing that could have gone wrong here, and did not, is footnote
behaviour moving while the endnote path was carved out of it. Every F-X013a and
F-X013b test still passes unchanged, and rendering `sample1.docx` leaves all
eight body pages byte identical, with only a ninth endnote page appended. That
is the strongest evidence available that this story is additive.

## Smells

### S1, an endnote line taller than a page is placed rather than dropped
`crates/rdocx-layout/src/paginator.rs:836`

`append_endnote_pages` guards its inner loop with "if nothing fits and the page
is already empty, place one line anyway". That is the same forward-progress
guarantee the page-foot path uses, and it is correct in that it terminates and
loses no text. It does mean a single note line taller than the content height
overflows the page rather than being clipped or scaled.

The same is true of body text today, so this is consistent rather than novel,
and a line taller than a page needs a font size no real document carries.
Recorded because the comment explains the loop, not the overflow.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs:751`, `page_foot_notes_in_line` is a
  filter over `notes_in_line` and the pair reads a little heavy for two call
  sites. Keeping both is still right, because the unfiltered iterator is what
  `append_endnote_pages` needs and collapsing them would inline the stream test
  at three sites instead of one.

## Not found

Checked and produced nothing:

- **correctness**. The stream is set from the `RunContent` variant at a single
  site, and `NoteRef` is `Copy`, `Eq` and `Hash`, so registry keying, dedup and
  ordering all agree on identity. The endnote emitter walks pages in order and
  collects first-reference order, which is the order a reader met them.
- **panics**. `notes.get(...)` returns `Option` and every use is guarded.
  `note.lines.get(first)` cannot go out of bounds. The `while first <
  note.lines.len()` loop advances by at least one line per iteration.
- **ooxml**. No parser or serialiser touched. `FootnoteRef` and `EndnoteRef`
  already parsed distinctly, and only layout conflated them.
- **structure**. No new module, trait, generic or feature flag. `NoteStream`
  and `NoteRef` replace a field rather than adding one, so no state where two
  fields disagree is representable. `draw_note` was extracted because the page
  foot and the document end must not drift apart in how a note looks, which is
  the same reasoning that produced F-X013a's defect when two paths diverged.
- **contract**. The implementation matches the plan, including the stated
  assumption that endnotes begin on a fresh page.
- **Public API**. `TextSegment::footnote_id` and `GlyphRun::footnote_id` become
  `note: Option<NoteRef>`, a breaking change to `oxml-layout`, which is an
  incubating 0.2.0 package with no consumer outside this workspace. `rdocx`'s
  public surface is unchanged: `RunRef::footnote_id()` reads the oxml model
  directly and still returns `Option<i32>`, and `Document::footnotes()` is
  untouched.
