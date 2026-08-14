# F-X007, all aspects, pass 14

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all thirteen earlier review and
remediation rounds. The reviewed state is 51 files and 7,954 changed or new
line entries: 32 tracked files with 5,284 additions and 714 deletions, plus
1,956 lines in nineteen untracked files.
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, semantic equality bypasses provenance when the typed values do not change

`crates/rdocx-oxml/src/borders.rs:199`
`crates/rdocx-oxml/src/borders.rs:206`
`crates/rdocx-oxml/src/borders.rs:213`
`crates/rdocx-oxml/src/numbering.rs:891`
`crates/rdocx-oxml/src/numbering.rs:898`
`crates/rdocx-oxml/src/numbering.rs:2092`
`crates/rdocx-oxml/src/numbering.rs:2095`
`crates/rdocx-oxml/README.md:36`
`.claude/plans/F-X007-design.md:55`

The public contract deliberately excludes `source_occurrence` from semantic
equality, while `CT_TabStop::new` marks a tab as new with `None`. The level
writer uses that semantic equality as its complete raw-container fast-path
test. If a caller replaces a parsed producer-bearing tab with a newly
constructed tab that has the same alignment, position, and leader, the current
`pPr` still compares equal to the parsed snapshot. The writer emits the entire
old raw `pPr` and never consults the new tab's `None` provenance.

For example, replacing `<w:tab w:pos="720" ext:id="a"/>` with
`CT_TabStop::new(Left, Twips(720))` retains `ext:id="a"`, even though the new
occurrence explicitly claims no source. The same problem makes a
provenance-only reset ineffective. Equality can remain semantic, but the raw
fast path must separately establish that preservation provenance is unchanged.

### D2, an existing `mc:Ignorable` produces a duplicate expanded-name attribute

`crates/rdocx-oxml/src/numbering.rs:294`
`crates/rdocx-oxml/src/numbering.rs:300`
`crates/rdocx-oxml/src/numbering.rs:1441`
`crates/rdocx-oxml/src/numbering.rs:1455`
`crates/rdocx-oxml/src/numbering.rs:1474`
`crates/rdocx-oxml/src/numbering.rs:1475`
`.claude/plans/F-X007-design.md:66`
`.claude/plans/F-X007-design.md:68`

The producer-only tabs path copies every foreign attribute from the original
container. That includes an existing `mc:Ignorable`. When a carrier is needed,
`mark_ignorable` unconditionally declares a fresh markup-compatibility prefix
and pushes another `Ignorable` attribute. A source `w:tabs` carrying
`mc:Ignorable="w15"` therefore becomes a start tag with both
`mc:Ignorable="w15"` and `mc1:Ignorable="rdocxPreserve"`.

Those spellings have the same expanded attribute name in the markup
compatibility namespace. XML Namespaces forbids duplicate attributes by
expanded name, so an external namespace-aware consumer can reject the generated
part. The new test covers occupied `mc` prefix text, but not an existing
markup-compatibility `Ignorable` attribute that must be extended rather than
duplicated.

### D3, the tab parser loses inherited namespace shadows inside unknown containers

`crates/rdocx-oxml/src/borders.rs:255`
`crates/rdocx-oxml/src/borders.rs:275`
`crates/rdocx-oxml/src/borders.rs:320`
`crates/rdocx-oxml/src/borders.rs:331`
`.claude/plans/F-X007-design.md:63`
`.claude/plans/F-X007-design.md:65`

`CT_Tabs::from_xml_with_prefixes` recomputes prefixes only from each empty
element and the original inherited list. It does not maintain a namespace
scope stack or skip unknown start-element subtrees. Given an outer `q:tabs`
where `q` is WordprocessingML, an inner foreign container may rebind `q` and
contain `<q:tab/>`. The tab event does not repeat the ancestor declaration, so
the parser restores the outer Word binding and materializes the foreign tab.
The inner `</q:tabs>` can also satisfy the outer end test and terminate parsing
before later legitimate tabs.

This reproduces with a default namespace shadow as well as a prefixed alias.
The direct foreign sibling fixture passes because its foreign prefix was never
in the inherited Word list. It does not prove contextual parsing across nested
scope changes, which is required for the advertised foreign-collision safety.

### D4, expanded valid tab-stop elements are ignored

`crates/rdocx-oxml/src/borders.rs:320`
`crates/rdocx-oxml/src/borders.rs:329`
`crates/rdocx-oxml/src/borders.rs:330`
`crates/rdocx-oxml/src/borders.rs:337`
`crates/rdocx-oxml/src/borders.rs:467`
`.claude/plans/F-X007-design.md:55`
`.claude/plans/F-X007-design.md:64`

The parser recognizes tab stops only as `Event::Empty`. XML permits the same
empty-content OOXML value to be written as
`<q:tab q:val="left" q:pos="720"></q:tab>`. That form falls through the start
branch, receives no typed value or `source_occurrence`, and returns a shorter
collection. In numbering, an otherwise unrelated typed `pPr` edit can then
treat the valid source occurrence as absent rather than preserving and
overlaying it. The provenance and contextual parser contract must cover both
lexical forms of an empty schema element.

### D5, the public contextual paragraph parser delegates nested properties to local-name parsers

`crates/rdocx-oxml/src/properties.rs:121`
`crates/rdocx-oxml/src/properties.rs:133`
`crates/rdocx-oxml/src/properties.rs:135`
`crates/rdocx-oxml/src/properties.rs:139`
`crates/rdocx-oxml/src/properties.rs:143`
`crates/rdocx-oxml/src/properties.rs:570`
`crates/rdocx-oxml/src/properties.rs:595`
`crates/rdocx-oxml/src/properties.rs:663`
`crates/rdocx-oxml/README.md:40`

`CT_PPr::from_xml_with_prefixes` advertises prefix-aware paragraph-property
parsing, but only its top-level dispatch and tabs path use the supplied
namespace identities. Recognized `rPr`, `pBdr`, and `sectPr` containers are
delegated to their existing local-name parsers. For a Word `q:rPr` containing
foreign `<ext:b/>`, `CT_RPr::from_xml` matches only the local `b` and sets the
typed bold property. Its end condition is local-name-only too.

The newly public method therefore remains namespace-sensitive for part of the
model while its README presents a general contextual parsing path. Either its
nested modeled parsers need the same scope-aware contract or the public surface
must be narrowed honestly. The current behavior can materialize foreign
producer markup as typed Word state.

### D6, the authoritative HLD omits the approved public provenance expansion

`.claude/plans/F-X007-design.md:55`
`.claude/plans/F-X007-design.md:70`
`.claude/plans/F-X007-design.md:105`
`.claude/plans/F-X007-design.md:123`
`docs/hld/10-bindings-spec.md:211`
`docs/hld/10-bindings-spec.md:217`
`docs/hld/14-development-backlog.md:1186`
`docs/hld/14-development-backlog.md:1197`

The revised plan names `CT_TabStop::source_occurrence`, semantic equality, the
contextual parser methods, the foreign carrier, and the 0.5 migration as part
of the accepted story. It also lists HLD10 and HLD14 in the exact HLD work
list. HLD10 documents the older raw fields on `CT_Lvl`, `CT_AbstractNum`,
`CT_Num`, and `CT_Numbering`, but does not tell 0.4 callers about the new public
tab-stop field or parser surface. HLD14 still describes only the original PR,
the two maintainer regressions, and README compilation. Its story contract and
test gate omit the large numbering-preservation expansion entirely.

The crate README and design plan contain useful migration text, but they do not
replace the authoritative current-intent set. The HLD impact list remains
incomplete until both the public compatibility boundary and the expanded story
gate match the approved implementation scope.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml`: all 144 unit tests and the crate README doctest
  passed.
- `cargo test -p rdocx`: all 69 unit, 81 integration, 17 regression, and two
  doctests passed.
- `cargo clippy -p rdocx-oxml -p rdocx --all-targets --all-features -- -D
  warnings`: passed.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and the exact CLI and dependency snippet checks
  passed.
- `cargo package --allow-dirty --list -p <package>` for all seven stable
  packages: every inventory contains exactly one intended README.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- `gh pr view 25` confirms that PR 25 remains merged into `sprint/s38` at
  `6aade64`, the three contributor commits retain Jon Stokes as author, and the
  public merge note thanks and credits `@jonstokes` while explaining the fix.

## Not found

The public struct literals in the workspace all supply the new field, the
constructor initializes new occurrences to `None`, and semantic equality
continues to compare only typed values. When the overlay is entered, invalid or
duplicate provenance claims cannot index out of bounds, the first duplicate
claim wins once, and the matcher plus writer retain the tested near-linear
10,000-occurrence work bound. The dedicated carrier uses a non-modelled local
name, carries tested producer attributes and nested XML, and has collision-free
preservation and markup-compatibility prefixes selected from declarations
across the complete numbering model. No further defect was found in fixed
OOXML property slots, recursive unsupported XML preservation, the 64-element
depth error, canonical numbering prefix generation, public list and table
bounds, contributor APIs, README examples, package inventories, deterministic
hashes, or PR authorship and credit. No separate smell or nitpick was found.
