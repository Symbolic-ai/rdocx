# F-273, correctness, pass 1

**Reviewed**: working diff against `dcada9d6`, 13 files, 492 added lines and 50 deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, differential gate has no Word observation

`crates/rdocx/tests/integration_test.rs:22568`

The test named as the approved Word gate renders only with the local deterministic engine, then asserts its own PDF text and page count. The pinned Word version constant does not make those expectations an external comparison. Microsoft Word 16.113.2 is running without an accessible window, so this generated DOCX has not yet produced a Word export. LibreOffice 26.2.5.2 gives two pages and Roman endnote labels for the same file, while the local render gives three pages and Arabic labels. The format policy is assigned to F-274, but page placement must be compared with Word before this gate can pass.

## Smells

None found.

## Nitpicks

None found.

## Not found

No additional correctness, contract, panic, OOXML preservation, or structure findings were found in the staged note lifecycle and rich story APIs.
