# F-X013b, correctness, pass 1

**Reviewed**: the uncommitted working tree. Five source files,
`rdocx-oxml/src/footnotes.rs`, `rdocx-layout/src/notes.rs` (new),
`rdocx-layout/src/paginator.rs`, `rdocx-layout/src/engine.rs` and
`rdocx/src/document.rs`. Roughly 810 added and 250 removed lines, of which the
engine's are mostly the deletion of `render_page_footnotes` and the tests
replacing it.
**Verdict**: 0 defects, 2 smells, 1 nitpick

Two defects were found and fixed during the pass. Both are recorded below with
the evidence, because both were invisible to the test set as first written and
only a sweep across reference positions exposed them.

## Defects

None outstanding.

### D1 found and fixed, the note area was measured from a stale cursor
`crates/rdocx-layout/src/paginator.rs:491`

`place_page_notes` sized the note area as `content_height - cursor_y`, but
`cursor_y` includes the space after the last paragraph placed. That trailing
space collapses at a page break, so measuring from it handed the note area less
room than had been reserved for it. When the shortfall exceeded the note's
first line, nothing could be placed, and the whole note carried to the next
page, arriving one page after its own reference.

Reproduced by a document of 60 body paragraphs with a single-paragraph note
referenced from paragraph 0: the note was drawn on page 2. The same document
with a two-paragraph note drew on page 1, which is why the first test set passed
and hid it.

Fixed by tracking `ink_bottom`, where the body's last mark actually sits, and
measuring the note area from that. Reserve and placement now agree by
construction, which was the point of the story.

### D2 found and fixed, notes were claimed before their paragraph had a page
`crates/rdocx-layout/src/paginator.rs:824`

`paginate_paragraph` called `claim_notes(&para.lines)` before deciding whether
the paragraph fits. A paragraph that then moved wholly to the next page left its
note claimed on the page it had left, so the note was drawn on the page before
its reference.

Reproduced by sweeping the reference across all 60 paragraphs: position 32 put
the reference on page 2 and the note on page 1.

Fixed by splitting pricing from claiming. `available_height_for` prices a
paragraph's notes without committing to them, and `claim_notes` is now called
only where lines are actually placed: the whole-paragraph path, the split path
for `lines[..split_at]`, and the split tail. A paragraph's notes now travel with
it.

Both are gated by `a_note_is_drawn_on_the_page_that_carries_its_reference`,
which fails against either fix reverted, at position 32 for D2 and at many
positions for D1.

## Smells

### S1, note line breaking still uses the final section's width
`crates/rdocx-layout/src/engine.rs:162`

`NoteRegistry::build` is handed `final_geometry.content_width()`, so in a
document whose sections differ in page width, notes are broken to the last
section's measure wherever they appear.

The design plan claimed this story fixes the wrong-geometry defect. That is only
half true and the plan has been corrected. Note *positioning* is fixed, because
each section builds its own `Pager` with its own geometry and the note area is
placed against that. Note *line breaking* is not, because the registry is built
once, ahead of pagination, and a note laid out per section would have to be laid
out once per distinct width.

No corpus or test document has sections of differing width, so this is not
reachable today. Recorded rather than fixed, because fixing it means keying the
registry by width, which is a real design change and not something to graft on
at review time.

### S2, unplaceable note content is dropped silently
`crates/rdocx-layout/src/paginator.rs:735`

`flush` keeps making pages while notes remain, and breaks out if a page places
nothing, to avoid looping forever. That guard is correct, but it discards the
remaining note content without a trace.

It is not reachable through any input found: the page-empty exemption in
`count_lines_that_fit_with_notes` guarantees at least one note line per page.
It is a backstop against a future change breaking that guarantee, and if it ever
fires the symptom is silently missing footnote text. Recorded as the kind of
thing that should surface rather than swallow.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs:329`, `height_of(*first,
  note.lines.len())` reads as though it takes a count from the start, when it
  means "everything from `first` onward". `height_from(first)` would say it.

## Not found

Checked and produced nothing:

- **panics**. No indexing, slicing, unwrap or expect on any input-derived value
  in the new code. `note.lines.get(first)` and the `skip`/`take` iterators
  cannot go out of bounds. `shape_marker` returns `Ok(None)` rather than
  propagating a font failure, matching how the deleted code behaved.
- **ooxml**. `w:type` is read prefix-tolerantly through the existing
  `matches_local_name`, and written with the fixed `w:` prefix ahead of `w:id`.
  The round trip is proven a fixed point by
  `a_separator_definition_survives_open_and_save`, and a foreign prefix by
  `note_types_are_read_through_a_foreign_prefix`. An unknown `w:type` degrades
  to `Normal` rather than erroring, which is the tolerant read the domain rules
  ask for.
- **structure**. One new module, asked for and approved before writing. No new
  trait, generic parameter, wrapper or feature flag. `NoteRegistry` has one
  consumer today, which is the paginator, and it exists to stop a second
  computation of note placement rather than to anticipate one.
- **contract**. The implementation matches the plan. Two plan claims were wrong
  and have been corrected in the plan itself: the geometry fix is partial, see
  S1, and the note stream retains separators, which changes what
  `CT_Footnotes::footnotes` contains.
- **Public API**. `CT_Footnote` gains a public field, a semver-minor addition.
  `Document::footnotes()` filters to `Normal`, so its observable behaviour is
  unchanged despite separators now being retained in the model. Covered by
  `get_by_id_does_not_return_a_separator`.
