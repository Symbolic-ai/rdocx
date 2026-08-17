# F-149, all, pass 3

**Reviewed**: remediated working tree against base `28bdbbc`, 16 implementation files and 1,464 changed lines, including 652 lines in the two approved untracked source modules
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested prior border and margin properties remain namespace-blind
`crates/rdocx-oxml/src/properties.rs:174`
`crates/rdocx-oxml/src/table.rs:523`

Paragraph and table prior-property projections now namespace-check their direct
children, but they still delegate `w:pBdr`, `w:tblBorders`, and
`w:tblCellMar` to parsers that select nested elements and attributes by local
name alone. A foreign `x:top` inside one of these Word containers is therefore
exposed as a Word border or margin. The typed prior state can still disagree
with the preserved XML despite the direct collision fixes.

### D2, foreign section attributes can suppress an otherwise valid revision
`crates/rdocx-oxml/src/document.rs:204`

The `w:pgMar` projection parses every attribute value as an integer before it
checks whether the attribute belongs to the WordprocessingML namespace. A
foreign producer attribute with a nonnumeric value therefore makes prior
section parsing fail. `CT_Revision::from_raw` then drops the typed projection,
so a revision with valid identity and author disappears from
`Document::revisions` instead of retaining its metadata and ignoring the
unmodelled attribute.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-1 content-control traversal and escaped-metadata defects remain fixed.
The pass-2 direct foreign-element and foreign-attribute cases are fixed and
their focused regression passes. The full `rdocx-oxml` and `rdocx` test suites
also pass. No additional correctness, contract, panic, OOXML ordering, test, or
structure findings were found.
