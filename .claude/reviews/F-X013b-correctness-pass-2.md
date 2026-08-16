# F-X013b, correctness, pass 2

**Reviewed**: the uncommitted working tree after the pass 1 remediation.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None. D1 and D2 were found and fixed during pass 1 and remain covered by
`a_note_is_drawn_on_the_page_that_carries_its_reference`, which fails against
either fix reverted.

## Smells

None outstanding.

### S1 from pass 1, filed rather than fixed

Note line breaking still uses the final section's content width. Filed as
**F-X017, Notes broken to their own section's width (S)**, depending on
F-X013b, with its own test gate. Not reachable by any current document, since
no corpus or test document has sections of differing width, and fixing it means
keying the registry by width, which is a design change rather than a review-time
patch. The design plan's overclaim has been corrected.

### S2 from pass 1, resolved
`crates/rdocx-layout/src/paginator.rs:741`

The no-progress backstop in `flush` no longer drops note content silently. It
carries a `debug_assert` naming what would be lost, so a future change that
breaks the one-note-line-per-page guarantee fails loudly in tests rather than
quietly truncating a footnote in release output. The `break` is retained so a
release build stops rather than spins.

## Nitpicks

### Resolved

`NoteLayout::height_of(first, lines.len())` read as a count from the start when
it meant "everything from here on". A `height_from(first)` helper now says that,
and `reserve_for` uses it.

## Not found

Re-checked after remediation, all still clean: **panics**, **ooxml**,
**structure**, **contract**, **public API**. The remediation touched three
lines of logic plus one helper, added no new surface, and the full suite,
clippy, formatting, the harness, the prose rules, the Codex adapter check, the
WASM targets and the bundled-fonts-off path all pass.
