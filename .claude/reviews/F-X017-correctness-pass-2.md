# F-X017, correctness, pass 2

**Reviewed**: the remediated F-X017 working diff on `work/f-x017-claude`, 6
files, 427 insertions and 62 deletions. Pass 1 raised 1 defect, 2 smells and 2
nitpicks. This pass re-reads the whole diff, not only the repairs.
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Defects

None.

D1 is fixed. `crates/rdocx-layout/src/engine.rs:2758-2772` no longer matches on
the text of a run. It counts every text run on `pages[2..]`, which is the
endnote region for this two-section document, and it now asserts that region
exists rather than inferring it: `pages.len() > 2` and both documents producing
the same page count. The count no longer depends on where the shaper happened to
break a sentence.

## Smells

None.

S1 is fixed. `crates/rdocx-layout/src/engine.rs:170-179` collects the section
widths and stops. The redundant push is gone, and the comment now states the
fact that made it redundant: `final_geometry` belongs to the section pushed at
`engine.rs:163`, so the endnote width is already in the list.

S2 is fixed. `.claude/plans/F-X017-design.md:131-155` records the **Public API of
a published crate** row with the semver impact stated in full: `rdocx-layout` is
published at 0.7.0, `pub mod notes` makes both changed signatures public
surface, an external caller of either does not compile, and under 0.x that is a
minor bump for the next `/release` to state.

## Nitpicks

- `crates/rdocx-layout/src/notes.rs:130`, carried from pass 1 and deliberately
  not fixed. `counters_before` is cloned once per note even when a single width
  is registered. Branching to avoid one small `HashMap` clone per note would add
  a case to read for a cost that does not appear in any measurement, and the
  clone is what makes the numbering correctness argument a single sentence.

The pass 1 nitpick about `note_line_count` was fixed rather than carried. It is
now `note_baseline_count` at `engine.rs:2682`, and its doc comment states that
the number is the line count plus one per note drawn and that every use compares
two such counts.

## Not found

- **correctness**. Re-checked the width key, the ten lookup sites, the
  numbering-state restore and the endnote measure. `to_bits` equality holds
  because both sides are `PageGeometry::content_width()` over the same `sectPr`.
  The restore leaves the counters a single layout would have left, since
  numbering does not depend on width.
- **contract**, **panics**, **ooxml**, **structure**. Unchanged from pass 1 and
  re-checked against the remediation, which touched only a comment, a test body,
  a helper name and the plan.
- **tests**. The gate still fails against reverted code. Registering only the
  final width makes
  `a_note_is_broken_to_the_width_of_its_own_section` report equal counts for
  both sections and the assertion fires. 80 tests pass in `rdocx-layout`, 53
  binaries pass across the workspace, and the harness reports 28 of 28.

## Exit condition

Zero defects, zero smells. The remaining nitpick is taste and is recorded rather
than fixed, with the reason.
