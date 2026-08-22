# F-167, all, pass 1

**Reviewed**: complete working diff, 4 files, 2,010 additions and 6 deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, content-control comparison drops preserved whitespace
`crates/rdocx/src/comparison.rs:1016`
`crates/rdocx/src/comparison.rs:1312`

`modeled_control_content` removes whitespace-only `RawXml` entries, then the
comparison rebuilds the complete `w:sdtContent` inner XML exclusively from that
filtered list. A content control whose modeled text changes and whose source is
pretty-printed with whitespace between children therefore loses those raw
bytes. The normalized postconditions filter the same entries, so the operation
succeeds despite violating the byte-for-byte preservation contract.

### D2, deleted run conversion corrupts `w:tab`
`crates/rdocx/src/comparison.rs:1381`

The prefix replacement from `<w:t` to `<w:delText` also rewrites `<w:tab/>` as
`<w:delTextab/>`. Any changed or deleted run that contains a tab therefore
produces a foreign element instead of retaining the tab inside the deletion.
The reject postcondition then differs from the original and rejects an ordinary
modeled text edit that includes a tab.

### D3, formatting changes inside matched table rows are not diagnosed
`crates/rdocx/src/comparison.rs:706`
`crates/rdocx/src/comparison.rs:1239`

`row_signature` deliberately excludes row, cell, paragraph, and run formatting.
When two rows have equal content signatures, `compare_table` checks only the row
properties and copies the original serialized row without descending into its
cells. A formatting-only change to cell properties or to a paragraph or run in
a cell therefore returns no diagnostic, even though the contract requires a
stable diagnostic for non-structural formatting differences.

### D4, unmatched LCS slices pair incompatible body structures
`crates/rdocx/src/comparison.rs:1165`
`crates/rdocx/src/comparison.rs:313`

Between LCS matches, the aligner pairs entries by position without considering
their structure kind. Replacing a paragraph with a table before an unchanged
anchor paragraph is therefore emitted as a paragraph-table pair and rejected
as an unlike body structure. The same change is representable as a paragraph
deletion plus a marked table insertion, and it falls within the approved exact
paragraph and table structure scope.

### D5, empty-shell cleanup ignores the owner namespace
`crates/rdocx/src/revision.rs:441`

The cleanup selects `pPr`, `rPr`, `trPr`, and `numPr` by local name without
requiring the owner element to be in the WordprocessingML namespace. Resolving
a selected Word revision nested below an otherwise empty foreign element such
as `x:rPr` therefore deletes the foreign owner. This violates the requirement
to preserve unmodelled namespace-qualified XML.

### D6, table cleanup ignores rows inside retained content controls
`crates/rdocx/src/revision.rs:656`

`direct_rows_all_remove` considers only direct `w:tr` children. If every direct
row has a selected deletion marker but a retained `w:sdt` child contains an
unselected row, the predicate still removes the complete table. Accepting the
direct row deletion consequently drops the content control and its retained
row, contrary to recursive content-control ownership and scoped revision
resolution.

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in metadata escaping, revision-id allocation,
timestamp validation, mutation atomicity, public API exposure, deterministic
LCS tie-breaking, schema child order, panic safety, or structural discipline.
