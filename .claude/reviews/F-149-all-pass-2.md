# F-149, all, pass 2

**Reviewed**: remediated working tree against base `28bdbbc`, 16 implementation files and 1,303 changed lines, including 649 lines in the two approved untracked source modules
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, prior table and section projections still accept foreign attributes
`crates/rdocx-oxml/src/table.rs:447`
`crates/rdocx-oxml/src/document.rs:192`

The pass-1 namespace fix now checks the namespace of table and section child
elements, but their attributes are still selected by local name alone. A Word
`w:jc` carrying only `x:val="center"` is therefore exposed as a centered prior
table property, and a Word `w:pgSz` carrying `x:w` and `x:h` is exposed as prior
section dimensions. Foreign same-local-name attributes must not populate the
WordprocessingML projection.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-1 content-control traversal and escaped-metadata defects are fixed.
No additional correctness, contract, panic, OOXML ordering, test, or structure
findings were found. The approved modules add no trait, generic parameter,
crate, dependency, or feature flag.
