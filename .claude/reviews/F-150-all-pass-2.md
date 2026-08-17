# F-150, all, pass 2

**Reviewed**: full working diff against `e25ef35`, 2 files, 1,274 additions and 2 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, rejecting a property change can drop an inherited namespace binding
`crates/rdocx/src/revision.rs:287`

When rejection replaces the current property owner with the prior property
child, the namespace promotion includes declarations from the change element
but not declarations from the property owner that is being removed. A prior
`w:rPr`, `w:pPr`, `w:tblPr`, or `w:sectPr` that uses a prefix declared only on
the current outer property element is therefore emitted with an unbound
prefix. The wrapper namespace remediation covers content revisions, but this
property replacement path still produces namespace-invalid XML.

### D2, paragraph-mark resolution retains the wrong paragraph formatting
`crates/rdocx/src/revision.rs:604`

`render_merged_paragraph` keeps the first paragraph's opening element and full
inner content, including its `w:pPr`, then explicitly drops the following
paragraph's `w:pPr` at line 611. Deleting the paragraph mark between two
paragraphs leaves the following paragraph mark, so the merged paragraph must
carry the following paragraph's paragraph properties. Accepting a deletion or
rejecting an insertion between differently formatted paragraphs therefore
produces the wrong formatting. The regression input at
`crates/rdocx/tests/regression_test.rs:252` includes a right-aligned following
paragraph but asserts only count and text, so it does not catch this result.

### D3, leap-second timestamps are normalized to the wrong instant
`crates/rdocx/src/revision.rs:987`

The parser admits second 60 for every date and clock time, then converts it by
ordinary 86,400-second-day arithmetic at lines 990 to 996. This both accepts
values such as `2026-08-17T12:00:60Z`, which are not valid RFC 3339 leap-second
timestamps, and maps a real `23:59:60Z` leap second onto the following day's
`00:00:00Z`. Date-range membership and ordering are consequently wrong for
accepted inputs. The implementation must either represent RFC 3339 leap
seconds correctly or reject second 60 so the selector remains an instant
comparison.

## Smells

No smells found.

## Nitpicks

No nitpicks found.

## Not found

Pass-1 D1 is fixed by placement-aware modeling and the opaque-lookalike
regression. Pass-1 D2 is fixed for selected content wrappers by namespace
promotion. Pass-1 D3 is fixed by considering every selected contextual marker.
Pass-1 D4 is fixed for field widths, calendar bounds, and checked arithmetic.
Pass-1 D5 is fixed by the nested-revision and populated-cache atomicity tests.

No additional findings were found in public API shape, author or id selection,
ordinary offset normalization, nested selected revision counting, deleted-text
conversion, row-marker ownership, mutation commit ordering, package ownership,
schema child order, panic safety, or structural-rule compliance.
