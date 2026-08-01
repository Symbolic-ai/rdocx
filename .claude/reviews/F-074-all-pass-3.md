# F-074, all, pass 3

**Reviewed**: settled working diff from claim base `4450afb`, 5 implementation
and HLD files with 1,641 added lines and 2 removed lines. The pass 1 and pass 2
reviews and local `corpus` symlink are workflow artifacts outside the feature
line count.
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, combined grid edits still misattach preserved column metadata
`crates/oxml-drawing/src/table.rs:698`

Column identity is still inferred from the public `Vec<Emu>` values. Parse
columns `100` with metadata A and `200` with metadata B, swap the columns, then
edit the moved B width to `100`. The current values are now `[100, 100]`, so the
same-position pass matches the first value to original A and the fallback gives
B to the second value. The writer therefore undoes the metadata swap and emits
A on the edited B column. Inserting a new `100` column before an original `100`
column has the same problem. The simple collision regression does not combine
reordering or insertion with an edit, so pass 2 D1 is not fully remediated.

### D2, public child equality still exposes stale optional-child boundaries
`crates/oxml-drawing/src/table.rs:42`
`crates/oxml-drawing/src/table.rs:143`

Whole-table equality now compares canonical serialisations, but
`CT_TableProperties` and `CT_TableCell` still compare `OrderedRawChildren`
directly. After clearing `style_id`, a raw child originally after the style
keeps boundary one in the edited value and reparses at boundary zero. After
clearing `text_body`, raw children originally around the text body similarly
collapse to the pre-properties boundary on reparse. The whole-table assertions
at `crates/oxml-drawing/src/table.rs:1339` and
`crates/oxml-drawing/src/table.rs:1344` mask both inequalities. Comparing the
public edited property or cell value to its reparsed counterpart still returns
false, so pass 2 D3 is not fully remediated.

### D3, valid Unicode inherited namespace prefixes are rejected
`crates/oxml-drawing/src/table.rs:352`

XML namespace prefixes are NCNames and may contain non-ASCII letters. The
corpus extractor accepts such prefixes as UTF-8 strings, but
`namespace_attribute_name` validates their bytes with ASCII-only predicates.
A valid ancestor binding such as `xmlns:é="urn:producer"` used by opaque
table content therefore makes `from_xml_with_inherited_namespaces` return an
invalid-prefix error instead of producing namespace-complete standalone XML.
The namespace tests cover ASCII prefixes and the `a` shadow case only.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in merge modelling, schema child order, fixed-prefix
output, typed-prefix conflict handling, inherited `a` shadowing, namespace
stack push and pop behaviour, row and cell stable-origin reconciliation,
opaque subtree preservation outside the cases above, panics on untrusted
input, test-gate reversion, contract scope, or structure.
