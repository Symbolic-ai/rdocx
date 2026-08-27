# F-X066, working, pass 1

**Reviewed**: working diff against claim Base
`3ddac3a3420eda6dc25abd9c5b1dce5721725834`, 3 files, 268 insertions and
4 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, inherited namespace bindings never reach the raw-item classifier

`crates/rdocx/src/run.rs:219`

The classifier creates a fresh `NsReader` from the captured child bytes, so it
can resolve only namespace declarations repeated directly inside that raw
subtree. The OXML run parser stores an unknown child with `capture_element` at
`crates/rdocx-oxml/src/text.rs:637`, which does not carry namespace bindings
declared only on the document, paragraph, or run ancestors. A normal form such
as `<w:pict><v:rect o:hr="t"/></w:pict>` under ancestor declarations therefore
resolves its prefixes as unknown and remains `UnsupportedXml` instead of
becoming `LegacyHorizontalRule`. The package regression avoids this case by
redeclaring all three namespaces directly on `w:pict` at
`crates/rdocx/tests/regression_test.rs:1898`. This violates the plan's required
classification from in-scope namespace URI facts. The parser must retain or
thread the inherited namespace context, or classify at the existing OXML parse
boundary, and the package regression must cover bindings that exist only on
ancestors.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness beyond D1: the accepted structure contains exactly one VML
  rectangle, requires the Office horizontal-rule attribute, and accepts only
  the specified `t` and `true` lexical forms.
- Contract beyond D1: unsupported, malformed, ambiguous, numeric, false,
  foreign-namespace, multiple-child, comment, and visible-content forms remain
  unsupported raw XML.
- Public compatibility: the new accessor and non-exhaustive enum variant are
  additive, documented, and preserve access to the exact captured bytes.
- OOXML beyond D1: serialization still uses the original raw subtree in its
  original run-item position, with no modeled rewrite or schema-order change.
- Layout and rendering: no new layout or renderer consumer was introduced.
- Panics and errors: no new production `unwrap`, `expect`, unchecked indexing,
  slicing, or arithmetic occurs on untrusted package input.
- Tests beyond D1: positive lexical aliases, negative namespace and structure
  cases, raw item order, package save and reopen, the pinned Word corpus, and
  the unchanged hash gate are represented in the implementation evidence.
- HLD scope: completion has not started, so no HLD file has been changed before
  the plan-listed completion step.
- Structure: no new module, file, test binary, trait, generic parameter,
  dependency, or forwarding-only wrapper was introduced.
