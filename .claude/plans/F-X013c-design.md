# F-X013c, Endnotes at the document end

**Status**: completed
**Sprint**: S41
**Size**: M
**Depends on**: F-X013b

## Problem

An endnote reference renders its note at the foot of the page carrying the
reference, exactly as a footnote does. Word puts endnotes at the end of the
document. `engine.rs:620` handles `RunContent::FootnoteRef` and
`RunContent::EndnoteRef` in one arm and records only a number, so by the time
layout sees a reference there is nothing left to say which stream it came from.

The ambiguity is not only cosmetic. `TextSegment::footnote_id` is an
`Option<i32>`, and `NoteRegistry` is keyed by that number alone. A document
numbering a footnote and an endnote alike collapses them into one entry.
`sample1.docx` does exactly this, with a footnote 2 and an endnote 2, and
building the registry silently rendered whichever stream was inserted last.
F-X013b pinned the order so footnotes win, which restored the old behaviour but
left the collision in place. This story removes it.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put", for note placement belonging
  to the paginator, which F-X013b established and which this story extends with
  a document-end region.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the regression
  category, and "The hash harness" for the labelled-delta rule.
- `docs/hld/14-development-backlog.md`, "F-X013c, Endnotes at the document end".

## Approach

### 1. A reference says which stream it came from

`oxml-layout` replaces the bare number with a tagged reference:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteStream { Footnote, Endnote }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteRef { pub stream: NoteStream, pub id: i32 }
```

`TextSegment::footnote_id` and `GlyphRun::footnote_id` become
`note: Option<NoteRef>`. The concept already lived in `oxml-layout`, so this
makes an existing field precise rather than pushing a new Word idea into the
format-neutral layer. `rpptx-render` propagates the field and needs the rename
only.

`RunRef::footnote_id()` in the public `rdocx` API reads the oxml model directly
rather than a layout segment, so it is untouched.

### 2. The registry keys by stream and id

`NoteRegistry` keys on `NoteRef` instead of `i32`. The precedence hack F-X013b
added, where an id already taken is skipped, is deleted: with the stream in the
key there is no collision to break.

### 3. Footnotes keep the page foot, endnotes get the document end

The paginator's note area admits `NoteStream::Footnote` only. Endnote
references are ignored by `claim_notes` and by the reserve, so they cost a page
nothing.

After pagination, endnotes are emitted as their own pages appended after the
last body page, in first-reference order across the document. They are laid out
as ordinary flow content from the top of the page, not anchored to the bottom,
and they carry no separator rule, because a separator exists to divide notes
from body text on a shared page and an endnote page has no body.

**Assumption, stated rather than asked.** Endnotes begin on a fresh page after
the last body page. Word continues them on the last body page when there is
room. A fresh page is chosen because an endnote flowing into a page that also
owes footnotes would put two note regions on one page competing for the same
height, and the interaction is not worth its complexity for a first cut. Easily
revisited, and recorded here so the choice is legible rather than accidental.

Endnote markers keep the raw id. Word defaults endnotes to lower roman
numerals through `w:endnotePr/w:numFmt`, which is a numbering-format story
rather than a placement one and is not taken here.

Endnote pages use the final section's geometry, which is what the document end
means.

## Rejected alternatives

- **Keep one number and disambiguate by lookup order.** That is what F-X013b
  does as a stopgap, and it is exactly the defect this story exists to remove.
- **A second `Option<i32>` field for endnotes.** Two fields that must agree,
  with a representable state where both are set.
- **Endnotes as a synthetic trailing section fed back through the paginator.**
  Cleaner in principle, but a section carries headers, footers and page
  numbering that endnote pages would then inherit or have to suppress. More
  moving parts than appending pages.
- **Continue endnotes on the last body page.** See the assumption above.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_footnote_and_an_endnote_sharing_a_number_render_their_own_text` | A document with footnote 2 and endnote 2 renders both texts, each in its own region |
| regression | `endnotes_render_after_the_last_body_page` | Endnote text appears on a page after every page holding body text, and on no page holding body text |
| regression | `an_endnote_reference_does_not_reserve_space_at_the_page_foot` | A page whose only reference is an endnote draws no separator and loses no body height |
| unit | `footnotes_and_endnotes_keep_their_own_regions` | A document with both puts footnote text above the bottom margin of its own page and endnote text on a later page |

**Test gate**, from the backlog: the first two regressions.

The F-X013b tests are retained unchanged and must keep passing, which is what
proves footnote behaviour did not move.

## HLD impact

- `docs/hld/03-architecture.md`, the note placement paragraph F-X013b added,
  extended to say endnotes are emitted at the document end rather than at the
  page foot.

## Risk routing

Two rows match.

- **Layout, pagination, line breaking, text shaping.** Deterministic font mode
  for any baseline recorded, re-records deliberate and separate. This story adds
  pages to documents that use endnotes, which is the largest output change in
  the sprint.
- **Public API of a published crate.** `TextSegment` and `GlyphRun` are public
  in `oxml-layout`, so replacing `footnote_id` with `note` is a breaking change
  to that crate's surface. `oxml-layout` is an incubating 0.2.0 package, and the
  field has no external consumer beyond this workspace. Stated at completion.
  `rdocx`'s own public API is unchanged.

No parser or serialiser is touched: the two reference kinds already parse
distinctly in `rdocx-oxml`, and only layout conflated them.

## Hash harness

**Unchanged, 28 of 28.** No corpus document contains a note of either kind, the
blind spot F-X013a recorded. A delta would mean this story reached a document it
should not have.

Evidence is the regression set plus an end-to-end render of `sample1.docx`,
which holds a footnote and an endnote both numbered 2. Its footnote must stay on
page 5 and its endnote must move to a new final page.

## Implementation checklist

- [x] Record the pre-change harness state
- [x] `NoteStream` and `NoteRef` in `oxml-layout`, replacing `footnote_id`
- [x] Set the stream from the `FootnoteRef` and `EndnoteRef` arms in the engine
- [x] Key `NoteRegistry` by `NoteRef`, delete the F-X013b precedence skip
- [x] Restrict the page note area to footnotes
- [x] Emit endnote pages after the last body page
- [x] Tests, including the shared-number regression
- [x] Full suite, harness, and an end-to-end `sample1.docx` render
- [x] `/microscope F-X013c --working`
- [x] `/verify`

## Open questions

None. The one judgement call, whether endnotes start a fresh page, is recorded
as a stated assumption above rather than left open.
