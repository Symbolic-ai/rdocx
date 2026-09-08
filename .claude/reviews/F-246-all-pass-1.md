# F-246, all, pass 1

**Reviewed**: complete working-tree diff, 20 files, 1,472 additions and 112 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, valid explicit style metadata is not projected

`crates/rdocx-oxml/src/styles.rs:170`

The new linked-style and UI fields are modeled only for `Event::Empty`.
Schema-valid explicit forms such as `<w:link w:val="BodyChar"></w:link>` and
`<w:hidden></w:hidden>` enter `extra_xml` instead. Graph validation then misses
the link, the public facade reports no value, and a later authored update can
emit a second singleton child. Explicit elements with ignorable whitespace,
comments, or processing instructions need the same typed projection while
their producer representation remains safe.

### D2, a table-style update can duplicate retained children and break order

`crates/rdocx-oxml/src/styles.rs:771`

`CT_TblPr::to_xml` already emits its `extra_xml` at schema boundaries. The new
serializer then extracts the same unmodeled children from the preserved raw
table properties and appends them immediately before `</w:tblPr>`. A caller
that obtains the typed table properties, changes one field, and passes them to
`set_style` therefore emits the retained child twice. Even a fresh authored
replacement appends retained children at the end instead of their schema
slots. This violates both exact preservation and `xsd:sequence`.

### D3, set-style cannot remove graph edges or optional properties

`crates/rdocx/src/document.rs:3570`

Every omitted builder field is restored from the existing style. A caller
cannot clear `basedOn`, `next`, `link`, any UI flag, table properties, or the
conditional-region list. In particular, reciprocal linked styles can never be
detached, so each permanently blocks removal of the other. The public method
is documented as replacement, but implements a patch with no clearing
operation. The facade needs a deterministic way to publish every valid update
promised by the create, update, and remove contract.

### D4, default table styles do not supply all modeled base properties

`crates/rdocx-layout/src/table.rs:286`

The default-style fallback was added only to cell shading, borders, and
paragraph-property resolution at line 825. Table width, alignment, indent,
layout, and default cell margins still read only direct `w:tblPr` values here.
An authored default table style carrying any of those modeled base properties
therefore does not affect layout, contrary to the plan's table-style resolution
and the HLD rule that direct table properties are the final overlay.

### D5, removal ignores references in non-body document stories

`crates/rdocx/src/document.rs:8327`

The live-reference check visits only the main document body. Footnote
paragraphs are held in the typed document state, while headers and footers are
owned package stories, and each can contain paragraph, run, or table style
references. Removing a style used by one of those stories succeeds and leaves
a dangling style id. The rejection check must cover all owned document
paragraphs and tables, not only body content.

## Smells

None.

## Nitpicks

None.

## Not found

No additional panic-safety or arithmetic issue was found in the new public
input paths. Staged mutations keep failures atomic and invalidate layout only
after validation. The source-built Word record and pinned LibreOffice
save-and-reopen rider exercise independent oracle evidence. No new trait,
generic parameter, feature flag, crate, forwarding wrapper, or module was
introduced.
