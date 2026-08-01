# F-074, all, pass 2

**Reviewed**: working diff from claim base `4450afb`, 5 implementation and HLD
files with 1,465 added lines and 2 removed lines. The pass 1 review and local
`corpus` symlink are workflow artifacts outside the feature line count.
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, a width edit can steal another column's preserved metadata
`crates/oxml-drawing/src/table.rs:574`

The metadata matcher assigns every exact width match before considering the
same-position fallback. Parse columns `100` with metadata A and `200` with
metadata B, then edit only the first public width to `200`. The first column
claims B, the unchanged second column receives no metadata, and A is dropped.
The existing edit regression changes `200` to a unique `250`, so it does not
exercise this collision. This leaves pass 1 D1 unresolved for a normal public
collection edit.

### D2, removing an earlier cell moves raw siblings past a surviving cell
`crates/oxml-drawing/src/table.rs:715`

For `cell-1, raw-child, cell-2`, the raw child is stored at boundary one.
After `cells.remove(0)`, the writer emits the surviving `cell-2` and then
boundary one, changing the order to `cell-2, raw-child`. The mutation test at
`crates/oxml-drawing/src/table.rs:1133` checks only that the raw child still
exists, not that it remains before the surviving cell. The new trailing fold
prevents loss but does not preserve the subtree in place, so pass 1 D1 is not
fully remediated.

### D3, optional-child edits do not compare equal after round-trip
`crates/oxml-drawing/src/table.rs:100`

Raw-child equality normalises only collection boundaries. Parse a table with
`a:tblPr`, a raw child after it, and `a:tblGrid`, then set the public
`properties` field to `None`. The writer emits the raw child before the grid,
and reparsing records it at boundary zero instead of boundary one. The original
edited table and reparsed table therefore compare unequal. The derived equality
on `CT_TableProperties` at `crates/oxml-drawing/src/table.rs:41` has the same
problem after clearing `style_id`, and `CT_TableCell` at
`crates/oxml-drawing/src/table.rs:67` has it after clearing `text_body`. These
are valid edits through the public contract, but the structural round-trip
promise does not hold.

### D4, a locally shadowed `a` binding is rejected as an inherited conflict
`crates/oxml-drawing/src/table.rs:172`

An ancestor may bind `a` to a producer namespace while an extracted table
locally rebinds `a` to the DrawingML namespace. The local declaration is the
effective binding and `from_xml` accepts it, but
`from_xml_with_inherited_namespaces` later rejects the shadowed ancestor entry
without checking the table-root declaration. The corpus extractor passes all
ancestor bindings at `crates/rpptx-oxml/tests/integration.rs:271`, so this valid
namespace stack fails instead of producing self-contained output. The new
namespace regression covers an inherited DrawingML alias and producer prefix,
but not local shadowing of an ancestor `a` declaration.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in schema child order, fixed-prefix handling outside
the inherited-shadow case, opaque subtree byte preservation without public
collection edits, panics on untrusted input, test-gate reversion, contract
scope, or structure.
