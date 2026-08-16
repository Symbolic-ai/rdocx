# F-X017, Notes broken to their own section's width

**Status**: approved
**Sprint**: S43
**Size**: S
**Depends on**: F-X013b

## Problem

`crates/rdocx-layout/src/engine.rs:172-179` builds the `NoteRegistry` once, and
passes `final_geometry.content_width()` as the width every note is broken to.
`final_geometry` comes from the body-level `sectPr`, which is the **last**
section's properties.

`crates/rdocx-layout/src/notes.rs:79-89` breaks every note at
`content_width - NOTE_INDENT` and the paginator draws it against the same
measure, so reserve and render agree with each other. They agree with the wrong
number whenever the section that carries the reference is not the final one and
the two differ in width. A landscape section followed by a portrait one breaks
its footnotes to the portrait measure, then draws them into the landscape page,
where the text stops well short of the right margin and the reserved height is
larger than the note needs.

Note *placement* is already per-section: `Pager` holds the section's own
`geometry` and `crates/rdocx-layout/src/paginator.rs:699-717` measures the
separator and the note area from it. Only the line breaking is not, which is the
half F-X013b left open.

## Spec reference

- `docs/hld/03-architecture.md`, the `rdocx-layout` paragraph on the
  `NoteRegistry`, "Footnotes and endnotes are laid out once into a
  `NoteRegistry` before pagination". That paragraph states the once-before-
  pagination rule this story keeps and the single width it silently assumed.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", for the `regression`
  category the gate uses.
- `docs/hld/14-development-backlog.md`, "F-X017, Notes broken to their own
  section's width".

## Approach

Keep the once-before-pagination rule. It is what lets the paginator reserve
without a mutable font manager, and nothing here needs it relaxed. Lay each note
out **once per distinct content width** instead of once per document.

`NoteRegistry::build` takes the widths rather than a width:

```rust
pub fn build(
    input: &LayoutInput,
    styles: &CT_Styles,
    media: &MediaRegistry,
    fm: &mut FontManager,
    num_state: &mut NumberingState,
    content_widths: &[f64],
) -> Result<Self>
```

The map is keyed by the note and the width it was broken to:

```rust
notes: HashMap<(NoteRef, u64), NoteLayout>,   // u64 is content_width.to_bits()
```

`to_bits` is exact, and both the key and the lookup derive from the same
`PageGeometry::content_width()` arithmetic on the same `sectPr`, so equality is
bit equality with no tolerance question. Duplicate widths collapse, so the
common document whose sections share a page size lays each note out exactly
once, as today.

Lookup gains the width:

```rust
pub fn get(&self, note: NoteRef, content_width: f64) -> Option<&NoteLayout>
```

Call sites already hold the right width:

- The eight `self.notes.get(..)` sites in `Pager`
  (`paginator.rs:366, 370, 396, 414, 438, 658, 694, 727`) pass
  `self.geometry.content_width()`, which is the section being paginated.
- `append_endnote_pages` (`paginator.rs:921, 949`) passes the final geometry's
  content width, which is the width it already draws endnotes at. Endnotes are
  emitted after the last body page, so the final section is the correct measure
  for them and their output does not move.

`engine.rs` collects the widths it will paginate, which is every
`sections[i].geometry.content_width()` plus `final_geometry.content_width()`
for the endnote pages, and passes them in. Every lookup key is therefore
registered by construction: the widths come from the same `Section` values the
paginator is handed.

`NoteLayout`, marker shaping, the continuation-separator flag and every
reserve, split and draw rule are untouched.

## Rejected alternatives

- **Lay notes out lazily, during pagination, at the width in hand.** Needs
  `&mut FontManager` inside the paginator. `notes.rs:9-11` records why it does
  not have one, and reversing that is a much larger change than this defect
  earns.
- **Key the registry by section index instead of width.** Threads a section
  index through `Pager` and `append_endnote_pages` that nothing else wants, and
  re-lays notes for two sections that share a width. Width is the property the
  layout actually depends on.
- **Re-break only when the widths differ, keeping a single-width fast path.**
  Two code paths for one behaviour. Deduplicating on the width key already
  gives the single-width document the same single layout, without a branch.
- **Round or quantise the width key.** Invents a tolerance no caller asked for.
  Both sides of the comparison are the same computation on the same input.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_note_is_broken_to_the_width_of_its_own_section` | A two-section document, sections differing in page width, breaks each note to the measure of the section holding its reference. The wide section's note produces fewer lines than the narrow section's note for the same text |
| regression | `a_single_section_document_lays_notes_out_exactly_as_before` | Line count, line widths and reserved height for a one-section document are identical to the pre-change values, so the fix cannot move the common case |
| unit | `the_registry_lays_a_note_out_once_per_distinct_width` | Two sections sharing a width produce one entry per note, two differing widths produce two |
| unit | `an_endnote_is_broken_to_the_final_sections_width` | Endnotes, which are emitted after the last body page, are looked up at the width they are drawn at |

**Test gate**, from the backlog: the first regression, plus the byte-identical
single-section half of the same gate, which is the second row.

## HLD impact

- `docs/hld/03-architecture.md`. The `NoteRegistry` paragraph gains the width
  rule: laid out once per distinct section content width rather than once per
  document, and endnotes measured against the final section because that is
  where they are emitted.

## Risk routing

Matched rows: **Layout, pagination, line breaking, text shaping** and **Public
API of a published crate**. The second was added during microscope pass 1, which
recorded it as smell S2 after finding it undeclared.

Layout:

- Read `docs/hld/08-rendering-spec.md` before editing.
- Deterministic font mode for every baseline. The new regression constructs its
  document in code and asserts on line counts and line widths, not on a
  recorded image, so it needs no new baseline.
- Re-record deliberately, never incidentally. This story expects no delta at
  all, so any harness movement is a defect and not a re-record prompt.

Published API:

- **Semver impact, breaking.** `crates/rdocx-layout/src/lib.rs:9` exports
  `pub mod notes`, so `NoteRegistry::build` and `NoteRegistry::get` are public
  surface of `rdocx-layout`, published at 0.7.0. `build` takes `&[f64]` where it
  took `f64`, and `get` takes the width as a second parameter. An external
  caller of either does not compile against the new version. Under 0.x that is a
  minor bump, and the next `/release` states it.
- No caller outside `rdocx-layout` exists in this workspace, which is why the
  change compiles cleanly and why the impact has to be declared rather than
  observed.
- No surface is added that no story asked for. The two signatures that changed
  are the two the story is about.

## Hash harness

**Expected unchanged.** `crates/rdocx/examples/generate_all_samples.rs` defines
no section break and no footnote or endnote in any of the seven samples, so no
sample reaches either the old or the new code path. A delta would mean the
single-width path changed, which is precisely what the second regression pins.

## Implementation checklist

- [x] Record the pre-change harness state, 28 of 28
- [x] `NoteRegistry` keyed by `(NoteRef, width bits)`, `build` taking the widths
- [x] `get` taking the width, and the `Pager` call sites passing the section's
      own content width
- [x] `append_endnote_pages` passing the final geometry's content width
- [x] `engine.rs` collecting the section widths
- [x] The tests, added to the existing modules rather than a new binary
- [x] Update `03-architecture.md`
- [x] Declare the published-surface semver impact, per microscope S2
- [x] `cargo test -p rdocx-layout`, `/microscope F-X017 --working`, `/verify`

## Open questions

None material. The width key is an implementation detail with a stated
justification, and the endnote measure follows from where endnotes are drawn.
