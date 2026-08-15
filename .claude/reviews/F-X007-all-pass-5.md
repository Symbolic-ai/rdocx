# F-X007, all aspects, pass 5

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all four earlier remediation
rounds. The reviewed state is 37 files and 2,589 changed or new line entries:
27 tracked files with 1,766 additions and 224 deletions, plus 599 lines in ten
untracked files.
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, locally rebound prefixes remain classified as WordprocessingML

`crates/rdocx-oxml/src/numbering.rs:76`
`crates/rdocx-oxml/src/numbering.rs:85`
`crates/rdocx-oxml/src/numbering.rs:116`

`word_prefixes_at` clones every inherited Word prefix and only adds bindings.
It never removes a prefix that the current element rebinds to another
namespace. For example, with `q` bound to WordprocessingML at the numbering
root, a valid level can locally bind `q` to a producer namespace and put
`q:ilvl="producer"` before `w:ilvl="0"`. The resolver still treats the locally
foreign `q:ilvl` as the modelled attribute and attempts to parse `producer` as
an integer. The same stale binding can make a locally foreign element take a
modelled scalar or container path. The new collision test at
`crates/rdocx-oxml/src/numbering.rs:1288` adds an alias but never shadows one,
so it does not cover namespace scope exit or rebinding. This violates the
prefix-tolerant identity contract at `docs/hld/04-opc-and-packaging.md:107`.

### D2, canonical root prefixes can silently replace foreign source bindings

`crates/rdocx-oxml/src/numbering.rs:40`
`crates/rdocx-oxml/src/numbering.rs:41`
`crates/rdocx-oxml/src/numbering.rs:821`
`crates/rdocx-oxml/src/numbering.rs:827`

Root attribute capture drops `xmlns:w` and `xmlns:r` by spelling without
checking their namespace values, then serialization always binds those names
to the canonical Word and relationship namespaces. Prefix-tolerant input can
legitimately use `q` for WordprocessingML while binding `w` to an ignorable
producer namespace. A preserved raw `<w:extension/>` child then changes from
the producer namespace to WordprocessingML on output. The same issue applies
to a foreign `r` binding used by raw content. Fixed-prefix modelled output does
not permit changing the expanded names of unmodelled XML. This contradicts the
verbatim producer-extension contract at
`docs/hld/04-opc-and-packaging.md:107`.

### D3, raw-child schema slots are selected by local name alone

`crates/rdocx-oxml/src/numbering.rs:127`
`crates/rdocx-oxml/src/numbering.rs:143`
`crates/rdocx-oxml/src/numbering.rs:157`
`crates/rdocx-oxml/src/numbering.rs:366`
`crates/rdocx-oxml/src/numbering.rs:550`

Modelled element and attribute paths now consult the Word namespace bindings,
but all three raw-boundary helpers still use `matches_local_name`. A foreign
ignorable child named `ext:lvlRestart`, `ext:tmpl`, or `ext:numPicBullet` is
therefore assigned the fixed slot of the Word child with that local name. For
example, a producer `ext:numPicBullet` after an existing numbering instance is
emitted before every abstract definition, even without a mutation. A foreign
`ext:tmpl` after a level similarly moves before the levels. The raw bytes
survive, but their relative location does not, contrary to the insertion-aware
boundary contract at `docs/hld/04-opc-and-packaging.md:109`.

### D4, changing typed level properties deletes their preserved extensions

`crates/rdocx-oxml/src/numbering.rs:464`
`crates/rdocx-oxml/src/numbering.rs:477`
`crates/rdocx-oxml/src/properties.rs:249`
`crates/rdocx-oxml/src/properties.rs:661`

The new property snapshots reuse raw `w:pPr` or `w:rPr` only while the public
typed value compares equal to its parsed snapshot. If a low-level caller
changes a supported property, serialization discards the raw container and
calls the ordinary typed writer. Those writers construct fresh containers and
have nowhere to carry the producer attributes or nested extension subtrees.
Thus changing an indentation in a parsed `w:pPr` deletes its `ext:marker` and
`ext:pPrData`, while changing bold in `w:rPr` deletes the parallel run
extensions. The regression at `crates/rdocx-oxml/src/numbering.rs:1432` only
serializes unchanged typed snapshots. This all-or-nothing fallback violates
the requirement that mutating a definition preserve producer extensions at
`docs/hld/04-opc-and-packaging.md:110` and the direct low-level use advertised
at `crates/rdocx-oxml/README.md:3`.

### D5, private raw snapshots break the published CT_Lvl construction and equality contracts

`crates/rdocx-oxml/src/numbering.rs:271`
`crates/rdocx-oxml/src/numbering.rs:291`
`crates/rdocx-oxml/src/numbering.rs:357`
`crates/rdocx-oxml/src/lib.rs:27`

`CT_Lvl` is publicly re-exported from a published low-level crate and was an
all-public struct. Adding the private `ppr_raw` and `rpr_raw` fields makes every
external struct literal and struct-update expression fail to compile, which is
a source-breaking change in the stable package family. The private
serialization snapshots also participate in the derived `PartialEq`. A newly
constructed bullet definition has no `ppr_raw`, while parsing its own canonical
XML fills `ppr_raw`, so the two otherwise identical public models compare
unequal. Preservation bookkeeping must not make a patch release remove public
construction or change semantic equality. The README explicitly directs
low-level model owners to this crate at `crates/rdocx-oxml/README.md:3`.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 19 focused tests passed.
- `cargo test -p rdocx-oxml`: 117 unit tests and one README doctest passed.
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

The pass-4 resolver correctly ignores an ordinary foreign local-name collision
that precedes a Word attribute, accepts an alternate prefix bound to the Word
namespace, and removes the aliased modelled attribute before writing its
canonical form. Unchanged expanded and self-closing `w:pPr` and `w:rPr`
containers retain their producer attributes and nested subtrees byte for byte,
including when `set_list_level` changes only the scalar level fields. Expanded
`start`, `numFmt`, `lvlText`, `lvlJc`, `multiLevelType`, and `abstractNumId`
elements take their modelled paths and serialize once. No additional defect was
found in fixed abstract or root insertion boundaries for correctly identified
Word children, public list and paragraph bounds, identifier allocation,
hyperlink and hard-break behavior, staged table geometry, README compilation
and inventories, package wiring, HLD file scope, hash stability, or contributor
credit. No structural-rule violation, resource-bound smell, additional
public-input panic, or documentation nitpick was found.
