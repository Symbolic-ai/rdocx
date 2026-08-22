# F-167, all, pass 2

**Reviewed**: complete working diff, 4 implementation files, 2,286 additions and 6 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, modeled fields cannot survive paragraph comparison
`crates/rdocx/src/comparison.rs:382`
`crates/rdocx-oxml/src/text.rs:2289`

The comparison serializes each matched run with `CT_R::to_xml`, but the run
serializer intentionally omits `RunContent::Field` because the paragraph
serializer owns `w:fldSimple` and complex field output. Comparing even two
identical paragraphs containing a modeled field therefore replaces the field
with an empty run, after which the normalized acceptance postcondition fails.
Unchanged modeled fields must remain usable as unrelated paragraph content.

### D2, final paragraph markers can be injected into run properties
`crates/rdocx/src/comparison.rs:236`

`mark_previous_paragraph` tests the complete serialized paragraph for
`</w:rPr>` and injects before the last matching close tag. If the paragraph has
a formatted run, that close tag belongs to the run rather than to
`w:pPr/w:rPr`. Inserting or deleting a final paragraph after such an anchor
therefore creates a run contextual marker instead of a paragraph-mark revision,
so accept or reject resolves the wrong owner and the comparison postcondition
fails.

### D3, row markers can be injected into nested table rows
`crates/rdocx/src/comparison.rs:781`

`marked_row` searches the complete outer row XML and injects before the last
`</w:trPr>`. When a cell contains a nested table row with properties, the last
match belongs to that nested row, even when the outer row also has properties.
Whole-row insertion or deletion then marks the nested row instead of the owner
row, breaking exact accepted and rejected table structure.

### D4, whole-table marking skips rows owned by content controls
`crates/rdocx/src/comparison.rs:762`

`marked_table` marks only `table.rows`. Rows held by table-level content
controls live in `table.content_controls`, so they remain unmarked during a
whole-table insertion or deletion. The resolver now correctly treats those as
owned rows, which means the table cannot disappear on the removing resolution
and the postcondition rejects a whole-table change that the approved scope
includes.

### D5, content-control block replacement moves preserved whitespace
`crates/rdocx/src/comparison.rs:969`
`crates/rdocx/src/comparison.rs:1236`

Whitespace slots are emitted only when an alignment entry has an original
left index. Paragraph-to-table expansion emits the inserted table first and
the original paragraph second, so whitespace that originally preceded the
paragraph is emitted after the inserted table. Acceptance filters the deleted
paragraph and leaves the raw bytes on the opposite side of the replacement.
The normalized postcondition ignores whitespace, so this positional
preservation failure succeeds unnoticed.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 fixes were confirmed for typed tab conversion, recursive table-row
cleanup, Word-namespace property-shell cleanup, nested table formatting
diagnostics, and ordinary paragraph-table replacement before an anchor. No
additional defects were found in metadata escaping, revision-id allocation,
timestamp validation, mutation atomicity, public API exposure, deterministic
LCS tie-breaking, schema child order, panic safety, or structural discipline.
