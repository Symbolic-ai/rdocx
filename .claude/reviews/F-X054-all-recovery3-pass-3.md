# F-X054, all, third recovery pass 3

**Reviewed**: uncommitted working diff, 20 files, 3,543 changed lines with
3,467 additions and 76 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Namespace value decode failures still use the generic public error class

`crates/rdocx/src/document.rs:347`

The new namespace pre-scan converts event-reader failures through
`OxmlError`, but declaration attribute and value failures still become
`Error::Other`. A document whose root contains
`xmlns:x="urn:&bad;"` reaches `decoded_and_normalized_value`, fails on the
unknown entity, and the Python binding raises `RdocxError`. Malformed document
XML must retain the existing `XmlError` boundary. The new binding regression
only corrupts the part with an unmatched end tag, so it exercises the event
reader conversion but not the declaration conversion paths.

## Smells

None.

## Nitpicks

None.

## Not found

The two third-recovery namespace defects remain fixed. A local
WordprocessingML alias shared by retained raw XML and modeled siblings replays
on the correct owner after a modified save and reopen. An intermediate raw
declaration shadows the owner declaration by provenance, including when the
prefix and URI are identical. Direct retained use of an owner-local foreign
`wp` binding still fails closed.

The named package, malformed end-tag, and stale-handle Python cases passed
with `PackageError`, `XmlError`, and `StaleElementError` respectively. All 8
shared binding tests passed after the exact pinned Python 3.12.9 extension was
rebuilt. All 168 `rdocx` regression tests passed. Prose checking and
`git diff --check` passed.

No additional findings were found in body, cell, paragraph, hyperlink, or run
item ordering, complete typed variant projection, exact raw bytes, modeled
unsupported facts, namespace alias resolution and replay, producer-defined
numbering preservation, layout and exporter marker suppression, fail-closed
ordinary or deleted text decoding, named package and stale-handle error
classification, legacy flattened accessors, public enum exhaustiveness, OOXML
child order, panic safety, public documentation, dependency structure, test
naming, or the repository structural rules.
