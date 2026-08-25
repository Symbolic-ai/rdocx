# F-X054, all, pass 2

**Reviewed**: uncommitted working diff, 13 files, 1,523 changed lines with
1,472 additions and 51 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, Body-local namespace scope is lost on save and reopen

`crates/rdocx/src/document.rs:1156`

The new body namespace bindings are populated on open, but document flush still
serializes only `self.document`. That typed serializer does not receive or emit
the declarations retained in `body_namespace_bindings`. For example, open a
document whose body declares `xmlns:x="urn:body"` and contains
`<x:producer/>`. The first `namespace_uri()` call returns `urn:body`, but
`to_bytes()` writes the raw child under a canonical body without the body-local
declaration. After reopen the same fact returns `None`, and the serialized XML
contains an unbound prefix. This violates both inherited namespace preservation
and the required save-reopen equality.

### D2, Local namespace facts return lexical XML instead of the resolved URI

`crates/rdocx/src/document.rs:193`

Local namespace declarations are read from `attribute.value` and then found
again with a byte scan. That value is the lexical attribute payload, not the
parser-normalized namespace name. A valid raw child such as
`<x:producer xmlns:x="urn:a&amp;b"/>` therefore reports `urn:a&amp;b` from
`namespace_uri()` instead of `urn:a&b`. The inherited path already uses
`decoded_and_normalized_value`, so the same namespace reports different facts
depending on where it is declared. This also contradicts the contract that
namespace classification use the XML parser rather than an ad hoc byte scan.

### D3, The exact-order gate leaves several public item variants untested

`crates/rdocx/tests/regression_test.rs:1655`

`crates/rdocx/tests/regression_test.rs:1742`

The fixtures route unexercised variants through wildcard arms. The paragraph
fixture has no comment end, bookmark end, or hyperlink revision. The run
fixture has no drawing and exercises only the page break kind. A regression
that omits a drawing, reverses a hyperlink revision boundary, or maps a column
break as a line break still passes the named exact-order gates. The approved
test contract requires every supported typed variant and raw boundary.

### D4, The round-trip gate compares only two raw-byte projections

`crates/rdocx/tests/regression_test.rs:1959`

`crates/rdocx/tests/regression_test.rs:1975`

The save-reopen test filters both sides down to unsupported body XML and
unsupported paragraph XML. It never compares the ordered typed facts, namespace
facts, cell items, hyperlink items, or run items, and it does not cover every
raw subtree. This is why the body-local namespace loss above remains green.
The approved round-trip contract requires equality of ordered public facts and
every raw subtree after reopen.

## Smells

None.

## Nitpicks

None.

## Not found

All five pass-1 triggers were rechecked and their direct defects are fixed. Raw
run children before properties are emitted, conventional-looking prefixes use
their actual binding, body-local declarations resolve before save, empty
default undeclarations do not panic, and CDATA and entity references count as
child content.

No additional defects were found in producer-defined numbering preservation,
fail-closed ordinary or deleted text decoding, Python error classification,
cell ordering, paragraph and hyperlink sidecar implementation, field
projection, legacy flattened accessors, public item enum exhaustiveness,
dependency structure, OOXML child order, or the focused pass-1 regression
commands. No smells or nitpicks were found.
