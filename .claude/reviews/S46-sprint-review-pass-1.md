# S46 sprint review, pass 1

**Reviewed**: `sprint/s46` against `7f081ad`, 45 files, 8,350 changed lines,
crates: `oxml-layout`, `oxml-opc`, `rdocx-html`, `rdocx-layout`, `rdocx-oxml`,
and `rdocx`
**Verdict**: 3 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, comment mutations leave bookmark and run-control indexes stale
`crates/rdocx/src/comments.rs:806`
`crates/rdocx/src/comments.rs:970`

Adding a comment reference shifts comment markers, hyperlinks, and raw XML
before inserting a run, but it does not shift `bookmark_markers` or
`content_controls`. Removing a reference-only run has the same omission in its
reverse remap. A paragraph containing an F-154 bookmark or an F-152 run-level
content control can therefore retain an old typed run boundary after an F-148
mutation. `Document::bookmarks`, `REF`, and `PAGEREF` then read the wrong range
in memory, while a typed control can serialize at a different boundary from
the one it occupied. The fix must remap every insertion-aware paragraph
inventory in both directions and add interaction regressions for comment
insertion and removal around bookmarks and run controls.

**Run-sprint class**: `fix-now`.

### B2, removing a comment skips anchors inside content controls
`crates/rdocx/src/comments.rs:945`

`remove_comment` deletes the comment and descendant reply entries, then walks
the document to remove their anchors. That walk explicitly ignores block and
cell content controls even though F-152 now parses their paragraphs, nested
controls, and runs into typed content. A producer comment anchored inside a
content control is left with dangling range or reference markup after its
comment entry has been deleted. The fix must recurse through every
`SdtContent` placement and prove that removal clears anchors inside block,
table, cell, paragraph, run, and nested controls without disturbing unrelated
typed or raw content.

**Run-sprint class**: `fix-now`.

### B3, immutable and mutable facade indexes disagree for wrapped content
`crates/rdocx/src/document.rs:553`
`crates/rdocx/src/document.rs:656`
`crates/rdocx/src/document.rs:677`
`crates/rdocx/src/document.rs:686`

`Document::paragraph` and `Document::table` enumerate through the recursive
F-152 accessors, so they include ordinary content wrapped by a body content
control. `paragraph_mut` and `table_mut` still enumerate only direct body
items. With a leading wrapped paragraph or table, immutable index zero and
mutable index zero address different content or the mutable lookup returns
`None`. This violates the approved recursive traversal contract and makes the
published facade unsafe to use by index. The fix must give immutable and
mutable accessors the same document order and add paired lookup tests for each
wrapped placement.

**Run-sprint class**: `fix-now`.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate requires one document carrying tracked changes, comments, content
controls, and bookmarks to preserve every unmodelled byte and expose all four
features for public reading and writing. It does not hold yet, and S46 is not
the milestone-closing sprint. Tracked-change stories remain for S47, while
`docs/sprints/SPRINT_PLAN.md:827` assigns the end-of-milestone gate to S48.

The narrower S46 definition also requires Word to open the authored comment
thread intact at `docs/sprints/CURRENT_SPRINT.md:50`. The automated regression
and package checks passed, and the candidate is bound to SHA-256
`a5ad0e8eb2d1a676daa07431deb2a0f11ee32e8bb92d099d14d5d16d43708adb`.
However, `docs/sprints/AS_BUILT.md:7079` records that Microsoft Word 16.104
build 16.104.25121423 was installed but no-repair opening, reply visibility,
and resolved-thread UI acceptance were not observed. The S46 Word-open clause
is therefore not evidenced and must not be claimed as met.

**Run-sprint class**: `human-action`. This is an evidence gap, not a code defect.

## Not found

Duplication, layering, harness, documentation, dependencies, and unrequested
public surface produced no additional findings. No manifest changed, the
`oxml-*` dependency direction remains intact, every S46 AS_BUILT entry records
the hash harness unchanged at 49 of 49, the integrated full verification passed
at `836c962`, and the ledger-only commit at `b2534a7` did not alter code.
