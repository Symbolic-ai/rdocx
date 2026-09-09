# F-247, default, pass 4

**Reviewed**: uncommitted working tree diff, 20 files, 4,429 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Instance removal still treats opaque XML data as a live reference
`crates/rdocx/src/document.rs:8968`
`crates/rdocx/src/document.rs:9167`

The pass-3 fix limits whole-graph validation to the main document and its
relationship-resolved Word stories, but `remove_numbering_instance` still uses
the older package-wide reference scan. That scan examines every well-formed XML
part and accepts any namespace-correct `w:numId`, regardless of whether the
part is a Word story. A preserved custom XML part containing a data-shaped
`w:numId` equal to an otherwise unreferenced instance therefore makes removal
return an error. The new opaque-custom-XML regression exercises only
`update_numbering_instance`, so it does not detect this remaining false
rejection. Removal needs the same owner-aware scope, while retaining any
additional typed owner such as the styles part that can legally reference a
numbering instance.

## Smells

None.

## Nitpicks

None.

## Pass-3 finding rechecked

- D1 is fixed for whole-graph validation. Story selection now starts with the
  main document and follows only the supported internal story relationships at
  `crates/rdocx/src/document.rs:9111`. The custom XML regression is at
  `crates/rdocx/src/document.rs:22056`. D1 above records the sibling removal
  path that still uses package-wide scanning.

## Earlier findings rechecked

- Style-inherited and related-story paragraph levels resolve before graph
  validation at `crates/rdocx/src/document.rs:9144`.
- `w:multiLevelType` raw preservation remains active at
  `crates/rdocx-oxml/src/numbering.rs:3285` and
  `crates/rdocx-oxml/src/numbering.rs:3419`.
- RTF standard formats and marker suppression remain mapped at
  `crates/rdocx/src/rtf.rs:1235`.
- `ListLevel` semantic equality continues to exclude preservation provenance at
  `crates/rdocx/src/document.rs:13480`.
- Modelled scalar-leaf payloads, sparse level identities, and typed attribute
  conflict preflight remain covered at
  `crates/rdocx-oxml/src/numbering.rs:2679`,
  `crates/rdocx/src/document.rs:8823`, and
  `crates/rdocx/src/document.rs:14126`.

## Checks run

- `git diff --check`, passed.
- `cargo fmt --all --check`, passed.
- `cargo test -p rdocx numbering --lib`, 28 passed with an isolated target
  directory.

## Not found

No independent panic, OOXML child-order, preservation, public equality,
render, RTF, ODT, EPUB, atomicity, structure, smell, or nitpick findings were
found. The remaining correctness, contract, package-scope, and test-sensitivity
issue is D1.
