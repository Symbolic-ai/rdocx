# F-149, all, pass 4

**Reviewed**: remediated working tree against base `28bdbbc`, 17 implementation files and 1,600 changed lines, including 665 lines in the two approved untracked source modules
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, prior paragraph borders remain namespace-blind
`crates/rdocx-oxml/src/properties.rs:174`
`crates/rdocx-oxml/src/borders.rs:144`

The pass-3 table border and margin paths now use scoped namespace-aware
readers, but `w:pBdr` in a prior paragraph-property projection still delegates
to `CT_PBdr::from_xml`. That parser selects nested edges and their attributes
by local name alone. A foreign `x:top` inside `w:pBdr` is therefore still
exposed as a Word paragraph border, leaving the pass-3 paragraph half of the
namespace-collision finding unresolved.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-3 nested table border and margin defect is fixed. Section numeric
attributes are now parsed only after a Word namespace match, so foreign
nonnumeric attributes no longer suppress a valid revision. All earlier
content-control traversal, metadata decoding, direct namespace collision, and
schema-order fixes remain intact. The focused namespace regression and scoped
clippy checks pass. No additional correctness, contract, panic, OOXML, test, or
structure findings were found.
