# F-272, correctness, pass 1

**Reviewed**: Uncommitted F-272 diff against `8dea107f`, 18 files, 627 added lines and 26 removed lines.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML, tests, and structure produced no findings.
The review checked staged publication in `crates/rdocx/src/document.rs`,
related note comment anchoring in `crates/rdocx/src/comments.rs`, stable ID
lookup and displayed labels in `crates/rdocx-layout/src/engine.rs` and
`crates/rdocx-layout/src/notes.rs`, and the focused tests in
`crates/rdocx/tests/integration_test.rs` and
`crates/rdocx/tests/regression_test.rs`.

The differential gate uses deterministic PDF output and a recorded Microsoft
Word for Mac 16.113.2 build 16.113.26092012 export of the same generated
DOCX. Word displayed body labels 1 and 2, placed the long note on three
pages, and placed the short note on page three. The local gate checks those
page, label, and continuation observations. The package assertions check
reorder, rich content, removal, and reopen separately.
