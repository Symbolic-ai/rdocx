# F-X007, all aspects, pass 11

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all ten earlier remediation
rounds. The reviewed state is 46 files and 5,862 changed or new line entries:
30 tracked files with 4,060 additions and 300 deletions, plus 1,502 lines in
sixteen untracked files.
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, producer-only tab shells undo an explicit clear on reparse

`crates/rdocx-oxml/src/numbering.rs:1148`
`crates/rdocx-oxml/src/numbering.rs:1156`
`crates/rdocx-oxml/src/numbering.rs:1197`
`crates/rdocx-oxml/src/numbering.rs:1203`
`crates/rdocx-oxml/src/numbering.rs:3497`
`crates/rdocx-oxml/src/numbering.rs:3508`
`crates/rdocx-oxml/src/borders.rs:192`
`crates/rdocx-oxml/src/borders.rs:210`
`crates/rdocx-oxml/src/borders.rs:228`
`.claude/plans/F-X007-design.md:50`
`.claude/plans/F-X007-design.md:93`

The producer-only projection removes the supported `w:val` and `w:pos`
attributes but keeps the modelled `w:tab` shell whenever that tab has a
producer attribute. The new regression explicitly requires
`<w:tab ext:id="a"/>`. Both typed tab fields are non-optional, and the parser
defaults a missing pair to `Left` at zero twips. Saving after `tabs = None` and
reparsing therefore materializes a typed tab at position zero, so the explicit
clear does not survive its first round trip. The emitted shell also lacks the
required WordprocessingML tab-stop attributes. Producer projection cannot keep
a modelled leaf shell that the typed parser will claim as a new value.

### D2, producer-only repeated children lose their occurrence boundaries

`crates/rdocx-oxml/src/numbering.rs:930`
`crates/rdocx-oxml/src/numbering.rs:998`
`crates/rdocx-oxml/src/numbering.rs:1008`
`crates/rdocx-oxml/src/numbering.rs:1177`
`crates/rdocx-oxml/src/numbering.rs:1184`
`crates/rdocx-oxml/src/numbering.rs:1204`
`crates/rdocx-oxml/src/numbering.rs:1212`
`crates/rdocx-oxml/src/numbering.rs:3486`
`.claude/plans/F-X007-design.md:46`
`.claude/plans/F-X007-design.md:77`

Explicit-clear projection routes `tabs/tab` through the generic schema-slot
overlay rather than the occurrence overlay. Every tab has schema position
zero. Producer nodes between tabs are also assigned position zero when the
next tab drains the pending list, and serialization writes all extras at that
position before all projected children. A direct probe of
`tab-a, ext:between, tab-b` followed by `tabs = None` produced
`ext:between, tab-a, tab-b`. The extension no longer remains between the two
producer-bearing occurrences. The pass-10 clear fixture has only one tab, so
it cannot detect this collapse.

### D3, exact-first matching can attach producer state to the wrong occurrence

`crates/rdocx-oxml/src/numbering.rs:1320`
`crates/rdocx-oxml/src/numbering.rs:1329`
`crates/rdocx-oxml/src/numbering.rs:1333`
`crates/rdocx-oxml/src/numbering.rs:1342`
`crates/rdocx-oxml/src/numbering.rs:1350`
`crates/rdocx-oxml/src/numbering.rs:1381`
`crates/rdocx-oxml/src/numbering.rs:1394`
`crates/rdocx-oxml/src/numbering.rs:3515`
`.claude/plans/F-X007-design.md:50`
`.claude/plans/F-X007-design.md:77`

The matcher assigns every exact identity before it pairs remaining occurrences
in order. If the first of two tabs is edited from position 720 to the second
tab's position 1,440, the first generated occurrence consumes the second
original occurrence from that identity queue. Ordered fallback then assigns
the first original occurrence to the second generated tab. A direct probe
showed `ext:id="b"` on the first output tab and `ext:id="a"` on the second,
with the original between-node moved before both. An insert plus a typed edit
has the corresponding failure: the inserted tab receives the edited original
tab's producer attribute, while the edited tab loses it. The single-occurrence
regression cannot exercise identity collisions or mixed insertion and edit.

### D4, the complete repeated-property writer remains quadratic

`crates/rdocx-oxml/src/numbering.rs:373`
`crates/rdocx-oxml/src/numbering.rs:378`
`crates/rdocx-oxml/src/numbering.rs:1315`
`crates/rdocx-oxml/src/numbering.rs:1356`
`crates/rdocx-oxml/src/numbering.rs:1382`
`crates/rdocx-oxml/src/numbering.rs:1386`
`crates/rdocx-oxml/src/numbering.rs:1388`
`crates/rdocx-oxml/src/numbering.rs:1390`
`crates/rdocx-oxml/src/numbering.rs:3571`
`crates/rdocx-oxml/src/numbering.rs:3578`

The hash-map matcher is near-linear, but the writer rescans the boundary bitmap
from zero through each matched original index. Ordered N-tab input therefore
visits one plus two through N boundary entries. Each newly emitted boundary
also calls `write_extras_at`, which scans the complete extras collection, so a
producer node between each tab adds another quadratic path. The regression's
work counter covers only `match_occurrences`. Its complete 10,000-tab path has
no counter or time bound and contains no between-occurrence extras. It can pass
while the public serialization operation still performs quadratic work.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 39 focused tests passed.
- `cargo test -p rdocx-oxml`: all 137 unit tests and the crate README doctest
  passed.
- `cargo test -p rdocx`: all 69 unit, 81 integration, 17 regression, and two
  doctests passed.
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

Simple typed mutation of one tab retains that occurrence's producer attribute.
Canonical and locally aliased tab insertion use matching generated start and
end QNames. A collision fixture that occupies `w` and `w1` selects `w2`,
retains the aliased original tab attribute, serializes matching `w2:tabs`
QNames, and reparses successfully. Container-local alias propagation is
correct. The hash-map and queue matcher itself is deterministic and
near-linear. No additional namespace, fixed schema-slot, `rPrChange`, default
namespace, no-namespace attribute, or 64-element depth regression was found.
The public 0.5.0 migration recipes, HLD14 and sprint records, README and
manifest contracts, package inventories, authoring bounds, table geometry,
hash stability, and PR credit remain consistent. No separate smell or nitpick
was found.
