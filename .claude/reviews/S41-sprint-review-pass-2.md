# S41 sprint review, pass 2

**Reviewed**: `sprint/s41` against `1991e37` after the pass 1 remediation, 40
files, 5832 insertions and 349 deletions. Crates unchanged from pass 1.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None outstanding.

### S1 from pass 1, resolved
`crates/rdocx-layout/src/paginator.rs:1294` and
`crates/rdocx-layout/src/paginator.rs:1328`

Both split paths now hand `count_lines_that_fit_with_notes` a start position
that includes `content_offset_top`, so the counter measures from where the lines
are actually drawn.

Pinned by `a_split_paragraph_clearing_a_drawing_stays_inside_the_page`, which
reproduces the defect directly rather than by proxy: against the unfixed code it
draws text at 720.7 with the bottom margin at 720, and it asserts no glyph on
any page passes that margin.

### S2 from pass 1, resolved
`crates/rdocx-layout/src/paginator.rs:1420`

`render_para_split` adds `content_offset_top` when recording `ink_bottom`, so
the split path and the whole-paragraph path now agree on where the body's last
mark sits. The note area is measured from that value, so this removes the
narrow case where a footnote could be drawn over the last line of a split
paragraph.

Covered by the same regression, since an ink bottom that is too high produces a
note area that is too tall and the overrun assertion catches the consequence.

### S3 from pass 1, resolved
`docs/hld/03-architecture.md:73`

The sentence claiming `oxml-layout`'s output, font and line modules are "100
percent docx-free" is gone. The spec now says those modules name no document
format, then states the one exception plainly: a text segment carries an
optional `NoteRef`, notes are a WordprocessingML construct with no
PresentationML counterpart, and it sits there because a note reference has to
survive line breaking, which is shared code.

That is the honest correction rather than the larger one. The alternative,
moving the reference behind a format-neutral name, would rename a thing to hide
what it is.

## Nice-to-have

### N1 and N2 from pass 1, both resolved

`NoteRegistry::is_empty` is deleted, having had no caller. `NOTE_FONT_SIZE` is
narrowed to private, since only its own module reads it. The other three note
constants remain public because the paginator reads them.

## Milestone gate

All five clauses of the sprint's definition of done still hold, on the same
evidence recorded in pass 1. Re-checked after remediation:

- Full workspace suite: zero failures across 53 test binaries, now including
  `a_split_paragraph_clearing_a_drawing_stays_inside_the_page`.
- Hash harness: 28 of 28, unchanged.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- Prose rules and Codex adapter check: clean.

**The remediation moved no output.** Re-rendering `sample1.docx` produces nine
pages byte identical to the pre-remediation render. That is the useful signal
here: three of the four fixes touch pagination arithmetic, and byte-identical
output on a document that exercises footnotes, endnotes and both wrap modes says
the corrections were confined to paths no current document reaches.

## Not found

Re-checked after remediation, all still clean: **layering**, **deps**,
**duplication**, **harness**, **surface**, **interaction**, **docs**. No
`Cargo.toml` changed at any point in the sprint. The remediation removed public
surface rather than adding any.

## Exit

Zero blocking and zero should-fix. The sprint is ready for `/close-sprint`,
which is the only command permitted to merge to `main` or create the `s41` tag,
and which has not been run.
