# F-074, all, pass 1

**Reviewed**: working diff from claim base `4450afb`, 4 feature files and
1,116 added lines. The untracked `corpus` symlink is local test plumbing and
contains no feature diff.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, public collection edits can drop or misattach preserved XML

`crates/oxml-drawing/src/table.rs:33`

The schema collections are public vectors, while their raw metadata is stored
privately by positional index. The grid writer looks up `column_raw` by the
current column index at `crates/oxml-drawing/src/table.rs:431`, and table and row
raw children are emitted from current row and cell indices at
`crates/oxml-drawing/src/table.rs:191` and
`crates/oxml-drawing/src/table.rs:558`. After parsing a row with two cells and
an `a:extLst` after them, `row.cells.remove(0)` leaves the extension at boundary
two, but the writer now visits only boundaries zero and one, so the extension
is dropped. Removing or reordering grid columns similarly attaches one
column's unsupported attributes and children to another width. This violates
the preservation contract through operations that the public fields allow.

### D2, extracted tables lose inherited namespace bindings for opaque XML

`crates/rpptx-oxml/tests/integration.rs:252`

The corpus path captures only the `a:tbl` subtree, so namespace declarations on
the containing slide or graphic data are not included in the bytes passed to
`CT_Table::from_xml`. The table writer then declares only `xmlns:a` at
`crates/oxml-drawing/src/table.rs:180`. A valid slide that declares `xmlns:x`
on `p:sld` and uses `<x:extension/>` inside a table therefore serialises to a
self-contained table with an unbound `x` prefix. The focused preservation
fixture declares `x` directly on the table, and quick-xml structural reparsing
does not validate namespace bindings, so the current tests do not expose the
invalid output.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML child order,
test-gate reversion, or structure.
