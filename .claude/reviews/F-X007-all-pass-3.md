# F-X007, all aspects, pass 3

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and both earlier remediation rounds.
The reviewed state is 35 files and 1,958 changed or new line entries: 27
tracked files with 1,393 additions and 193 deletions, plus 372 lines in eight
untracked files.
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested compatibility attributes are filtered by local name instead of qualified name

`crates/rdocx-oxml/src/numbering.rs:67`
`crates/rdocx-oxml/src/numbering.rs:356`

`capture_extra_attributes` drops every attribute whose local name matches a
modelled attribute. That also drops a foreign producer attribute such as
`ext:ilvl` from `w:lvl`, `ext:numId` from `w:num`, or `ext:abstractNumId` from
`w:abstractNum`. It can also drop a declaration such as
`xmlns:ilvl="urn:producer"`, because its local name is `ilvl`. A raw
`<ilvl:extension/>` child then survives while its binding is removed, producing
an unbound prefix on save. The ordinary `w15` and `w16` cases in the regression
do not collide with modelled attribute names, so they pass. This contradicts
the modelled-container attribute contract at
`docs/hld/04-opc-and-packaging.md:107`.

### D2, self-closing modelled descendants are still captured as raw children

`crates/rdocx-oxml/src/numbering.rs:363`
`crates/rdocx-oxml/src/numbering.rs:218`

The self-closing remediation recognizes root `w:numbering` and root-level
`w:abstractNum`, but `CT_AbstractNum::from_xml` still treats a valid
self-closing `<w:lvl w:ilvl="0"/>` as raw XML. A later `set_list_level` cannot
find it and inserts a second level with the same `ilvl`. Inside an expanded
level, valid self-closing `w:pPr` and `w:rPr` elements also fall through the
generic raw-child branch rather than their fixed schema slots. If the setter
materializes earlier fields, those properties can be emitted before
`w:start`. The regression at `crates/rdocx-oxml/src/numbering.rs:1134` covers
only the root and an empty abstract definition, so neither nested form is
exercised.

### D3, inserting a missing level can move abstract-definition predecessors after the level

`crates/rdocx-oxml/src/numbering.rs:360`
`crates/rdocx-oxml/src/numbering.rs:760`

Abstract-definition raw children are positioned only by the count of modelled
`multiLevelType` and `lvl` children. For an abstract definition containing
`w:nsid` and `w:tmpl` but no level, both raw children can occupy boundary zero.
`set_list_level` inserts level zero at that boundary and shifts both raw
children to boundary one. Serialization then emits the new `w:lvl` before
`w:nsid` and `w:tmpl`, although both are schema predecessors of every level.
The same failure occurs after a modelled `w:multiLevelType` for `w:tmpl`. This
violates schema order and the insertion-aware abstract-definition claim at
`docs/hld/04-opc-and-packaging.md:109`.

### D4, adding the first list can move a schema-final root child before the list

`crates/rdocx-oxml/src/numbering.rs:604`
`crates/rdocx-oxml/src/numbering.rs:704`

At an empty numbering root, both a schema predecessor and a schema-final raw
child have boundary zero. `add_list` calls `shift_extras_after` at zero, so a
final child such as `w:numIdMac`, or the preserved root extension used by the
feature regression, stays at zero. The writer emits boundary-zero extras before
the new abstract definition. The final child has therefore crossed the newly
inserted abstract definition and numbering instance, violating root child
order. The existing preservation test starts with one abstract definition and
one instance, which gives its final extension a nonzero boundary and masks the
empty-root case.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml`: 114 unit tests and 1 doc test passed.
- `cargo test -p rdocx`: 69 unit tests, 81 integration tests, 17 regression
  tests, and 2 doc tests passed.
- `python3 scripts/readme_doctests.py`: all 12 Rust examples across the six
  stable libraries compiled, and all required shell and dependency snippets
  passed their contracts.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: each inventory contained exactly one intended README.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- GitHub PR 25 remains merged into `sprint/s38` at `6aade64`. Its three source
  commits retain Jon Stokes as author, and the public merge note credits
  `@jonstokes` and explains the value of the fix.

## Not found

No additional defect was found in ordinary root, abstract-definition,
instance, or level namespace inheritance, fixed `CT_Lvl` slots for
`w:lvlRestart` and the other raw schema children, the already-covered
self-closing root and abstract definition, public list and paragraph bounds,
ID allocation, hyperlink and hard-break behavior, staged table geometry,
stable README examples and snippets, package README wiring, HLD file scope,
hash stability, or contributor credit. No structural-rule violation,
resource-bound smell, additional public-input panic, or documentation nitpick
was found.
