# F-071, all, pass 1

**Reviewed**: Uncommitted F-071 working tree, 6 files, 926 insertions and 20 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, standalone placeholders can lose fixed namespace declarations
`crates/rpptx-oxml/src/placeholder.rs:207`

The self-contained writer declares only `xmlns:p`, while parsing removes local
declarations for every prefix in `FIXED_SHAPE_TREE_PREFIXES`. A placeholder
whose opaque child uses a locally declared `a:`, `r:`, or `mc:` prefix therefore
serialises that raw child without its required namespace binding. For example,
an alternate-prefix `p:ph` with `xmlns:a` and an opaque `a:ext` child produces
an invalid standalone fragment instead of preserved XML.

### D2, shapes extracted from an alternate-prefix tree are not self-contained
`crates/rpptx-oxml/src/shape_tree.rs:245`

`CT_Shape::from_fragment` resolves prefixes inherited from the containing tree,
but `CT_Shape` does not retain those inherited bindings. Its writer declares
only the fixed `p:`, `a:`, `r:`, and `mc:` prefixes, then writes `p:cNvPr`,
`p:cNvSpPr`, and `p:spPr` as their original raw bytes. A shape parsed from a
tree that binds PresentationML as `q:` on an ancestor therefore emits raw
`q:` children with no `xmlns:q` when the public `shape.to_xml()` method is
called. The result contradicts the method's self-contained contract and is not
a namespace-valid standalone shape.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, matching-contract, panic, schema-order,
test-gate, or structural findings were found. The matching tests cover index
priority, type fallback, the absent-type default, and both required equivalence
classes. No production `unwrap`, input indexing, new trait, generic parameter,
feature flag, crate, or forwarding wrapper was introduced by F-071.
