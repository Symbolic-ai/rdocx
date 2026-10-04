# F-274, correctness, pass 1

**Reviewed**: working diff against the claimed base, 26 tracked files, 2,179 insertions and 211 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, unselected special records render
`crates/rdocx-layout/src/notes.rs:140`

The registry lays out every special note record in a part. It does not consult the special IDs selected by document note properties. A package with an unreferenced separator or two separator records can therefore render content Word does not select, and the last duplicate type wins by hash-map insertion order.

### D2, removing a document policy hides its authored separator
`crates/rdocx/src/document.rs:21500`

`remove_note_policy` removes the whole note-property element, including the `special_references` mapping that `set_note_special_record` added. After a consumer clears a numbering or placement override, Word no longer selects that authored separator or continuation record, although the special note owner remains.

### D3, endnote carryover drops authored continuation notices
`crates/rdocx-layout/src/paginator.rs:3011`

The endnote spill loop draws an authored continuation separator on the new page but never lays out or draws `ContinuationNotice` on the page from which the endnote continues. The design requires authored continuation content for both families.

### D4, opposite-family special references change during a note-property rewrite
`crates/rdocx-oxml/src/document.rs:1510`

The parser accepts both `w:footnote` and `w:endnote` children in either note-property element. The serializer chooses a child name from the parent, so a `w:endnote` child inside `w:footnotePr` becomes `w:footnote` after a modeled property edit instead of staying unmodelled verbatim.

### D5, oversized endnote continuation separator can prevent pagination progress
`crates/rdocx-layout/src/paginator.rs:3035`

After a page flush, an authored continuation separator can consume the full page height. The next loop iteration sees no note line fitting, flushes again because `cursor_y` is nonzero, and repeats forever with the same note line. A checked oversized special record or a progress fallback is needed.

## Smells

None found.

## Nitpicks

None found.

## Not found

No additional defects found in the checked API validation, custom body marker serialization, or focused first-page separator placement. No new traits, generic parameters, crates, or modules were introduced.
