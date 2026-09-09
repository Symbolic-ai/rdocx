# F-246, all, pass 2

**Reviewed**: remediated working-tree diff excluding review records, 20 files, 2,210 additions and 220 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, empty scalar elements still lose producer metadata

`crates/rdocx-oxml/src/styles.rs:141`

Pass 1 preservation applies only to explicit start and end forms. The common
self-closing form is projected without retaining its raw element. An aliased
`q:link`, or a `w:hidden` carrying a producer namespace declaration and custom
attribute, is rewritten canonically and loses those unmodeled attributes on
save. Empty and explicit forms need the same snapshot-based replay behavior.

### D2, style root attributes are discarded during every save

`crates/rdocx-oxml/src/styles.rs:105`

The style reader consumes only `styleId`, `type`, and `default`. It retains no
other root attribute or local namespace declaration, while `to_xml` constructs
a fresh `w:style` start at line 284. A modeled update therefore drops producer
attributes even when they are unrelated to the edit. Conflicting local `w` or
`r` bindings can also make fixed-prefix replay unsafe unless they are rejected
or remapped before publication.

### D3, changed conditional regions lose their root attributes

`crates/rdocx-oxml/src/styles.rs:920`

When a conditional region changes, its serializer rebuilds `w:tblStylePr` with
only `w:type`. Preserved child XML is replayed, but producer root attributes and
namespace declarations from `raw_xml` disappear. This violates the update
preservation contract for a directly exposed and authored story type.

### D4, partial table-property updates replace whole nested groups

`crates/rdocx/src/document.rs:3657`

The style update merge replaces the complete border or cell-margin value when
the authored `CT_TblPr` supplies one. Changing one border edge drops the other
five edges and any retained producer edge. Changing one margin drops the other
three. This is inconsistent with the established non-None patch semantics for
paragraph and run properties and can destroy unrelated producer formatting.

### D5, raw non-body story references remain unchecked

`crates/rdocx/src/document.rs:8466`

The package scan covers headers, footers, and endnotes only. A retained raw
subtree inside footnotes, comments, or glossary content can carry a valid
`w:pStyle`, `w:rStyle`, or `w:tblStyle` that the typed paragraph walk does not
project. Removing that style succeeds and leaves the package reference
dangling. The same namespace-aware scan should cover every related owned Word
story part.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 1 explicit-form projection, clear operations, base table cascade, and
typed footnote rejection are otherwise correct. Existing direct table behavior
and all 49 hash entries remain unchanged. Staging, graph validation, reciprocal
link maintenance, resolver cycle bounds, external oracle execution, panic
safety, and repository structure produced no additional findings.
