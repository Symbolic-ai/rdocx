# F-X007, all aspects, pass 12

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all eleven earlier remediation
rounds. The reviewed state is 47 files and 6,982 changed or new line entries:
30 tracked files with 4,688 additions and 646 deletions, plus 1,648 lines in
seventeen untracked files.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, explicit clear still drops producer attributes that cannot supply a foreign carrier prefix

`crates/rdocx-oxml/src/numbering.rs:1154`
`crates/rdocx-oxml/src/numbering.rs:1155`
`crates/rdocx-oxml/src/numbering.rs:1156`
`crates/rdocx-oxml/src/numbering.rs:1162`
`crates/rdocx-oxml/src/numbering.rs:1164`
`crates/rdocx-oxml/src/numbering.rs:1176`
`crates/rdocx-oxml/src/numbering.rs:1181`
`crates/rdocx-oxml/src/numbering.rs:3347`
`crates/rdocx-oxml/src/numbering.rs:3366`
`crates/rdocx-oxml/src/numbering.rs:3372`
`crates/rdocx-oxml/src/numbering.rs:3637`
`.claude/plans/F-X007-design.md:49`
`.claude/plans/F-X007-design.md:51`

The pass-11 carrier path recognizes both an unsupported Word-qualified
attribute and a no-namespace attribute as producer data, but it chooses a
carrier only from an attribute with a prefixed, non-Word QName. If a cleared
`w:tab` carries `producer="keep"`, there is no separator and therefore no
carrier prefix. If it carries `w:producer="keep"`, its prefix is deliberately
excluded because it is an in-scope Word prefix. In both cases
`producer_attributes` is nonempty, but the no-carrier branch emits only nested
children. With no nested child it returns `None`, losing the attribute
completely.

The earlier regressions expressly establish that unsupported Word attributes
and no-namespace attributes are unmodelled data. The new clear regression uses
only `ext:id`, which can provide the foreign carrier prefix, so it does not
exercise either failing form. Explicitly clearing the typed tab collection
must not make these already-supported producer attribute identities disappear.

### D2, the monotonic matcher misassigns edits that precede a later insertion or removal

`crates/rdocx-oxml/src/numbering.rs:1467`
`crates/rdocx-oxml/src/numbering.rs:1469`
`crates/rdocx-oxml/src/numbering.rs:1473`
`crates/rdocx-oxml/src/numbering.rs:1475`
`crates/rdocx-oxml/src/numbering.rs:1477`
`crates/rdocx-oxml/src/numbering.rs:1478`
`crates/rdocx-oxml/src/numbering.rs:1528`
`crates/rdocx-oxml/src/numbering.rs:1540`
`crates/rdocx-oxml/src/numbering.rs:3740`
`crates/rdocx-oxml/src/numbering.rs:3771`
`crates/rdocx-oxml/src/numbering.rs:3781`
`.claude/plans/F-X007-design.md:50`
`.claude/plans/F-X007-design.md:77`

At every mismatch, the matcher uses only the remaining-length difference to
decide whether the current original or generated occurrence is the unmatched
one. For original identities `A, B`, editing `A` and appending `X` produces
`edited-A, B, X`. Because the generated side is longer at the first mismatch,
the algorithm marks `edited-A` as the insertion. It then pairs original `A`
with generated `B` through the equal-length fallback and original `B` with
generated `X`. The writer consequently drops `A`'s producer data from the
edited occurrence, transfers it to `B`, transfers `B`'s data to `X`, and
replays the original boundary against the wrong neighbors. Editing an earlier
occurrence and removing a later one fails symmetrically.

The new regression covers an insertion before the edited occurrence, which is
the one ordering the remaining-length heuristic identifies correctly. It does
not cover an edit before a later insertion or removal, even though an unchanged
later identity provides a deterministic anchor. The matcher remains linear,
but it does not yet retain producer ownership for the complete mixed-edit
contract.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 41 focused tests passed,
  including the complete 10,000-tab path with 9,999 producer boundaries.
- `cargo test -p rdocx-oxml`: all 139 unit tests and the crate README doctest
  passed.
- `cargo test -p rdocx`: all 69 unit, 81 integration, 17 regression, and two
  doctests passed.
- `cargo clippy -p rdocx-oxml --all-targets --all-features -- -D warnings`:
  passed.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and the CLI and dependency snippet contracts
  passed.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: every inventory contains exactly one intended README.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- `gh pr view 25` confirms the PR remains merged into `sprint/s38` at
  `6aade64`, all three contributor commits retain Jon Stokes as author, and the
  public comment credits `@jonstokes` and explains the value of the fix.

## Not found

The pass-11 foreign-prefixed carrier prevents a cleared modelled tab shell from
reappearing as a typed default, and occurrence-specific projection retains the
tested producer payload boundaries. Generated repeated containers use matching
start and end QNames, inherit container-local aliases, and retain collision-safe
expanded identities. The bucketed writer and monotonic boundary cursor remove
the previously identified quadratic rescans, and the measured 10,000-occurrence
gate includes a producer node at every internal boundary. No additional defect
was found in recursive unsupported XML preservation, fixed OOXML schema slots,
default and aliased namespace handling, the 64-element depth error, public
authoring bounds, table geometry, hyperlinks, hard breaks, 0.5.0 migration
documentation, HLD and sprint scope, README quality, package wiring, hashes, or
PR credit. No separate smell or nitpick was found.
