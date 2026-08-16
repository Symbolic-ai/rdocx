# F-156, all, pass 1

**Reviewed**: working diff, 31 changed paths, 16,156 added lines and 16,023 deleted lines including the mechanical crate move
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, the shared README advertises an unimplemented Word facade path

`crates/oxml-chart/README.md:8`

The migration guidance tells users to choose `rdocx` for charts inside a
complete document, but F-157 and F-158 have not added the Word chart part or
facade API yet. A current `rdocx` consumer cannot follow that guidance. Keep
the README on the shared engine and the implemented `rpptx` facade until the
Word-side stories land.

## Smells

### S1, packaged unit tests reach outside the published crate

`crates/oxml-chart/src/lib.rs:15057`

The publication-candidate test reads the workspace root and three sibling
crate manifests through `include_str!`. Those paths are absent from the
six-file `oxml-chart` archive, so the package does not carry self-contained
unit-test sources. Keep local manifest assertions in the crate test and move
workspace ownership, active-consumer, and shim dependency assertions to the
repository-level release regression module.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic-safety, OOXML, test-gate, or
structural findings were found. The implementation body differs from the old
owner only in crate-derived labels, the owner assertion, and the namespace
documentation. Parser and serializer ordering and raw-subtree behavior remain
unchanged.
