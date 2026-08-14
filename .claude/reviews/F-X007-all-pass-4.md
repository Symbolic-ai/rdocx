# F-X007, all aspects, pass 4

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all three earlier remediation
rounds. The reviewed state is 36 files and 2,206 changed or new line entries:
27 tracked files with 1,521 additions and 203 deletions, plus 482 lines in nine
untracked files.
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, modelled attribute lookup still confuses foreign and aliased qualified names

`crates/rdocx-oxml/src/numbering.rs:59`
`crates/rdocx-oxml/src/numbering.rs:122`
`crates/rdocx-oxml/src/numbering.rs:377`
`crates/rdocx-oxml/src/numbering.rs:560`

The pass-3 remediation filters retained attributes by exact spelling, but the
value lookup still accepts the first attribute with the requested local name.
Consequently, placing `ext:ilvl`, `ext:numId`, or `ext:abstractNumId` before the
real Word attribute can make a valid producer extension control the modelled
identifier or make parsing fail when its value is not numeric. The regression
puts every `w:` attribute before its colliding `ext:` attribute, so it does not
exercise this ordering.

There is a second failure at the same identity boundary. Prefix-tolerant input
may use `q:ilvl` with `q` bound to the WordprocessingML namespace. The lookup
models that attribute, while the exact-spelling filter retains it as extra XML.
Serialization then adds canonical `w:ilvl` as well, yielding two attributes
with the same expanded name and therefore namespace-invalid XML. Attribute
lookup and exclusion must agree on expanded qualified identity while retaining
genuinely foreign collisions. The current behavior contradicts the namespace
and compatibility-attribute contract at
`docs/hld/04-opc-and-packaging.md:107`.

### D2, nested paragraph and run property extensions are discarded

`crates/rdocx-oxml/src/numbering.rs:219`
`crates/rdocx-oxml/src/numbering.rs:251`
`crates/rdocx-oxml/src/numbering.rs:316`
`crates/rdocx-oxml/src/properties.rs:123`
`crates/rdocx-oxml/src/properties.rs:135`

The self-closing remediation represents `w:pPr` and `w:rPr` only as empty
default property structs, then emits fresh attribute-free elements. Thus
`<w:pPr ext:marker="p"/>` and `<w:rPr ext:marker="r"/>` lose their attributes
on the next numbering serialization. Expanded property containers are also
sent to the general property parsers without their start tags. Those parsers
discard unmodelled nested subtrees, so an extension inside either container is
lost as well. The new test asserts only that each property tag occurs once at
`crates/rdocx-oxml/src/numbering.rs:1245`, not that its attributes or content
survive. This violates the modelled-container and producer-extension contract
at `docs/hld/04-opc-and-packaging.md:107` and the crate README's preservation
claim at `crates/rdocx-oxml/README.md:3`.

### D3, expanded forms of modelled empty-content elements are treated as raw XML

`crates/rdocx-oxml/src/numbering.rs:217`
`crates/rdocx-oxml/src/numbering.rs:231`
`crates/rdocx-oxml/src/numbering.rs:471`
`crates/rdocx-oxml/src/numbering.rs:481`
`crates/rdocx-oxml/src/numbering.rs:799`

The level parser models `start`, `numFmt`, `lvlText`, and `lvlJc` only when
quick-xml reports a self-closing event. XML permits the equivalent expanded
form, such as `<w:numFmt w:val="decimal"></w:numFmt>`, but the start-event
branch captures it as an unmodelled subtree. `set_list_level` then materializes
a second modelled value and the writer emits both, potentially putting the old
raw sequence before a new `start` and producing duplicate, schema-disordered
children. `CT_Num` has the same split: an expanded `w:abstractNumId` is captured
as raw XML and leaves the modelled reference at zero. The new setter can then
reject a list that exists, or resolve the wrong abstract definition when zero
also exists. All current numbering fixtures use self-closing scalar children,
so the green round trips do not cover these equivalent inputs. This contradicts
the known-list update contract at `docs/hld/04-opc-and-packaging.md:104`.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 18 focused tests passed.
- `cargo test -p rdocx-oxml`: 116 unit tests and one README doctest passed.
- `cargo test -p rdocx`: 69 unit tests, 81 integration tests, 17 regression
  tests, and two doctests passed.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled with warnings denied.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: each inventory contained exactly one intended README.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- GitHub PR 25 remains merged into `sprint/s38` at `6aade64`. Its three source
  commits retain Jon Stokes as author, and the public merge note credits
  `@jonstokes` and explains the value of the contribution.

## Not found

The pass-3 fixed-slot remediation keeps `w:nsid`, `w:tmpl`, and other abstract
schema predecessors before inserted levels, and it keeps raw children between
existing levels attached to the correct insertion boundary. The root boundary
remediation keeps `w:numPicBullet` before modelled records and schema-final
content after newly added abstract and instance records. No additional defect
was found in self-closing root, abstract, or level identity, ordinary foreign
attribute collisions in the tested attribute order, public list and paragraph
bounds, identifier allocation, hyperlink and hard-break behavior, staged table
geometry, README example compilation and inventories, package wiring, HLD file
scope, hash stability, or contributor credit. No structural-rule violation,
resource-bound smell, additional public-input panic, or documentation nitpick
was found.
