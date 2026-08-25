# F-X054, integration, pass 3

**Reviewed**: staged squash integration against `dc9d53f`, 38 files and 4,917
changed lines with 4,808 additions and 109 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, ODT producer-list regression omits container and deduplication coverage
`crates/rdocx/src/odt.rs:5460`

The repaired fixture adds every level-local loss but creates only one body
paragraph and leaves the numbering root, instance, and abstract preservation
fields empty. It therefore cannot fail if the producer-format branch stops
calling `scan_numbering_container_losses`, nor can it fail if repeated uses in
the body and a table cell publish duplicate container or level diagnostics.
Those were part of the pass-2 hidden-loss defect. Add retained root attributes
and XML, instance attributes and XML, abstract type metadata, attributes and
XML, plus body and table-cell paragraphs sharing this numbering identity. Then
assert the complete exact diagnostic sequence and one occurrence of each
numbering loss.

## Smells

None.

## Nitpicks

None.

## Not found

The EPUB producer branch at `crates/rdocx/src/epub.rs:799` suppresses the false
unresolved diagnostic, emits no list marker, and reports format, non-default
start, marker text, suffix, alignment, paragraph properties, run properties,
and retained level XML as losses. Its producer-specific messages no longer say
that marker text was replaced or spacing was normalized. The focused producer
run passed 3 tests, and the complete EPUB filter passed 34 runnable tests with
the pinned EPUBCheck test ignored by its environment guard.

The ODT implementation at `crates/rdocx/src/odt.rs:471` flattens producer-list
paragraphs without adding them to an ODT list. The loss scanners at
`crates/rdocx/src/odt.rs:1185` and `crates/rdocx/src/odt.rs:1248` cover retained
root, instance, abstract, and level state. The shared diagnostic key set at
`crates/rdocx/src/odt.rs:1718` deduplicates repeated body and table-cell visits
by path and message. The complete ODT library filter passed all 41 tests. The
broader `odt` filter also passed 41 library tests and the public writer
round-trip, then failed only when the sandboxed pinned LibreOffice oracle
subprocess could not start.

Modeled numbering remains intact. Layout numbering passed 14 tests, low-level
OOXML numbering passed 51 tests, and the complete native regression binary
passed all 169 tests. Producer-defined numbering round-trip, HTML and Markdown
marker suppression, RTF loss diagnostics, visible-text rejection, namespace
owner replay, schema ordering, and public enum exhaustiveness produced no
additional finding.

The public merge at `crates/rdocx/src/lib.rs:46` retains the EPUB, ODT, and SVG
results together with every F-X054 ordered-reader export. The Python mapping at
`crates/rdocx-py/src/lib.rs:66` preserves `XmlError` for OOXML failures and the
generic `RdocxError` for HTML and ODT failures. The focused binding test passed
1 test, and `cargo check` passed for all `rdocx` and `rdocx-py` targets.

The staged HLD changes are limited to the four files listed by the design plan.
They accurately record malformed visible-text rejection, producer-numbering
preservation, marker suppression, direct ordered facade facts, namespace
replay, and the intentional pre-1.0 source break. Formatting, staged prose,
and `git diff --check` passed with zero violations.
