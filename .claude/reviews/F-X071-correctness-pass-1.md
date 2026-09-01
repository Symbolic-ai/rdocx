# F-X071, correctness, pass 1

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through working `HEAD` `53507fa`, 19 implementation files and 3,049 changed lines (2,932 additions, 117 deletions)
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, a foreign background lookalike becomes a document-background fact
`crates/rdocx-oxml/src/document.rs:892`

`CT_Document::from_xml` captures any element whose local name is `background`
without checking that its expanded name is WordprocessingML. The new facade then
reports every captured value as a real page background at
`crates/rdocx/src/document.rs:2336`. A document containing a direct
`<ext:background xmlns:ext="urn:producer"/>` child therefore makes
`has_document_background()` return `true`, even though the foreign child has no
Word background semantics. The added test at
`crates/rdocx/src/document.rs:7049` assigns `background_xml` directly and cannot
detect this parser boundary error.

### D2, the public inline and anchor parsers accept foreign relationship attributes
`crates/rdocx-oxml/src/drawing.rs:525`
`crates/rdocx-oxml/src/drawing.rs:662`
`crates/rdocx-oxml/src/drawing.rs:921`
`crates/rdocx-oxml/src/drawing.rs:936`

The newly exposed `link_id` fact is populated by local attribute name in both
public `CT_Anchor::from_xml` and public `CT_Inline::from_xml`. Direct callers can
therefore parse `<a:blip xmlns:ext="urn:producer" ext:link="rIdBad"/>` and receive
`Some("rIdBad")`. `CT_Drawing::from_xml` repairs the result afterward with the
namespace-aware scan at `crates/rdocx-oxml/src/drawing.rs:1308` and
`crates/rdocx-oxml/src/drawing.rs:1337`, but that does not repair the public
nested parser contracts. Foreign same-local attributes must remain untyped on
every public parse path.

### D3, an untrusted numbering level can panic the new reader projection
`crates/rdocx/src/document.rs:2925`

`has_list_paragraph_presentation` uses
`level.saturating_add(1) * 720`. The addition saturates, but the multiplication
is unchecked. The numbering parser accepts any `u32` `w:ilvl` at
`crates/rdocx-oxml/src/numbering.rs:2562`. A definition with
`w:ilvl="4294967295"` and a `w:pPr`, followed by
`numbering_level(num_id, u32::MAX)`, panics from integer overflow in a debug
build. In a release build it wraps before the `i32` cast and compares against a
nonsensical standard indentation. Parsed package data must not make this public
read-only fact panic.

### D4, foreign row revision lookalikes are moved to Word revision slots
`crates/rdocx-oxml/src/table.rs:1019`
`crates/rdocx-oxml/src/table.rs:1056`

Foreign `ins` and `del` children are distinguished from Word revision elements,
but they are still stored in `revision_xml` and assigned the fixed Word slots by
`row_revision_raw_slot` at `crates/rdocx-oxml/src/table.rs:1227`. The serializer
then emits those bytes only at slots 12 or 13 at
`crates/rdocx-oxml/src/table.rs:1157` and
`crates/rdocx-oxml/src/table.rs:1166`. For example, an
`<ext:ins/>` between `w:tblHeader` and `w:jc` moves after `w:jc` on save. The
design requires unknown row properties to retain their observed `CT_TrPr`
boundary. Only malformed WordprocessingML revision markers belong in the fixed
revision slots.

### D5, a bitmap-filled shape is classified as a DrawingML picture
`crates/rdocx-oxml/src/drawing.rs:1110`
`crates/rdocx/src/run.rs:62`

`image_relationship_ids` accepts an Office relationship attribute on any
DrawingML `a:blip` anywhere in the inline or anchor subtree. It does not require
the schema-owned picture path under `pic:pic` and `pic:blipFill`.
`DrawingRef::kind` then checks that relationship before checking whether the
anchor parsed as a shape. A valid anchored `wps:wsp` whose `wps:spPr` contains
an `a:blipFill/a:blip` bitmap fill is consequently reported as
`DrawingKind::Image`, despite the public enum defining that variant as a
DrawingML picture. The tests at `crates/rdocx-oxml/src/drawing.rs:1432` use a
schema-invalid direct `a:blip` child and do not distinguish a picture payload
from another construct that happens to contain a blip.

## Smells

None.

## Nitpicks

None.

## Not found

- **Correctness and contract**: No additional issue was found in default-style
  numbering association, direct numbering overrides, `numId=0`, or the narrowed
  `has_unmodeled_properties` result.
- **Panics and bounds**: Apart from D3, malformed revision XML rejects without a
  panic, and the nested revision projection enforces its declared depth bound.
- **OOXML and preservation**: Apart from D1, D2, D4, and D5, table owner
  namespace propagation, inherited aliases, fixed-prefix shadows, retained raw
  subtrees, repeated serialization, and schema child order showed no issue.
- **PR 64 interaction**: Revision classification, private revision fields,
  source-ordered nested projection, and complex-field display ordering showed no
  issue in the combined tree.
- **Tests**: `cargo test -p rdocx-oxml` passed 321 unit tests and one doctest.
  `cargo test -p rdocx --lib` passed 325 tests with three ignored.
  `cargo test -p rdocx-layout revision --lib` passed 11 focused tests.
  `git diff --check` passed. No separate test smell was found beyond the missing
  sensitivity described in D1 and D5.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, or dynamic dispatch violation was found.
