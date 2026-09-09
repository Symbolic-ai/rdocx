# F-247, default, pass 3

**Reviewed**: uncommitted working tree diff, 20 files, 4,354 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Package-wide paragraph scanning treats opaque XML data as live Word content
`crates/rdocx/src/document.rs:9112`
`crates/rdocx/src/document.rs:14227`

The new live-reference validator sends every well-formed package part through
the paragraph scanner. The scanner then accepts any element bound to the
WordprocessingML namespace and named `w:p`, without first proving that its part
is a relationship-resolved Word story. A preserved custom XML part can
legitimately contain such data without representing document content. If that
opaque data includes `w:numPr`, `validate_numbering_graph` treats it as a live
reference. An unchanged graph can therefore fail validation because of an
unrelated identifier, or moving a real numbering instance can be rejected
because the opaque part names a level absent from the target definition. The
scan must be limited to actual Word story owners while retaining the new
style-inheritance resolution within those owners.

## Smells

None.

## Nitpicks

None.

## Pass-2 findings rechecked

- D1 is fixed for style-inherited and related-story paragraphs by the effective
  numbering projection at `crates/rdocx/src/document.rs:9118`. D1 above records
  the new package-scope overreach in that fix.
- D2 is fixed. `w:multiLevelType` captures its complete raw occurrence at
  `crates/rdocx-oxml/src/numbering.rs:3285` and replays or overlays it at
  `crates/rdocx-oxml/src/numbering.rs:3419`.
- D3 is fixed. RTF export maps every standard typed value and emits no marker
  for producer-defined values at `crates/rdocx/src/rtf.rs:1235`, with the
  focused export regression at `crates/rdocx/src/rtf.rs:1609`.
- D4 is fixed. `ListLevel` semantic equality excludes `source_level` at
  `crates/rdocx/src/document.rs:13454`, with the provenance regression at
  `crates/rdocx/src/document.rs:21679`.

## Pass-1 findings rechecked

- Modelled `CT_Lvl` scalar leaves retain producer payload through the raw
  sidecars beginning at `crates/rdocx-oxml/src/numbering.rs:2679`.
- Typed level-attribute conflicts fail before facade publication at
  `crates/rdocx/src/document.rs:14100`.
- Sparse imported level identifiers remain explicit and are matched by
  identifier during updates at `crates/rdocx/src/document.rs:8823`.
- Direct main-body level references remain covered by whole-graph validation
  at `crates/rdocx/src/document.rs:9079`.

## Checks run

- `git diff --check`, passed.
- `cargo fmt --all --check`, passed.
- `cargo test -p rdocx-oxml numbering --lib`, 62 passed with an isolated target
  directory.
- `cargo test -p rdocx numbering --lib`, 27 passed with an isolated target
  directory.

## Not found

No independent panic, OOXML child-order, preservation, public equality, RTF,
ODT, EPUB, atomicity, structure, smell, or nitpick findings were found. The
remaining correctness, contract, and test-sensitivity issue is D1.
