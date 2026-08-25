# F-X054, all, recovery pass 2

**Reviewed**: uncommitted working diff, 14 files, 2,132 changed lines with
2,066 additions and 66 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, Nested non-Word namespace declarations bypass the preservation guard

`crates/rdocx/src/document.rs:391`

The document-wide fallback recognizes only a noncanonical `xmlns:w`
declaration below the body. The root and body scope check cannot see a
declaration local to a modeled paragraph, table, cell, content control,
hyperlink, or run. For example, a paragraph can declare
`xmlns:wp="urn:foreign"` and retain `<wp:producer/>` as a raw child. Even an
unchanged save is classified as safe, so the typed serializer drops the local
declaration, emits its canonical `wp` binding on the document root, and writes
the raw child bytes unchanged. Reopen then reports the producer child in the
Wordprocessing Drawing namespace instead of `urn:foreign`. An arbitrary local
prefix becomes unbound instead. A modification takes the same serialization
path rather than failing closed. This leaves recovery D1 and D3 incomplete for
nested prefixes other than `w`.

### D2, Escaped inherited root namespace URIs change on save and reopen

`crates/rdocx/src/document.rs:371`

`crates/rdocx-oxml/src/document.rs:854`

The new safety check treats an ordinary root prefix as safe, while the existing
typed document parser stores that declaration's lexical attribute payload.
For a valid root declaration such as `xmlns:x="urn:a&amp;b"`, the new body
scope projection correctly reports `urn:a&b` before save. Canonical
serialization passes the retained lexical `urn:a&amp;b` through an escaping
attribute writer, producing `urn:a&amp;amp;b`. After reopen the same raw body
fact reports `urn:a&amp;b`. This violates inherited namespace identity and the
required ordered save and reopen equality even on an unchanged document.

## Smells

None.

## Nitpicks

None.

## Not found

The direct recovery D1 through D6 cases were rechecked. Complete document-part
preservation works for the detected root, body, and nested `w` collisions.
Paragraph, table, content-control, hyperlink, and run raw bytes remain equal in
those cases. Root and body collisions for `wp`, `a`, `pic`, and `c` preserve the
producer part. Qualified and local names come from accepted quick XML events.
Expanded empty controls and numeric whitespace references report the expected
child-content facts.

No additional findings were found in cell, paragraph, hyperlink, or run item
ordering, public enum exhaustiveness, drawing and field projections, legacy
flattened accessors, producer-defined numbering preservation, layout and
exporter marker suppression, fail-closed ordinary or deleted text decoding,
Python error classification, OOXML child order, panic safety, public API
documentation, dependency structure, or the repository structural rules. The
154-test `rdocx` regression binary and the focused numbering, layout, text, and
PyO3 gates passed during this review.
