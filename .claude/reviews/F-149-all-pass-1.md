# F-149, all, pass 1

**Reviewed**: working tree against base `28bdbbc`, 15 files and 1,187 changed lines, including 592 lines in the two approved untracked source modules
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, revision wrappers directly inside content controls are not reported
`crates/rdocx-oxml/src/content_control.rs:579`
`crates/rdocx-oxml/src/revision.rs:312`

A valid `w:ins`, `w:del`, `w:moveFrom`, or `w:moveTo` directly inside a
run-level `w:sdtContent` is stored as `SdtContent::RawXml`. Revision traversal
then ignores every raw content-control child. The subtree round-trips, but
`Document::revisions` omits the revision, contrary to the approved recursive
content-control traversal contract.

### D2, escaped revision metadata is exposed without XML decoding
`crates/rdocx-oxml/src/revision.rs:420`

Revision attributes are copied from the raw attribute bytes without entity
decoding. An author such as `A &amp; B` is therefore reported by `author()` as
`A &amp; B` instead of the XML value `A & B`. The same issue affects an escaped
date value. Raw serialization remains correct, but the typed metadata
projection is wrong.

### D3, prior table and section projections accept foreign same-local-name properties
`crates/rdocx-oxml/src/table.rs:446`
`crates/rdocx-oxml/src/document.rs:187`

The new prior-property projections compute namespace bindings, but their table
and section parsers still select ordinary children and attributes by local name
alone. For example, a foreign `<x:jc x:val="center"/>` inside
`w:tblPrChange` is exposed as a Word table alignment, and a foreign
`<x:titlePg/>` inside `w:sectPrChange` is exposed as a Word section property.
This violates the namespace-collision requirement and makes the typed prior
state disagree with the preserved XML.

## Smells

None.

## Nitpicks

None.

## Not found

Panics and structure produced no findings. The approved modules add no trait,
generic parameter, crate, dependency, or feature flag. The existing tests
cover the declared happy-path round-trip and ordering cases, but do not cover
the defect inputs above.
