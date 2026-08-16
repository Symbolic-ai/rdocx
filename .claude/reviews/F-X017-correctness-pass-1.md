# F-X017, correctness, pass 1

**Reviewed**: the F-X017 working diff on `work/f-x017-claude`, 5 files, 387
insertions and 50 deletions. `crates/rdocx-layout/src/notes.rs`, `engine.rs`,
`paginator.rs`, `style_resolver.rs` and `docs/hld/03-architecture.md`.
**Verdict**: 1 defect, 2 smells, 2 nitpicks

## Defects

### D1, the endnote test does not prove what it claims
`crates/rdocx-layout/src/engine.rs:2765`

`an_endnote_is_broken_to_the_final_sections_width` counts endnote lines by
filtering glyph runs whose `text` starts with `"measure"`:

```rust
matches!(element, PositionedElement::Text(run) if run.text.starts_with("measure"))
```

A line becomes its own `GlyphRun`, so this counts only the lines whose break
happens to fall immediately before the word "measure" in `NOTE_PROSE`. That is
an accident of where the shaper broke this particular sentence at this
particular width. The test passes today, and it passes for a reason unrelated to
the property under test: change the note text, the font, or the margin and the
count silently becomes zero on both sides, where the equality assertion still
holds and the `> 0` guard is the only thing standing between the test and
vacuous success.

The endnote pages are identifiable structurally. `append_endnote_pages` appends
them after every body page, and this document has one body page per section, so
the endnote content is every text run on the pages after the first two.

## Smells

### S1, a width that is provably never new
`crates/rdocx-layout/src/engine.rs:180`

```rust
content_widths.push(final_geometry.content_width());
```

`engine.rs:163` pushes the final section unconditionally, before this line runs,
so `sections` always ends with a section whose geometry is `final_geometry` and
the pushed width is always already present. The line reads as though it handles
a case where the final section is absent from `sections`, and no such case
exists. A reader who trusts it will go looking for the path that needs it.

The endnote dependence it was meant to document belongs in the comment, which
already states it, rather than in a statement that does nothing.

### S2, an undeclared breaking change to a published crate's surface
`crates/rdocx-layout/src/notes.rs:88` and `crates/rdocx-layout/src/notes.rs:176`

`rdocx-layout` is published at 0.7.0 and `crates/rdocx-layout/src/lib.rs:9`
exports `pub mod notes`, so `NoteRegistry::build` and `NoteRegistry::get` are
public API. Both signatures changed incompatibly: `content_width: f64` became
`content_widths: &[f64]`, and `get` gained a parameter. No caller outside
`rdocx-layout` exists in this workspace, so nothing here fails to compile, and
that is exactly why it would be missed.

The design plan's `## Risk routing` records only the layout row. The **Public
API of a published crate** row also matches, and its mandatory concern is that
the semver impact is stated. It is not stated anywhere in the plan.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:2711`, `note_line_count` counts the note
  marker's own baseline as a line, because the marker is drawn a rise above the
  first line rather than on it. Every assertion using it compares two counts
  that are each inflated by one per note drawn, so the comparisons hold, but the
  number is not the line count its name promises.
- `crates/rdocx-layout/src/notes.rs:130`, `counters_before` is cloned once per
  note even when a single width is registered, which is every document today.
  One `HashMap` clone per note is not worth a branch, but the allocation is
  paid for a case that is usually absent.

## Not found

- **contract**. The diff does what the plan describes and stops there. Notes are
  keyed by width, the paginator passes its own section's width at all ten call
  sites, endnotes are looked up at the final width, and nothing else moved.
  Checked `paginator.rs:366, 370, 396, 414, 438, 658, 694, 727, 921, 949`.
- **panics**. No new `unwrap`, no indexing, no slicing and no arithmetic on
  untrusted input. `to_bits` is total. The `expect` calls are confined to test
  code, where a failure is the diagnostic.
- **ooxml**. No parser or serialiser is touched. No schema order, prefix or
  captured-subtree concern arises.
- **structure**. `NoteKey` is a type alias over a tuple used by the one struct
  that owns it. No new trait, no new generic parameter, no `Box<dyn>`, no
  wrapper, no feature flag, no new module or file. `#[derive(Clone)]` on
  `NumberingState` carries the sentence justifying it at
  `style_resolver.rs:41-45`.
- **tests**, in part. The gate,
  `a_note_is_broken_to_the_width_of_its_own_section`, was confirmed to fail
  against reverted code: registering only the final width makes it report the
  same line count for both sections. The single-section regression and the three
  registry unit tests hold. Only D1 is at issue.
