# F-X013b, Footnote reservation and splitting

**Status**: completed
**Sprint**: S41
**Size**: L
**Depends on**: F-X013a

## Problem

Notes are placed after pagination has already finished. `Engine::layout` calls
`paginator::paginate_sections`, and only then `render_page_footnotes`, which
computes `page.height - margin_bottom - total_fn_height - separator_offset` and
draws the note area upward from the bottom margin.

Pagination therefore fills the page with body content having no idea the note
area exists. `Pager::content_height` is `geometry.content_height()` and nothing
subtracts from it. On a page whose body reaches the bottom margin, the note area
is drawn straight over the body text. This is visible on page 5 of the external
contribution's own `sample1.docx`, where the note overprints the table of
contents entries above it.

Two further consequences of the split placement:

- Notes are laid out twice, once inside `render_page_footnotes` and never
  anywhere else, so no reservation can be consistent with what is drawn.
- `render_page_footnotes` receives `final_geometry`, the last section's
  geometry, so in a multi-section document notes are positioned against the
  wrong page size.

Separately, the note stream model cannot express what this story needs.
`CT_Footnotes::from_xml` decides what is a separator by testing `id <= 0`
(`footnotes.rs:67`) rather than by reading `w:type`. In `sample1.docx` the
`continuationSeparator` carries `w:id="1"`, so it is stored and returned as if
it were a real note. `to_xml_root` never writes `w:type` at all and only writes
the notes it kept, so opening and saving a document silently deletes its
separator definitions.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put", for `rdocx-layout` owning the
  flow model and the paginator, which is what makes the paginator the right
  home for note placement.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the regression
  category, and "The hash harness" for the labelled-delta rule.
- `docs/hld/14-development-backlog.md`, "F-X013b, Footnote reservation and
  splitting".

## Approach

Lay each note out exactly once, before pagination, then let the paginator
reserve, split and draw from that single source. Reserve and render stop being
two computations that can disagree.

### 1. Note types in `rdocx-oxml`

`CT_Footnote` gains a type, and the parser reads `w:type` instead of guessing
from the id.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoteType {
    #[default]
    Normal,
    Separator,
    ContinuationSeparator,
    ContinuationNotice,
}

pub struct CT_Footnote {
    pub id: i32,
    pub note_type: NoteType,
    pub paragraphs: Vec<CT_P>,
}
```

All notes are retained, separators included, so a round trip stops discarding
them. `to_xml_root` writes `w:type` for any note that is not `Normal`.
`get_by_id` filters to `Normal`, so no layout path can pick up a separator, and
a document whose `continuationSeparator` sits at id 1 no longer exposes it as a
real note.

`Document::footnotes()` is public API of a published crate and must keep
returning only real notes. It gains the same `Normal` filter, so its observable
behaviour is unchanged.

### 2. A new `crates/rdocx-layout/src/notes.rs`

Explicitly asked for and approved, per the module rule in `CLAUDE.md`.

```rust
pub struct NoteLayout {
    /// Pre-shaped superscript number, so the paginator never needs a font.
    pub marker: TextSegment,
    /// The note's lines, flattened across its paragraphs.
    pub lines: Vec<LayoutLine>,
}

pub struct NoteRegistry { /* private */ }

impl NoteRegistry {
    pub fn build(
        input: &LayoutInput,
        styles: &CT_Styles,
        media: &MediaRegistry,
        fm: &mut FontManager,
        num_state: &mut NumberingState,
        content_width: f64,
    ) -> Result<Self>;
    pub fn get(&self, id: i32) -> Option<&NoteLayout>;
    pub fn has_continuation_separator(&self) -> bool;
    pub fn is_empty(&self) -> bool;
}
```

Built once in `Engine::layout` while `&mut FontManager` is still available.
Notes are laid out at `content_width - FOOTNOTE_INDENT`, the width F-X013a
established. Because the marker is shaped here, the paginator needs no font
access, which matters because it only holds `&FontManager` and shaping needs
`&mut`.

### 3. Reservation and splitting in `paginator.rs`

`Pager` gains the registry and per-page note state:

```rust
struct PlacedNote { id: i32, first_line: usize, line_count: usize, continued: bool }

notes: &'a NoteRegistry,
page_notes: Vec<PlacedNote>,      // notes placed on the page being built
carry: Option<PlacedNote>,        // a note that ran past this page's bottom
```

- `reserved_height()` is zero when nothing is placed, otherwise the separator
  offset plus the heights of the note lines actually placed on this page.
- `available_height()` is `content_height - reserved_height()`, and replaces
  bare `content_height` at every body-fitting decision.
- Note ids are read from the line items already being placed, through
  `seg.footnote_id`. No new field on `ParagraphBlock` is needed, and a
  paragraph split across pages therefore reserves each note on the page whose
  line actually carries the reference, not on the page that owns the paragraph.
- `count_lines_that_fit` becomes note aware. It walks lines one at a time,
  accumulating the notes each line introduces, and stops before the first line
  whose acceptance would push the body past `content_height` minus the reserve
  that line implies. Greedy per line, so no fixpoint iteration is needed.

Splitting: the note area takes whatever height is left once the body has placed
at least one line. If a note's lines do not all fit, the lines that fit are
placed and the remainder becomes the `carry`, which is placed first on the next
page and drawn without repeating its marker. Forward progress is guaranteed by
capping the reserve so at least one body line always fits, which is what stops
an oversized note from emitting empty pages forever.

`finish_page` draws the separator and the placed note lines, then moves the
carry onto the fresh page. A page opening with a carry draws the continuation
separator, the full content width rule, rather than the one-third rule used for
a note that starts on its own page. Whether a continuation separator is drawn
at all follows `NoteRegistry::has_continuation_separator`, so a document that
never defined one does not gain a rule it did not ask for.

### 4. `engine.rs`

`render_page_footnotes` is deleted. `Engine::layout` builds the registry and
threads it through `paginate_sections` into each `Pager`. Because each section
constructs its own `Pager` with its own geometry, notes stop being positioned
against the final section's page size.

**Corrected after review.** That last sentence is only half right. Note
*positioning* is now per section. Note *line breaking* is not: the registry is
built once, ahead of pagination, against `final_geometry.content_width()`, so a
document whose sections differ in page width breaks its notes to the last
section's measure. Not reachable by any current document. Filed as F-X017.

## Rejected alternatives

- **Keep the post-pass and share only a height map.** Rejected in design with
  the user. Reserve and render stay two computations that can drift, and the
  splitting logic would have to exist in both.
- **Reserve per paragraph rather than per line**, as the external PR did. A
  paragraph split across a page boundary would reserve its notes on the wrong
  page.
- **Scan the finished page elements for note ids**, as the current post-pass
  does. That works only after placement is final, which is too late to inform
  the placement itself.
- **Iterate to a fixpoint** between reserve and body height. The greedy per-line
  walk reaches the same answer without the loop, because the reserve only ever
  grows as lines are added.
- **Split notes at paragraph rather than line granularity.** Coarser than the
  page boundary it exists to serve, and a single long note paragraph would not
  split at all.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_page_whose_body_fills_the_text_area_does_not_overlap_its_notes` | The lowest body glyph sits above the separator, on a document whose body would otherwise reach the bottom margin |
| regression | `a_note_taller_than_its_remaining_space_continues_on_the_next_page` | The note's later lines appear on page 2, its earlier lines on page 1, and the marker is drawn once |
| regression | `a_page_referencing_one_note_twice_reserves_it_once` | Two references to the same id on one page produce one note block and one separator |
| regression | `an_oversized_note_still_leaves_room_for_body_text` | A note taller than a whole page leaves at least one body line on the page, and pagination terminates |
| unit | `a_continued_note_draws_the_continuation_separator` | Page 2 of a split note draws the full-width rule, page 1 the one-third rule, and neither is drawn when the stream defines no continuation separator |
| round-trip | `a_separator_definition_survives_open_and_save` | `w:type` values and separator notes are preserved byte for byte through parse and serialise, across namespace prefixes |
| unit | `get_by_id_does_not_return_a_separator` | A `continuationSeparator` at id 1 is not reachable as a real note, and `Document::footnotes()` still lists only real notes |

**Test gate**, from the backlog: the three named regressions covering overlap,
continuation, and reserving a repeated reference once.

Tests join the existing `mod tests` in each crate. No new `tests/` binary.

## HLD impact

- `docs/hld/03-architecture.md`, "What stays put". The sentence describing what
  `rdocx-layout` keeps needs to say that note placement is part of the
  paginator rather than a post-pagination pass.

## Risk routing

Three rows match.

- **Layout, pagination, line breaking, text shaping.** Read
  `docs/hld/08-rendering-spec.md`. Deterministic font mode for any baseline
  recorded, and any re-record is deliberate and separately committed. This
  story changes where body text can sit on a page carrying notes, so it is the
  highest-risk row here.
- **Any parser or serialiser.** `footnotes.rs` gains `w:type` on read and write.
  Prefix-tolerant on read, fixed `w:` prefix on write. A round-trip test proves
  a separator definition survives, which it does not today.
- **Public API of a published crate.** `Document::footnotes()` must keep
  returning only real notes despite separators now being retained in the model.
  `CT_Footnote` gains a public field, which is a semver-minor addition, stated
  at completion. No new public surface beyond what the story needs.

The new module was asked for and approved before writing, per the structural
rule in `CLAUDE.md`.

## Hash harness

**Unchanged, 28 of 28, and that proves nothing.** No corpus document contains a
note, so the harness cannot see any of this. This is the blind spot F-X013a
recorded, and it applies in full here.

Any harness delta at all would therefore be a genuine surprise and must be
explained before merging, since the only way this story reaches a corpus
document is through a path it was not supposed to touch.

The real evidence is the regression set plus an end-to-end render of
`sample1.docx`, where page 5 must stop overprinting its table of contents.

## Implementation checklist

- [x] Record the pre-change harness state
- [x] `NoteType` and `note_type` on `CT_Footnote`, parsed from `w:type`
- [x] Retain separators in the model, write `w:type` back, filter `get_by_id`
- [x] Filter `Document::footnotes()` so its public behaviour is unchanged
- [x] Round-trip test for separator preservation
- [x] New `crates/rdocx-layout/src/notes.rs` with `NoteRegistry` and pre-shaped
      markers
- [x] Build the registry in `Engine::layout`, thread it into the paginator
- [x] `Pager` note state, `reserved_height`, `available_height`, and
      `available_height_for`, which prices a paragraph's notes without claiming
      them
- [x] Note-aware `count_lines_that_fit`, replacing bare `content_height` at
      every body-fitting decision
- [x] Splitting with the carry, and the forward-progress cap
- [x] Continuation separator on a page opening with a carry
- [x] Delete `render_page_footnotes`
- [x] Full test suite, harness, and an end-to-end `sample1.docx` render
- [x] `/microscope F-X013b --working`
- [x] `/verify`

## Open questions

None outstanding. Two were asked and answered during design: note placement
moves into the paginator with a new module, and the continuation separator is
honoured rather than deferred.
