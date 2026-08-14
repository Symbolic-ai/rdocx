# F-X007, all aspects, pass 10

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all nine earlier remediation
rounds. The reviewed state is 45 files and 5,455 changed or new line entries:
30 tracked files with 3,803 additions and 300 deletions, plus 1,352 lines in
fifteen untracked files.
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, explicit clear replays stale modelled XML or drops the producer container

`crates/rdocx-oxml/src/numbering.rs:1371`
`crates/rdocx-oxml/src/numbering.rs:1383`
`crates/rdocx-oxml/src/numbering.rs:1452`
`crates/rdocx-oxml/src/numbering.rs:1465`
`crates/rdocx-oxml/src/numbering.rs:1492`
`crates/rdocx-oxml/src/numbering.rs:1765`
`crates/rdocx-oxml/src/numbering.rs:1787`
`crates/rdocx-oxml/src/numbering.rs:3275`
`.claude/plans/F-X007-design.md:48`
`.claude/plans/F-X007-design.md:50`

The pass-9 retention path detects producer content in an unmatched modelled
child, but retains and writes the child's complete original raw bytes. It does
not remove the typed projection that the caller explicitly cleared. A source
`w:tabs` containing one typed `w:tab` plus `ext:data`, followed by
`ppr.tabs = None`, therefore writes both `ext:data` and the old typed tab. The
new regression uses a producer-only tabs container, so it cannot detect this
stale typed state. Nested composites have the same failure through the
recursive retained-raw path. At the level boundary, setting `ppr` or `rpr`
itself to `None` takes the opposite wrong branch and skips its raw preservation
state completely, dropping the producer subtree. An explicit clear must remove
the modelled values while retaining only producer attributes and subtrees.

### D2, editing a tab's typed value discards that occurrence's producer data

`crates/rdocx-oxml/src/numbering.rs:1224`
`crates/rdocx-oxml/src/numbering.rs:1270`
`crates/rdocx-oxml/src/numbering.rs:1278`
`crates/rdocx-oxml/src/numbering.rs:1302`
`crates/rdocx-oxml/src/numbering.rs:3305`
`.claude/plans/F-X007-design.md:50`
`.claude/plans/F-X007-design.md:77`

The occurrence matcher defines identity as the complete `val`, `pos`, and
`leader` tuple, then merges producer data only when that tuple is unchanged.
Changing an existing tab position from 720 to 1,440 makes the generated tab
unmatched. It is written directly, while the unused original modelled tab is
discarded. A direct public-API probe produced the new `w:pos="1440"` but lost
the source `ext:id="a"`. The regression covers removing one distinct identity
and inserting a new identity. It does not cover an ordinary typed mutation of
one retained occurrence. Typed tab edits remain subject to the plan's producer
attribute and subtree preservation contract.

### D3, an aliased tabs container is closed with a different QName

`crates/rdocx-oxml/src/numbering.rs:1183`
`crates/rdocx-oxml/src/numbering.rs:1261`
`crates/rdocx-oxml/src/numbering.rs:1277`
`crates/rdocx-oxml/src/numbering.rs:1311`
`.claude/plans/F-X007-design.md:46`
`.claude/plans/F-X007-design.md:76`

The repeated-tabs merger takes its start QName from the generated container but
its end QName from the original container. With a valid source
`<q:tabs xmlns:q="...wordprocessingml...">`, any tab edit opens `w:tabs` and
then attempts to close `q:tabs`. The public `CT_Numbering::to_xml` call returns
`MismatchedEndTag { expected: "w:tabs", found: "q:tabs" }` instead of XML.
The same failure applies when collision avoidance selects another generated
Word prefix. Prefix-tolerant input must not turn a typed edit into a
serialization error.

### D4, a tabs-local Word alias is not inherited by occurrence matching

`crates/rdocx-oxml/src/numbering.rs:1157`
`crates/rdocx-oxml/src/numbering.rs:1184`
`crates/rdocx-oxml/src/numbering.rs:1191`
`crates/rdocx-oxml/src/numbering.rs:1217`
`crates/rdocx-oxml/src/numbering.rs:1265`
`crates/rdocx-oxml/src/numbering.rs:1294`
`.claude/plans/F-X007-design.md:46`
`.claude/plans/F-X007-design.md:76`

`repeated_property_overlay` correctly resolves a namespace alias declared on
the `tabs` container while classifying its children, but the returned overlay
does not retain those in-scope prefixes. Identity extraction and child merging
receive only the caller's parent prefixes. For
`<w:tabs xmlns:q="...wordprocessingml..."><q:tab ... ext:id="a"/></w:tabs>`,
inserting a new tab leaves the original typed values unchanged, yet the
original occurrence is not recognized and `ext:id="a"` disappears. This is a
different failure from the start/end mismatch because the container QName can
remain canonical while only its children use the local alias.

### D5, occurrence matching is quadratic over an unbounded public collection

`crates/rdocx-oxml/src/numbering.rs:1274`
`crates/rdocx-oxml/src/numbering.rs:1278`
`crates/rdocx-oxml/src/numbering.rs:1279`
`crates/rdocx-oxml/src/borders.rs:232`
`crates/rdocx-oxml/src/borders.rs:239`

For every generated tab, the matcher scans the original identities from the
beginning and consults a separate used bitmap. Even an unchanged ordered list
takes one plus two through N comparisons. `CT_Tabs::tabs` is a public `Vec`,
and parsing accepts every occurrence without a bound, so an extended numbering
part with many tab stops makes the preservation edit path consume quadratic
CPU. The new recursion limit does not bound this sibling collection. The
occurrence index needs bounded or near-linear lookup while keeping deterministic
queues for repeated identities.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 35 focused tests passed.
- `cargo test -p rdocx-oxml`: all 133 unit tests and the crate README doctest
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

For canonical prefixes and distinct typed identities, removing the first tab
retains the second occurrence's producer attributes, and inserting a new first
tab retains the original between-occurrence node. No additional schema-slot or
property-writer ordering regression was found. Unsupported nested XML,
`rPrChange`, default-foreign elements, no-namespace attributes, generated
prefix collision avoidance outside the specialized repeated path, and the
64-element recursion error remain correct. The public 0.5.0 migration recipes,
HLD14 and sprint title/version records, README and manifest contracts, package
inventories, authoring bounds, table geometry, hash stability, and PR credit
remain consistent. No separate smell or nitpick was found.
