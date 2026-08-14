# F-X007, all aspects, pass 13

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all twelve earlier remediation
rounds. The reviewed state is 48 files and 7,283 changed or new line entries:
30 tracked files with 4,871 additions and 642 deletions, plus 1,770 lines in
eighteen untracked files.
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, the preservation carrier reparses as a typed tab through the public property parser

`crates/rdocx-oxml/src/numbering.rs:1224`
`crates/rdocx-oxml/src/numbering.rs:1229`
`crates/rdocx-oxml/src/numbering.rs:1467`
`crates/rdocx-oxml/src/borders.rs:210`
`crates/rdocx-oxml/src/borders.rs:211`
`crates/rdocx-oxml/src/borders.rs:212`
`crates/rdocx-oxml/src/borders.rs:245`
`crates/rdocx-oxml/src/borders.rs:246`
`crates/rdocx-oxml/src/numbering.rs:3786`
`crates/rdocx-oxml/src/numbering.rs:3787`
`crates/rdocx-oxml/README.md:3`

The generated carrier deliberately keeps the removed leaf's local name, so a
cleared tab becomes `rdocxPreserve:tab`. The numbering-specific projector
correctly treats that QName as foreign, but the public `CT_Tabs::from_xml`
parser dispatches on local name alone. It therefore calls
`CT_TabStop::from_xml_attrs` for the carrier and materializes its defaults,
Left at zero twips. Direct `CT_PPr::from_xml` use has the same path through
`CT_Tabs`.

The new regression reparses only through `CT_Numbering`, whose private
namespace-aware projection masks the public parser behavior. The low-level
crate explicitly advertises direct typed XML ownership, so XML produced by its
numbering serializer cannot resurrect a cleared typed value when parsed
through another public model parser.

### D2, the library-created foreign carrier is not markup-compatibility ignorable

`crates/rdocx-oxml/src/numbering.rs:18`
`crates/rdocx-oxml/src/numbering.rs:1224`
`crates/rdocx-oxml/src/numbering.rs:1227`
`crates/rdocx-oxml/src/numbering.rs:1229`
`crates/rdocx-oxml/src/numbering.rs:1467`
`crates/rdocx-oxml/src/borders.rs:266`
`crates/rdocx-oxml/src/borders.rs:270`
`crates/rdocx-oxml/src/borders.rs:276`
`crates/rdocx-oxml/src/numbering.rs:3766`
`crates/rdocx-oxml/src/numbering.rs:3781`
`.claude/plans/F-X007-design.md:93`
`.claude/plans/F-X007-design.md:94`

`CT_Tabs` is a sequence of WordprocessingML `w:tab` children. The remediation
inserts a new element in `urn:rdocx:preserved-property` directly into that
sequence and declares only its namespace. It does not put the new prefix in an
in-scope `mc:Ignorable` declaration. The source fixture has no markup
compatibility declaration either. Consequently this is not merely preserved
producer markup of unknown validity. The serializer itself creates a foreign
child outside the declared OOXML sequence without the compatibility instruction
that permits an unaware consumer to discard it.

The regression proves only that the repository's tolerant parser accepts the
result. It does not prove the schema and external-consumer requirement in the
parser or serializer risk rider. A generated carrier must be safe for Word and
other conforming consumers, not only for the private round-trip path.

### D3, carrier prefix allocation can shadow an inherited producer binding

`crates/rdocx-oxml/src/numbering.rs:1133`
`crates/rdocx-oxml/src/numbering.rs:1139`
`crates/rdocx-oxml/src/numbering.rs:1147`
`crates/rdocx-oxml/src/numbering.rs:1149`
`crates/rdocx-oxml/src/numbering.rs:1160`
`crates/rdocx-oxml/src/numbering.rs:1166`
`crates/rdocx-oxml/src/numbering.rs:1222`
`crates/rdocx-oxml/src/numbering.rs:1229`
`crates/rdocx-oxml/src/numbering.rs:3766`
`crates/rdocx-oxml/src/numbering.rs:3769`
`.claude/plans/F-X007-design.md:46`
`.claude/plans/F-X007-design.md:48`

The prefix allocator scans only namespace declarations and QName spellings
inside the removed leaf subtree. It receives no complete ancestor namespace
scope. Suppose an ancestor binds `rdocxPreserve` to `urn:producer` and the tab
has a no-namespace producer attribute whose QName-valued content is
`rdocxPreserve:token`. The leaf contains no declaration or QName using that
prefix, so the allocator selects `rdocxPreserve` and locally rebinds it to the
rdocx preservation namespace. The copied attribute bytes are unchanged, but
its QName value now resolves to the wrong expanded identity.

The regression declares the occupied prefix directly on the tab, which the
subtree scan can see. It does not cover an inherited non-Word binding or
producer QName-valued attributes and text. A preservation prefix must avoid
the complete in-scope namespace set rather than shadowing producer semantics
that the model intentionally does not parse.

### D4, anchors do not resolve an edit and later collection change inside the same segment

`crates/rdocx-oxml/src/numbering.rs:1520`
`crates/rdocx-oxml/src/numbering.rs:1524`
`crates/rdocx-oxml/src/numbering.rs:1528`
`crates/rdocx-oxml/src/numbering.rs:1538`
`crates/rdocx-oxml/src/numbering.rs:1545`
`crates/rdocx-oxml/src/numbering.rs:1547`
`crates/rdocx-oxml/src/numbering.rs:1550`
`crates/rdocx-oxml/src/numbering.rs:1626`
`crates/rdocx-oxml/src/numbering.rs:3918`
`crates/rdocx-oxml/src/numbering.rs:3940`
`crates/rdocx-oxml/src/numbering.rs:3941`
`crates/rdocx-oxml/src/numbering.rs:3967`
`crates/rdocx-oxml/src/numbering.rs:3968`
`.claude/plans/F-X007-design.md:50`
`.claude/plans/F-X007-design.md:77`

Unique unchanged identities now partition the sequence correctly, but each
intervening segment still uses the pass-11 remaining-length heuristic. For
original `A, B` and generated `edited-A, X, B`, unique `B` anchors the segment
at original index 1 and generated index 2. Within the preceding segment the
generated side is longer, so the first mismatch marks `edited-A` as the
insertion. The equal-length fallback then pairs original `A` with `X`. Producer
state is dropped from the edited occurrence and transferred to the inserted
one. Original `A, X, B` to generated `edited-A, B` fails symmetrically by
treating `A` as the removal and transferring `X`'s state to the edit.

The regression appends after the unchanged `B` and `C` anchors, or pops after
an unchanged `B` anchor. The typed edit and collection change are therefore in
different segments. It does not cover the same-segment case, repeated-identity
segments without a unique anchor, or an insertion or removal immediately
before the next anchor. The matcher remains linear, but occurrence ownership
is still incorrect for ordinary mixed vector edits.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 43 focused tests passed,
  including the anchored alignment and 10,000-tab work gates.
- `cargo test -p rdocx-oxml`: all 141 unit tests and the crate README doctest
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

The generated carrier retains the tested Word-qualified and no-namespace
attributes through the numbering-specific parse path and avoids every prefix
declared or used directly in the captured leaf subtree. Unique unchanged
identities anchor the tested append and pop operations. The matcher, boundary
writer, and 10,000-occurrence path retain explicit linear work bounds with a
producer node at every internal boundary. No additional defect was found in
recursive unsupported XML preservation, fixed OOXML schema slots outside the
new carrier, default and aliased Word namespace handling, collision-safe model
prefixes, the 64-element depth error, public authoring bounds, table geometry,
hyperlinks, hard breaks, 0.5.0 migration documentation, HLD and sprint scope,
README quality, package inventory, hashes, or PR credit. No separate smell or
nitpick was found.
