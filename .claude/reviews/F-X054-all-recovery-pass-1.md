# F-X054, all, recovery pass 1

**Reviewed**: uncommitted working diff, 14 files, 2,054 changed lines with
1,969 additions and 85 deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, Typed child resets reclassify inherited raw descendants

`crates/rdocx-oxml/src/document.rs:862`

The recovery serializer adds canonical `w`, `r`, and `mc` declarations to the
root of every modeled direct body child. That protects modeled descendants,
but it also changes the inherited scope of preserved raw descendants inside
that paragraph, table, or content control. For example, a body with
`xmlns:w="urn:foreign"` can contain a modeled `q:p` whose direct raw child is
`<w:r><w:t>foreign</w:t></w:r>`. Before save the paragraph item is unsupported
foreign XML. Save adds the canonical `xmlns:w` to the paragraph without
changing the raw bytes, so reopen parses the same bytes as a modeled Word run.
The current collision regression keeps the shadowed raw node as a direct body
child and therefore misses the nested trigger. This violates namespace scope
preservation, raw subtree identity, and ordered save and reopen equality.

### D2, Hyperlink shadow handling changes exposed raw bytes

`crates/rdocx-oxml/src/text.rs:4173`

`RunItemRef::UnsupportedXml` now exposes exact retained bytes, but the existing
hyperlink shadow path rewrites a raw run child by adding an `xmlns:w`
declaration before serialization. An aliased Word hyperlink with a local
foreign `xmlns:w`, a modeled aliased run, and raw `<w:producer/>` therefore
exposes `<w:producer/>` before save and
`<w:producer xmlns:w="urn:foreign"/>` after reopen. Namespace semantics survive,
but the approved round-trip contract requires every exposed raw subtree to
remain byte equal. The round-trip fixture has no locally shadowed hyperlink or
run raw child, so it remains green.

### D3, A shadowed `wp` prefix corrupts modeled drawings on save

`crates/rdocx-oxml/src/document.rs:1050`

The canonical reset list covers only `w`, `r`, and `mc`. The drawing serializer
emits `wp:inline` and `wp:anchor` without a local `xmlns:wp`, while the root
scope deliberately retains an existing `xmlns:wp` even when it is foreign. A
document or body that binds `wp` to `urn:foreign` and contains a modeled drawing
therefore saves the modeled drawing under the foreign namespace. The recovery
test iterates only `w`, `r`, and `mc`, so it does not prove canonical modeled
drawing prefixes or schema-valid output under a `wp` collision.

### D4, Qualified names are still derived by an ad hoc byte scanner

`crates/rdocx/src/document.rs:163`

`raw_element_name` asks quick XML only whether some start event exists, then
finds the qualified name by scanning from the first `<` byte. Both
`qualified_name()` and `local_name()` are derived from this result. The approved
contract explicitly requires unsupported XML classification to come from the
existing XML parser rather than a new byte scanner. Parser validation followed
by independent lexical classification does not satisfy that contract and can
diverge from the event that was actually accepted.

### D5, Empty modeled content controls report child content

`crates/rdocx/src/document.rs:147`

Every modeled compatibility fact returns `true` from `has_child_content()`.
An expanded empty content control such as `<q:sdt></q:sdt>` is parsed as a
modeled body fact even though it has no child element and no visible text. The
public fact therefore differs from an equivalent self-closing retained fact
and contradicts its documented meaning. The modeled-fact regression exercises
only a control with populated `sdtContent`.

### D6, Numeric whitespace references count as visible text

`crates/rdocx/src/document.rs:230`

The raw child-content detector treats every general reference as visible
content without resolving it. Valid XML such as
`<x:item>&#32;</x:item>` or `<x:item>&#xA;</x:item>` therefore reports child
content even though the resolved value is XML whitespace and the equivalent
literal text path reports false. The entity regression covers `&amp;`, which is
visible, but not a whitespace reference.

## Smells

None.

## Nitpicks

None.

## Not found

All pass 1 defects and the direct pass 2 defects remain remediated. The pass 3
root and direct-body collision cases also remain fixed for the covered `w`,
`r`, `mc`, and ordinary-prefix inputs. Focused ordered-reader and collision
regressions passed.

No additional findings were found in producer-defined numbering preservation,
layout marker suppression, HTML, Markdown, or RTF handling, fail-closed
ordinary and deleted text decoding, Python error classification, cell,
paragraph, hyperlink, or run item ordering, field projection, legacy flattened
accessors, public enum exhaustiveness, panic safety, dependency structure, or
the structural rules.
