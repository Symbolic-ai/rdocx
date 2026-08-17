# S46 sprint review, pass 2

**Reviewed**: `sprint/s46` against `7f081ad`, 46 files, 9,021 changed lines,
crates: `oxml-layout`, `oxml-opc`, `rdocx-html`, `rdocx-layout`, `rdocx-oxml`,
and `rdocx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Closed from pass 1

### B1, comment boundary indexes remain aligned
`crates/rdocx-oxml/src/text.rs:479`
`crates/rdocx/tests/regression_test.rs:413`

`insert_unwrapped_run` now shifts comment markers, bookmark projections, run
controls, hyperlinks, and raw XML through one paragraph-owned operation.
Comment removal collapses the same boundary inventories together. The focused
regression proves bookmark ranges remain correct in memory and after reload,
and proves the run control stays between the comment reference and following
run through both insertion and removal. B1 is closed.

### B2, comment cleanup reaches every typed control placement
`crates/rdocx/src/comments.rs:916`
`crates/rdocx/tests/regression_test.rs:474`

The cleanup walk now descends through body controls, table and row controls,
cell controls, paragraph controls, run controls, and nested controls. The
focused regression nests the comment paragraph and reference across those
placements, removes the comment, then proves both range and reference markup
are absent while the wrapped paragraph remains. B2 is closed.

### B3, mutable facade indexes match recursive immutable indexes
`crates/rdocx/src/document.rs:901`
`crates/rdocx/tests/regression_test.rs:535`

`paragraph_mut` and `table_mut` now resolve through the same recursive content
order as their immutable counterparts. The regression mutates direct, wrapped,
and deeply wrapped paragraphs plus a wrapped table by immutable index, then
reads the same objects back through the immutable API. B3 is closed.

The three focused regression tests also passed independently during this
review.

## Milestone gate

The M14 gate requires one document carrying tracked changes, comments, content
controls, and bookmarks to preserve every unmodelled byte and expose all four
features for public reading and writing. It does not hold yet, and S46 is not
the milestone-closing sprint. Tracked-change stories remain for S47, while
`docs/sprints/SPRINT_PLAN.md:827` assigns the end-of-milestone gate to S48.

The narrower S46 definition requires Word to open the authored comment thread
intact at `docs/sprints/CURRENT_SPRINT.md:50`. Automated regression and package
checks pass, and the candidate remains bound to SHA-256
`a5ad0e8eb2d1a676daa07431deb2a0f11ee32e8bb92d099d14d5d16d43708adb`.
However, `docs/sprints/AS_BUILT.md:7079` records that Microsoft Word 16.104
build 16.104.25121423 was installed but no-repair opening, reply visibility,
and resolved-thread UI acceptance were not observed. The S46 Word-open clause
is therefore not evidenced and must not be claimed as met.

**Run-sprint class**: `human-action`. This is an evidence gap, not a defect.

## Not found

The full integrated delta was rechecked for interaction, duplication,
layering, harness, gate, documentation, dependency, and public-surface issues.
No additional findings were found. No manifest changed, the `oxml-*`
dependency direction remains intact, every S46 AS_BUILT entry records the hash
harness unchanged at 49 of 49, and the remediation commit also records 49 of
49 unchanged. The initial integrated full verification passed at `836c962`.
