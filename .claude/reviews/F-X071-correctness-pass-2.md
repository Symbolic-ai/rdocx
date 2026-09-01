# F-X071, correctness, pass 2

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through working `HEAD` `9a880fe2c084ec6ee87d6dc17560407b602eb9ae`, 19 implementation files and 3,276 changed lines (3,109 additions, 167 deletions)
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, the namespace fix drops foreign document-background bytes
`crates/rdocx-oxml/src/document.rs:894`

The parser now correctly refuses to type a foreign `background` lookalike, but
the fallback consumes a start element with `read_to_end_into` and does not store
it. The empty-element branch at `crates/rdocx-oxml/src/document.rs:898` also has
no raw fallback. Opening and saving a document containing
`<ext:background ext:color="red"/>` therefore deletes that producer subtree.
The regression at `crates/rdocx-oxml/src/document.rs:1079` checks only that the
typed field is absent. It does not serialize or reopen the document to prove
the unmodelled bytes survive. A foreign same-local element must remain untyped
and preserved.

### D2, picture relationships are accepted outside the schema-owned graphic payload
`crates/rdocx-oxml/src/drawing.rs:1068`

`image_relationship_ids` starts picture tracking at any `pic:pic` anywhere in
the captured inline or anchor subtree. It does not require the ancestor path
`a:graphic/a:graphicData`, the picture graphic-data URI, or the direct
`graphicData` child position that the writer emits at
`crates/rdocx-oxml/src/drawing.rs:1182`. As a result, a producer extension or
other payload containing a correctly namespaced
`pic:pic/pic:blipFill/a:blip` is exposed as the drawing's image relationship.
The positive regression at `crates/rdocx-oxml/src/drawing.rs:1472` itself puts
`pic:pic` directly under `wp:inline` and therefore locks in a schema-invalid
path instead of proving the picture payload identity.

### D3, undeclared conventional drawing prefixes still acquire namespace semantics
`crates/rdocx-oxml/src/drawing.rs:1165`

`namespace_matches` treats `ResolveResult::Unknown` as the requested namespace
whenever the raw prefix spelling is conventional. An undeclared `pic:pic`,
`a:blip`, or `r:link` can therefore satisfy the new picture detector even
though it has no expanded name. This is reachable through public
`CT_Drawing::from_xml`, whose default scope does not supply the `a` or `pic`
bindings. Foreign bound shadows are rejected, but malformed unbound fixed
prefixes are incorrectly promoted to typed image facts rather than failing
closed.

### D4, duplicate picture payloads are merged into one relationship fact
`crates/rdocx-oxml/src/drawing.rs:1059`

The helper accumulates one `embed_id` and one `link_id` across the entire
subtree and never records that a picture or blip has already supplied a
relationship. After the first `pic:pic` closes, a second sibling picture can
overwrite one value while leaving the other value from the first picture. Two
`a:blip` children inside one `pic:blipFill` behave the same way. The helper
returns the combined pair at `crates/rdocx-oxml/src/drawing.rs:1134`, so
structurally ambiguous input is published as one image relationship instead of
remaining unsupported. No duplicate or ambiguity regression exercises this
path.

### D5, foreign row-property XML is dropped from RTF without a loss diagnostic
`crates/rdocx/src/rtf.rs:411`

The corrected row parser now stores foreign `ins` and `del` lookalikes in
`CT_TrPr::extra_xml` at `crates/rdocx-oxml/src/table.rs:1019`. The RTF row
property scanner reports typed revisions and `revision_xml`, but never reports
`extra_xml`. Commit `9a880fe` removed the old revision-XML expectation from
`crates/rdocx/tests/integration_test.rs:1274` without adding the generic raw
row-property diagnostic. Exporting the test's `<ext:del/>` now drops a retained
source item silently, contrary to the RTF facade contract that every
unrepresentable input carries a loss diagnostic.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-1 remediation**: Direct `CT_Inline::from_xml` and
  `CT_Anchor::from_xml` now fail closed for relationship attributes. Extreme
  numbering levels no longer overflow. Foreign `CT_TrPr` lookalikes retain
  their observed serialization boundaries. Bitmap fills inside `wps:wsp` no
  longer classify that shape as a picture. Foreign document backgrounds no
  longer acquire the typed background identity, apart from the preservation
  loss in D1.
- **Correctness and contract**: No additional issue was found in default-style
  numbering association, direct numbering overrides, `numId=0`, narrowed
  unmodelled numbering facts, revision classification, or complex-field source
  ordering.
- **Panics and bounds**: No new panic, unchecked arithmetic, recursive depth,
  or malformed revision XML issue was found.
- **OOXML and preservation**: Apart from D1 to D4, inherited table namespace
  scope, row-property schema order, foreign prefix shadows, retained raw table
  subtrees, and repeated serialization showed no issue.
- **Tests**: `cargo test -p rdocx-oxml` passed 325 unit tests and one doctest.
  `cargo test -p rdocx --lib --quiet` passed 326 tests with three ignored. The
  focused RTF integration test passed, demonstrating the silent-loss
  expectation described in D5. `git diff --check` passed.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, or dynamic dispatch violation was found.
