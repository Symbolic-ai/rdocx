# F-150, all, pass 3

**Reviewed**: full working diff against `e25ef35`, 2 files, 1,340 additions and 2 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, consecutive deleted paragraph marks do not merge the complete sequence
`crates/rdocx/src/revision.rs:485`

The body renderer merges one marked paragraph with exactly its next paragraph
and then advances past both. `render_merged_paragraph` renders a selected mark
on that next paragraph as an ordinary contextual marker, so a sequence of three
paragraphs whose first two paragraph marks are deleted becomes two paragraphs
instead of one. Both revisions are nevertheless counted and removed. This
violates the placement-specific paragraph-mark resolution contract.

### D2, removing an outer revision skips validation of selected descendants
`crates/rdocx/src/revision.rs:399`

When the chosen action removes a content wrapper, the renderer calls
`mark_selected_descendants` and drops the complete subtree without resolving
those descendants. Rejecting a selected insertion that contains a separately
selected malformed property change therefore succeeds and counts both
revisions, although resolving that property change itself would fail. The
approved inside-out and atomic-failure contract requires every selected
descendant to resolve successfully before the staged document can commit.

### D3, property rejection accepts the wrong prior-property element type
`crates/rdocx/src/revision.rs:524`

`selected_property_change` treats the first child as the recorded prior value
without checking its Word namespace or local name against the owner. For
example, a selected `w:rPrChange` containing `w:pPr` replaces the current
`w:rPr` with `w:pPr`. The permissive document reparse can retain that child as
raw XML, so the operation commits malformed schema content instead of failing
atomically. The existing malformed-property regression covers only a missing
child.

### D4, valid lowercase RFC 3339 separators are rejected
`crates/rdocx/src/revision.rs:961`

RFC 3339 permits lowercase `t` and `z`, but the parser requires uppercase `T`
and strips only uppercase `Z`. A bound such as
`2026-08-17t09:00:00z` is therefore rejected even though the public methods
promise RFC 3339 instant ranges. The leap-second remediation correctly rejects
second 60 consistently, but the selector is still not total over the stated
input format.

## Smells

No smells found.

## Nitpicks

No nitpicks found.

## Not found

Pass-2 D1 is fixed by recovering declarations from the original property
owner before prior-property replacement. Pass-2 D2 is fixed for a single
paragraph-mark merge by retaining the following paragraph properties. Pass-2
D3 is fixed by rejecting all second-60 bounds without mutation.

No additional findings were found in the eight-method public API shape,
ordinary author and id selection, offset and fractional-second ordering,
content-wrapper namespace promotion, deleted-text conversion, row-marker
ownership, unmodelled lookalike preservation, package ownership, cache
invalidation after a successful commit, schema child order outside the cited
malformed path, panic safety, oracle pinning, or structural-rule compliance.
