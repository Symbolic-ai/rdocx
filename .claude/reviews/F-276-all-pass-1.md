# F-276, all aspects, pass 1

**Reviewed**: Working diff against `8f4631d264333ba31bddec46f53ade1d389b0c3a`, 13 files, 2715 insertions and 161 deletions. Approved design and its HLD contract were read before review.
**Verdict**: 4 defects, 0 smells, 0 nitpicks.

## Defects

### D1, sequential numbering replacement can cascade and change list ownership

`crates/rdocx/src/field.rs:2562`

The numbering map is applied through repeated replacements on already modified
XML. It is a `HashMap`, so replacement order is not deterministic.
`crates/rdocx/src/document.rs:24218` allocates new numbering IDs from destination
state through `reserve_numbering_instance_id`, without excluding source IDs.
A destination occupying IDs 1 and 2 and a fragment using IDs 2 and 3 can
therefore produce the map 2 to 3 and 3 to 4. Processing 2 first changes its
references to 3, then the next replacement changes those same references to
4. The imported paragraphs now use the wrong list. Reversing map iteration
produces different output for the same inputs. Comment and note loops at
`crates/rdocx/src/field.rs:2578` and
`crates/rdocx/src/field.rs:2589` have the same problem. Apply the complete map
against original attribute values in one namespace-aware pass. Add overlapping
source and destination numbering IDs to the regression gate.

### D2, leaf reuse accepts a destination with a different outgoing graph

`crates/rdocx/src/field.rs:2618`

Equivalent related-part reuse checks that the source has no relationships,
but the destination comparison checks only payload bytes and content type.
For a source leaf image and a destination image with the same bytes and type
plus an opaque outgoing relationship, the importer reuses the destination
owner. `copy_fragment_part` writes the same payload and leaves that existing
relationship set in place because the source has no relationships to copy.
The imported content acquires a graph absent from the source. This violates
the declared relationship-free leaf reuse boundary. Require the destination
candidate to have no outgoing relationships too, and test equal payloads
with different leaf status.

### D3, custom XML collision remapping disagrees with store lookup identity

`crates/rdocx/src/field.rs:3375`

Custom XML lookup normalizes braces, whitespace and case in
`crates/rdocx/src/content_control.rs:650`, but replacement compares only case.
A binding with an unbraced store ID can resolve a braced item-property ID,
then fail to rewrite that property when the destination has the same store.
The valid import is rejected with the unsafe-rewrite error. There is another
failure with two selected bindings that use upper and lower case spellings
of the same ID. `fragment_store_item_ids` retains both lexical spellings and
`crates/rdocx/src/field.rs:2217` reserves a different new ID for each. The first
replacement changes the one copied property, so the second cannot find the
old ID and fails. Group stores by normalized identity or resolved source
part, allocate once, and rewrite all equivalent binding spellings and the
item property together. Test aliases in selected content and companions.

### D4, numbering style links lose their inherited namespace during remapping

`crates/rdocx/src/field.rs:2842`

The new style collision pass sends raw `styleLink` and `numStyleLink` subtrees
directly to `patch_word_value`. Those raw children normally inherit their
Word namespace from the numbering root. The numbering parser captures the
child without copying ancestor namespace declarations in
`crates/rdocx-oxml/src/numbering.rs:3348`. For example,
`<w:styleLink w:val="FragmentStyle"/>` has no declaration in the isolated
buffer. `patch_word_value` starts a fresh namespace reader, does not recognize
the unresolved `w` prefix as Word, and silently leaves the link unchanged.
When the style is renamed to `FragmentStyleMerge1`, its copied numbering
definition still points at `FragmentStyle`, attaching to a conflicting
destination definition. Remap these children with their actual inherited
scope or patch the complete numbering XML. Test both link forms with inherited
canonical and producer prefixes and assert the resulting style association.

## Smells

Zero smells found.

## Nitpicks

Zero nitpicks found.

## Not found

No additional correctness or contract findings beyond D1 through D4.
No additional OOXML findings beyond D4. No introduced panic finding was
established. Structure checks found no new traits, generic parameters,
forwarding wrappers, crates or modules requiring a second implementation.

The tests exercise the public API through supported owner pairs, nested
controls, note and comment companions, opaque cycles, external edges,
signature invalidation and atomic failures. These additions depend on the
new implementation and would fail against the previous owner restrictions.
The tests do not cover the specific collision and namespace cases above.
No separate test-only defect is recorded.

This pass used independent code and contract inspection. Worker test results
are supporting evidence, not an independently repeated gate. Publication
archive measurements remain a worker completion obligation and are not
claimed passed by this review. No source or test files were changed.
