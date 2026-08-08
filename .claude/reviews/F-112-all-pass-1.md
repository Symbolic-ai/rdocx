# F-112, all, pass 1

**Reviewed**: uncommitted working diff against `HEAD`, 4 files, 518 additions and 2 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, inserting paragraph properties does not move the pre-property raw boundary

`crates/rpptx/src/lib.rs:1684`
`crates/rpptx/src/lib.rs:1689`
`crates/oxml-drawing/src/text/paragraph.rs:1544`

Both paragraph setters create an absent `a:pPr` by assigning the option
directly. A parsed paragraph without `a:pPr` can already have preserved content
at raw boundary 0 before its first run, including an `mc:AlternateContent`
substitution for a run. The writer emits boundary 0 before `a:pPr`, so the new
property follows that preserved content. After markup-compatibility processing,
the paragraph can therefore contain a run before `a:pPr`, violating the
required sequence and producing a repair risk. Creating `a:pPr` must shift the
paragraph raw boundaries from 0 so the preserved content stays before the first
run at boundary 1.

The preservation test constructs exactly a boundary-0 marker before the first
field, then checks only that `a:pPr` precedes the field. It does not check that
`a:pPr` also precedes the preserved marker, so the current assertion at
`crates/rpptx/tests/integration.rs:235` remains green with the ordering defect.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness produced no other findings. `CT_TextBody::new`, body replacement,
paragraph replacement, and paragraph append all retain at least one paragraph.
Replacement keeps body properties, optional list style, first-paragraph
properties, end properties, and placeholder metadata. It deliberately replaces
the selected paragraph's runs, fields, and breaks with one regular run. Append
keeps existing choices and caller order.

Raw-boundary handling outside D1 produced no findings. Text-body replacement
collapses deleted paragraph boundaries after the one surviving paragraph while
retaining their byte order. Paragraph replacement separates raw content before
properties, after runs, and after end properties. Paragraph and run append move
the old trailing boundary after the appended item. Untouched fields and breaks
remain typed and ordered.

Text value handling produced no findings. New and replaced text is escaped by
the existing `TextValue` writer, which adds `xml:space="preserve"` for leading
or trailing whitespace. A direct run text replacement retains the source space
intent and the run's typed and unmodelled state.

Contract produced no other findings. `ShapeMut::set_text` creates a minimal
valid body for an ordinary shape without one and leaves an existing body's
frame and list-level state intact. `a:lstStyle` is optional in a minimal body.
`text_frame` remains available only for an ordinary shape that owns a body.
Paragraph, bullet, character-property, and Latin-font values are accepted by
ownership and written by their existing typed schema-order writers.

Panics produced no findings. Invalid shape and paragraph indices return
`None`. Unsupported shape kinds return `None` or a contextual error. The
post-push paragraph and run assertions are justified by immediate local
invariants.

Borrow handling produced no findings. Each nested mutable handle is tied to the
borrow used to obtain it. Structural mutation cannot occur while a returned
paragraph or run handle is live, and append returns a borrow of the exact item
that was just inserted.

OOXML produced no other findings. New text uses fixed `a:` prefixes and the
existing paragraph, character-property, font, and bullet writers. Clearing
text emits one paragraph and validates without `EmptyTextBody`. Placeholder
type and `idx` remain unchanged.

Tests produced no other findings. `cargo test -p oxml-drawing` passed with 117
tests and 2 ignored tests. `cargo test -p rpptx --test integration` passed with
46 tests and 4 ignored tests. The render gate saves and reparses the changed
deck, resolves the changed placeholder text, then compares its raster output
with the cleared form through `layout_presentation_deterministic`. That path
loads bundled and presentation-embedded fonts only and records no system-font
baseline. `cargo fmt --all --check` and `git diff --check` passed.

Structure produced no findings. No new module, trait, generic parameter,
feature, dependency, forwarding wrapper, or erased concrete type was added.
