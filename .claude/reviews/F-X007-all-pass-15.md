# F-X007, all aspects, pass 15

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all fourteen earlier review and
remediation rounds. The reviewed state is 52 files and 8,412 changed or new
line entries: 32 tracked files with 5,543 additions and 714 deletions, plus
2,155 lines in twenty untracked files.
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, narrowing `CT_PPr` removed existing prefix tolerance from non-numbering callers

`crates/rdocx-oxml/src/properties.rs:117`
`crates/rdocx-oxml/src/properties.rs:118`
`crates/rdocx-oxml/src/properties.rs:132`
`crates/rdocx-oxml/src/properties.rs:150`
`crates/rdocx-oxml/src/styles.rs:95`
`crates/rdocx-oxml/src/styles.rs:96`
`crates/rdocx-oxml/src/text.rs:342`
`crates/rdocx-oxml/src/text.rs:343`
`crates/rdocx-oxml/README.md:3`
`docs/hld/10-bindings-spec.md:223`
`docs/hld/10-bindings-spec.md:226`

Removing the newly exposed contextual `CT_PPr` method was appropriate, and the
numbering projection now gives the canonical parser safe `w:` XML. The public
`CT_PPr::from_xml` entry point was not restored to its prior prefix-tolerant
behavior, though. It now hard-codes `w` as the only inherited Word prefix and
filters every child and attribute through that list.

The styles and document-text parsers still identify `pPr` by local name before
calling this public method. A valid aliased paragraph such as
`<q:pPr><q:jc q:val="center"/></q:pPr>`, with `q` bound to WordprocessingML on
an ancestor, reaches `CT_PPr::from_xml` and returns no justification. The same
loss affects styles, numbering references, indentation, tabs, and other
paragraph properties outside the private numbering projection. This regresses
the crate's advertised prefix-tolerant parsing and is not part of the approved
0.5 field migration. Narrowing the new surface must not narrow the established
parser behavior used by the rest of the crate.

### D2, `mc:Ignorable` expanded-name lookup uses global declaration order instead of ancestor scope

`crates/rdocx-oxml/src/numbering.rs:308`
`crates/rdocx-oxml/src/numbering.rs:320`
`crates/rdocx-oxml/src/numbering.rs:347`
`crates/rdocx-oxml/src/numbering.rs:350`
`crates/rdocx-oxml/src/numbering.rs:388`
`crates/rdocx-oxml/src/numbering.rs:395`
`crates/rdocx-oxml/src/numbering.rs:2523`
`crates/rdocx-oxml/src/numbering.rs:2533`
`.claude/plans/F-X007-design.md:69`
`.claude/plans/F-X007-design.md:73`
`docs/hld/14-development-backlog.md:1200`

The flat pass-14 fixture is fixed, but `namespace_for` resolves a non-local
prefix by taking the first matching declaration from the complete document
list. That list is deliberately global and ordered from root declarations to
abstract and level declarations. It is not the in-scope namespace chain of the
container being rewritten.

For example, let the numbering root bind `compat` to the markup-compatibility
namespace, then let the containing `w:lvl` rebind `compat` to a producer
namespace. A descendant `w:tabs compat:Ignorable="producer"` is a producer
attribute in its actual scope. During an explicit clear, the resolver finds the
root binding first, misclassifies the attribute as expanded-name
`mc:Ignorable`, appends the preservation token, and suppresses creation of a
real compatibility attribute. The output retains the nearer level binding, so
the merged attribute is still in the producer namespace and the generated
carrier is not ignorable. Reversing the two bindings can instead preserve the
real attribute and then emit a duplicate after locally rebinding its prefix.

Complete declaration collection prevents generated prefix collisions, but it
cannot resolve an individual attribute's expanded name without its actual
ancestor scope. The compatibility merge needs the scoped bindings captured for
that property container.

### D3, nested tab namespace tracking has quadratic memory and copy work

`crates/rdocx-oxml/src/borders.rs:255`
`crates/rdocx-oxml/src/borders.rs:256`
`crates/rdocx-oxml/src/borders.rs:272`
`crates/rdocx-oxml/src/borders.rs:319`
`crates/rdocx-oxml/src/borders.rs:332`
`crates/rdocx-oxml/src/borders.rs:338`
`.claude/plans/F-X007-design.md:63`
`.claude/plans/F-X007-design.md:64`

The new iterative scope stack correctly isolates the tested prefix and default
namespace shadows, but each start element clones the complete inherited
`Vec<String>` and retains that clone until the corresponding end. A valid
unknown subtree can declare one additional WordprocessingML alias at every
depth. At depth `d`, the stack then owns vectors of lengths 1 through `d`, and
constructing them copies the same cumulative prefix strings. Memory and copy
work are quadratic in input depth, with no parser depth limit.

This is reachable through the public XML parser without creating any tab
stops, so the 10,000-occurrence linear matcher gate does not cover it. Namespace
scope should be represented as bounded deltas or the parser should reject
excessive nesting with a normal error, as the numbering property projector
already does.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml`: all 148 unit tests and the crate README doctest
  passed.
- `cargo test -p rdocx`: all 69 unit, 81 integration, 17 regression, and two
  doctests passed.
- `cargo clippy -p rdocx-oxml -p rdocx --all-targets --all-features -- -D
  warnings`: passed.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and the inventory and snippet contracts passed.
- `cargo package --allow-dirty --list -p <package>` for all seven stable
  packages: every inventory contains exactly one intended README. The current
  `rdocx-oxml` dry-run evidence records an 82,094-byte archive, below 10 MiB.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- `gh pr view 25` confirms that PR 25 remains merged into `sprint/s38` at
  `6aade64`, Jon Stokes remains the contributor, and the public note credits
  `@jonstokes` while explaining the value and maintainer hardening.

## Not found

The pass-14 provenance fix now checks source-occurrence vectors at both raw
reuse boundaries, so a semantically identical new tab no longer inherits the
old producer payload. Duplicate and out-of-range provenance claims remain safe
and deterministic, and the repeated-property matcher and writer keep their
tested 10,000-occurrence linear work bound. The public tab parser correctly
handles direct empty and expanded tabs, ignores nested foreign tab subtrees,
and does not terminate on a shadowed nested `tabs` end. Numbering projection
normalizes modeled Word property XML before the canonical `CT_PPr` parser and
keeps foreign nested run properties out of typed state. No further defect was
found in the carrier local name, flat and local expanded-name `mc:Ignorable`
merging, schema slots, unsupported XML preservation, the 64-element property
depth error, public list and table bounds, HLD10 and HLD14 migration scope,
README examples, package inventories, deterministic hashes, or PR authorship
and credit. No separate smell or nitpick was found.
