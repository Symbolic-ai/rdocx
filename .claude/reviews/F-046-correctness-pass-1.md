# F-046, correctness, pass 1

**Reviewed**: working tree diff, 51 files, 321 insertions, 4601 deletions, and 20 binary asset deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an empty anchored drawing now emits an empty image
`crates/rdocx-layout/src/paginator.rs:310`

When an anchor has neither an image relationship ID nor a recognised shape,
the converter assigns the empty-byte media ID and this arm emits a positioned
image with empty bytes and MIME type. The pre-cutover paginator skipped an
anchored image whose relationship ID was empty. This changes the neutral page
element stream for malformed or unsupported anchors despite the contract to
preserve every existing flow-model decision.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness issue was found. Contract scope, panic paths, OOXML
preservation, test coverage, and structural rules produced no findings.
