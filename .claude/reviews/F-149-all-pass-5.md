# F-149, all, pass 5

**Reviewed**: remediated working tree against base `28bdbbc`, 17 implementation files and 1,644 changed lines, including 681 lines in the two approved untracked source modules
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, foreign border children are decoded before their namespace is accepted
`crates/rdocx-oxml/src/borders.rs:152`
`crates/rdocx-oxml/src/table.rs:95`
`crates/rdocx-oxml/src/revision.rs:523`

The paragraph and table border readers build a `CT_BorderEdge` before checking
whether the child element is in the WordprocessingML namespace. A foreign
`x:top` carrying a Word-prefixed numeric attribute such as `w:sz="invalid"`
therefore returns a numeric parse error instead of being ignored as a foreign
child. This can make an otherwise projectable prior paragraph or table property
revision fail the whole document parse. The paragraph collision regression
uses only foreign-prefixed attributes on the foreign child, so it does not
exercise this parse-after-element-match case.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-4 paragraph-border projection defect is otherwise fixed: scoped PPr
parsing calls the namespace-aware `CT_PBdr` reader, and accepted border
attributes are namespace-checked. All earlier content-control traversal,
metadata decoding, direct namespace collision, section parse-after-match,
schema-order, exact raw preservation, ordering, facade, panic, contract, and
structure findings remain fixed. The focused namespace regression passes. No
additional correctness, OOXML, or test findings were found.
