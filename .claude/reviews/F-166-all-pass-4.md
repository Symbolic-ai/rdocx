# F-166, all, pass 4

**Reviewed**: Uncommitted working diff, 4 files and 1,728 changed lines, with
1,684 additions and 44 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Nested section properties can hide a record-varying header or footer

`crates/rdocx/src/field.rs:333`

`crates/rdocx/src/field.rs:1901`

`crates/rdocx/src/field.rs:1911`

The sectioned dependency scan obtains header and footer parts exclusively from
`referenced_header_footer_parts`, but that helper enumerates only direct body
paragraphs and the body-final section properties. It explicitly ignores block
content controls and preserved raw body wrappers. A schema-valid block-level
`w:sdt` or `w:customXml` can contain a paragraph whose `w:pPr/w:sectPr` owns the
only reference to a header or footer. If that part contains a record-varying
`MERGEFIELD`, `mail_merge_sections` never scans it and accepts the records
instead of returning the required rejection. The package story remains at its
stored value because the field traversal relies on the same incomplete part
discovery.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-3 D1 is resolved. Identity discovery and editing now scan the complete
serialized body. Raw simple and complex `REF` and `PAGEREF` instructions,
hyperlink anchors, and unresolved generated-name candidates are reserved and
remapped.

Pass-3 D2 is resolved. Attribute and instruction text entities are decoded for
comparison, and changed simple attributes and complex instruction text are
escaped again when written.

Pass-3 D3 is resolved. The new field and identity scanners resolve expanded
attribute names, distinguish WordprocessingML attributes from foreign
same-local-name attributes, and require unbound drawing identity attributes
where the drawing schemas do.

Correctness and contract: no additional finding beyond D1. Empty and single
record handling, record order, section property movement, and merge-local
missing-value behavior remain consistent with the approved design.

Panics: none found. Empty records are rejected before candidate indexing,
identity allocation uses checked arithmetic, and XML edit spans are bounded.

OOXML: no additional finding beyond D1. Namespace scope is retained by the
namespace resolver, final section properties remain schema-final, and changed
reference values are re-escaped without reserializing unrelated raw subtrees.

Footnote source and dirty handling: no finding. Clean relationship-resolved
parts are patched in place and retain unmodelled children. Complete typed
serialization remains confined to explicitly dirty footnotes.

Tests: no independent harness defect found. `cargo test -p rdocx --test
regression_test` passed all 92 tests. The uncovered nested section-reference
case is the trigger documented in D1.

Structure and scope: no finding. The additional scanner and edit helpers remain
concrete and local to the two merge operations. No new trait, generic, module,
forwarding wrapper, feature flag, or speculative abstraction was introduced.

Verification also passed `cargo check -p rdocx --all-targets`, `git diff
--check`, and `python3 scripts/prose_check.py`.
