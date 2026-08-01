# F-074, all, pass 4

**Reviewed**: settled working diff from claim base `4450afb`, 5 implementation
and HLD files with 1,805 added lines and 2 removed lines. The pass 1 through
pass 3 reviews and local `corpus` symlink are workflow artifacts outside the
feature line count.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, unequal edit and deletion counts remain ambiguous for grid metadata
`crates/oxml-drawing/src/table.rs:809`

The ambiguity check rejects only when both the unmatched current count and the
unmatched original count exceed one. Parse two metadata-bearing columns with
widths 100 and 200, delete one column, then edit the surviving width to 150.
The unmatched counts are one and two, so serialisation succeeds and assigns
the first original column's metadata. The same public state can instead mean
that the first column was deleted and the second was edited, which requires the
second metadata. Insertion plus edit has the inverse one-to-many ambiguity.
The writer must return the typed ambiguity error whenever more than one
metadata association remains possible, including these unequal-count cases.

### D2, grid and row equality still expose reconciled raw-child boundaries
`crates/oxml-drawing/src/table.rs:123`
`crates/oxml-drawing/src/table.rs:165`

`CT_TableGrid` and `CT_TableRow` are public equality-bearing child models, but
their equality implementations compare `OrderedRawChildren` storage directly.
After a grid column or row cell is removed, their writers reconcile an original
raw sibling to its new boundary. Reparse records that effective boundary, so
the edited child and its reparsed counterpart compare unequal even though they
serialise identically. The collection-edit regression at
`crates/oxml-drawing/src/table.rs:1382` checks only whole-table equality, whose
canonical serialisation masks both child-level failures. Grid and row equality
need the same canonical treatment now used by table properties and cells.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in merge modelling, schema child order, fixed-prefix
output, namespace stack push and pop behaviour, inherited namespace
completion, XML NCName Unicode validation, optional property and cell child
equality, stable row and cell origin reconciliation, opaque subtree
preservation, panics on untrusted input, test-gate reversion, contract scope,
or structure. The three exact pass 3 regression cases pass in the focused
`oxml-drawing` test run.
