# F-X007, all aspects, pass 9

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all eight earlier remediation
rounds. The reviewed state is 44 files and 5,001 changed or new line entries:
30 tracked files with 3,462 additions and 298 deletions, plus 1,241 lines in
fourteen untracked files.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, producer-only modeled property containers disappear on an unrelated typed edit

`crates/rdocx-oxml/src/numbering.rs:969`
`crates/rdocx-oxml/src/numbering.rs:980`
`crates/rdocx-oxml/src/numbering.rs:1156`
`crates/rdocx-oxml/src/numbering.rs:1160`
`crates/rdocx-oxml/src/numbering.rs:1187`
`crates/rdocx-oxml/src/numbering.rs:1213`
`crates/rdocx-oxml/src/numbering.rs:1239`
`crates/rdocx-oxml/src/numbering.rs:2919`
`crates/rdocx-oxml/src/numbering.rs:2924`
`crates/rdocx-oxml/src/borders.rs:261`
`crates/rdocx-oxml/src/borders.rs:262`
`.claude/plans/F-X007-design.md:48`
`.claude/plans/F-X007-design.md:50`

The recursive merger preserves nested producer XML only when the current typed
writer still emits the same modeled child. Both the top-level and recursive
overlays classify a recognized QName as modeled, remove it from `extras`, and
then iterate only the generated children. Neither path writes an unmatched
original modeled entry. For example, a valid level property can contain
`<w:tabs><ext:data/></w:tabs>` with the extension declared ignorable. Parsing
creates `CT_Tabs` with an empty typed vector, whose writer emits no `w:tabs`.
An unrelated typed indentation edit therefore generates no tabs child, and the
overlay silently drops the original producer container and subtree. The same
failure occurs when a modeled composite containing producer XML is explicitly
cleared. The new regression keeps a typed `numId` inside `numPr`, so the
generated composite remains present and does not exercise the missing-child
path. This still violates the plan's requirement that typed property changes
retain producer child subtrees.

### D2, repeated schema children are matched and positioned only by their shared slot

`crates/rdocx-oxml/src/numbering.rs:146`
`crates/rdocx-oxml/src/numbering.rs:969`
`crates/rdocx-oxml/src/numbering.rs:979`
`crates/rdocx-oxml/src/numbering.rs:1006`
`crates/rdocx-oxml/src/numbering.rs:1156`
`crates/rdocx-oxml/src/numbering.rs:1160`
`crates/rdocx-oxml/src/numbering.rs:1164`
`crates/rdocx-oxml/src/numbering.rs:1169`
`crates/rdocx-oxml/src/numbering.rs:1174`
`crates/rdocx-oxml/src/borders.rs:232`
`crates/rdocx-oxml/src/borders.rs:235`
`crates/rdocx-oxml/src/borders.rs:269`
`.claude/plans/F-X007-design.md:76`
`.claude/plans/F-X007-design.md:77`

`tabs/tab` is a repeated sequence, but every original and generated tab is
stored at schema position zero. The recursive merger pairs each generated tab
with the first unused original entry at that position. If two source tabs carry
distinct producer attributes and the first typed tab is removed, the remaining
generated tab is merged with the removed first tab. Its producer attribute is
transferred to the wrong tab, while the surviving tab's own producer attribute
is discarded as an unused original entry. Inserting a tab at the beginning has
the corresponding shift. Producer elements between repeated tabs are also
assigned the same slot and replayed before the first generated tab rather than
between their original neighbors. The public `CT_Tabs::tabs` vector supports
these ordinary ordered collection edits. Matching only by schema slot cannot
preserve occurrence identity or relative position.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 33 focused tests passed.
- `cargo test -p rdocx-oxml`: all 131 unit tests and the crate README doctest
  passed.
- `cargo clippy -p rdocx-oxml --all-targets --all-features -- -D warnings`:
  passed.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and the shell and dependency contracts passed.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: every inventory contains exactly one intended README.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- `gh pr view 25` confirms the PR remains merged into `sprint/s38` at
  `6aade64`, credits `@jonstokes`, and retains the public valuable-fix note.

## Not found

The pass-8 recursive merger retains attributes and nested producer subtrees
when a unique generated composite remains present. `rPrChange` now has the
schema-final `CT_RPr` slot. The main numbering parser consistently separates
element and attribute expanded-name checks, including default-foreign elements
and no-namespace attribute collisions. The published migration recipe now
lists the added fields per struct and correctly names `CT_Numbering`'s
`root_attributes`. The 64-element recursion bound still returns a normal
error. HLD14 and all sprint records consistently select `Tag v0.5.0`.
Manifest README wiring, archive inventories, documentation compilation, public
authoring bounds, table geometry, hash stability, and PR credit show no
additional defect. No smell or nitpick was found.
