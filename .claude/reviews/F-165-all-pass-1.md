# F-165, all, pass 1

**Reviewed**: uncommitted worker diff, 8 files, 531 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, repeated row content controls lack direct preservation coverage

`crates/rdocx/src/document.rs:6054`

The round-trip fixture proves table, row, cell, merge, banding, numbering, and
raw XML preservation, but none of its repeated rows or cells contains a typed
content control. The implementation checklist explicitly claims
content-control preservation through repetition. A regression in the
source-index remapping for table-level row controls could pass every new F-165
test. Add a content-control-wrapped row or cell to the multi-row fixture and
assert its position and value after repetition and reopen.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: multi-row ordering, lexical evaluation, continuous numbering,
  and invalid-reference rejection match the approved contract.
- Contract: no public method, type, trait, file, dependency, or feature flag
  was added.
- Panics: production changes add no indexing, slicing, arithmetic, unwrap, or
  expect on caller input.
- OOXML: the implementation clones existing typed owners and does not rebuild
  table or numbering parts. Raw row and cell bytes have round-trip assertions.
- Structure: the numbering lookup has one concrete responsibility and the
  recursive validators follow the existing typed placement variants directly.
