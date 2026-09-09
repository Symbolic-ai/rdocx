# F-245, all, pass 2

**Reviewed**: working-tree implementation diff excluding review records, 17 files, 1,926 additions and 40 deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, aliased namespace declarations are removed from retained XML

`crates/rdocx-oxml/src/font_table.rs:589`

`raw_attributes` drops every namespace declaration whose value is one of the
three canonical namespaces, regardless of its prefix. An input rooted at
`q:fonts` therefore loses `xmlns:q` when changed output adds only `xmlns:w`.
Any retained `q:panose1`, `q:charset`, `q:sig`, or other unmodeled child is
then emitted with an unbound prefix. The same failure applies to locally
declared aliases used by retained producer attributes. This violates the
prefix-tolerant read and namespace-safe raw preservation contract.

### D2, valid explicit elements with ignorable trivia are not modeled

`crates/rdocx-oxml/src/font_table.rs:393`

`explicit_element_is_empty` accepts only an immediate end event. A valid
explicit form such as `w:altName` containing a producer comment, processing
instruction, or ignorable whitespace remains raw and never reaches the typed
field. A later `set_font` adds a second singleton property while retaining the
first one. The same path affects family, pitch, and embedded-face elements, so
the pass 1 explicit-element defect remains for common non-content XML trivia.

### D3, removal can delete a relationship the font table does not own

`crates/rdocx/src/document.rs:8880`

The target lookup correctly requires an internal relationship of type `font`,
but the following retain at line 8888 deletes the relationship id even when
that lookup returned `None`. A producer font-table element whose `r:id` points
to an external or wrong-type relationship therefore causes
`remove_embedded_font` or `remove_font` to delete an unrelated package edge.
The XML reference may be removed, but a relationship outside the exact owned
font contract must remain untouched.

### D4, part deletion checks only one relationship owner

`crates/rdocx/src/document.rs:8896`

The last-reference check searches only the active font table's relationship
set. If another package relationship owned by a different part resolves to the
same font part, removal deletes the part and leaves that other relationship
dangling. The package graph must be checked across all owners before an
existing producer part is treated as exclusively owned and removed.

### D5, the Word half of the differential gate is still self-derived

`crates/rdocx/tests/integration_test.rs:441`

The new LibreOffice conversion is genuine external evidence. The Word
projection is not. Its four expected values are the same literals assigned by
`authored_document` and `corpus_font` at lines 263 through 289, so the Word
assertion cannot detect disagreement with Word. The test still needs a
source-encoded observation from the pinned Word build, or an explicit manual
oracle record, to satisfy the plan's Word and LibreOffice comparison contract.

### D6, the HLD overstates relationship reuse after the safety fix

`docs/hld/04-opc-and-packaging.md:76`

The HLD says replacement reuses the existing relationship and part without a
qualification. The corrected implementation deliberately allocates a new
relationship and part when either resource is shared. Since HLD describes
current intent, it must state that reuse occurs only for an exclusively
referenced relationship and target.

## Smells

None.

## Nitpicks

None.

## Not found

No additional arithmetic, bounds, or panic-safety defect was found in the
public byte and GUID paths. The pass 1 root-boundary and same-owner shared-font
defects are fixed and covered. Staging remains atomic, all four face-kind
mappings preserve schema order, the legacy tint and shade helper is unchanged,
and no unapproved trait, generic, feature flag, crate, wrapper, or second module
was introduced.
