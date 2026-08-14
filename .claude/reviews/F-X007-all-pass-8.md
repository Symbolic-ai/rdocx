# F-X007, all aspects, pass 8

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all seven earlier remediation
rounds. The reviewed state is 43 files and 4,603 changed or new line entries:
30 tracked files with 3,216 additions and 298 deletions, plus 1,089 lines in
thirteen untracked files.
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, mutating a composite property still drops its nested producer XML

`crates/rdocx-oxml/src/numbering.rs:979`
`crates/rdocx-oxml/src/numbering.rs:999`
`crates/rdocx-oxml/src/numbering.rs:1013`
`crates/rdocx-oxml/src/numbering.rs:1064`
`crates/rdocx-oxml/src/numbering.rs:1073`
`crates/rdocx-oxml/src/numbering.rs:1078`
`crates/rdocx-oxml/src/numbering.rs:2662`
`crates/rdocx-oxml/src/numbering.rs:2675`
`docs/hld/04-opc-and-packaging.md:107`
`docs/hld/04-opc-and-packaging.md:111`

The new projection correctly notices unsupported WordprocessingML content, but
the mutation overlay preserves only attributes when the same direct modeled
child changes. `merge_property_child_attributes` reads the original direct
child start tag and copies selected attributes onto the generated child. It
does not merge the original child's nested events. For example, parsing
`<w:numPr><w:numId w:val="1"/><w:producer/></w:numPr>` and then changing
`ppr.num_id` makes the canonical `numPr` differ. The serializer takes the
attribute-only merge branch and loses `w:producer`. The same failure applies to
nested producer content inside a changed `pBdr`, `tabs`, nested `rPr`, or
`sectPr`. The pass-7 regression puts the producer child in `numPr` but changes
the separate `ind` child, so raw `numPr` is reused and the destructive branch
is not exercised. This still contradicts the producer-extension preservation
contract.

### D2, the CT_RPr schema slot table omits the schema-final rPrChange child

`crates/rdocx-oxml/src/numbering.rs:56`
`crates/rdocx-oxml/src/numbering.rs:95`
`crates/rdocx-oxml/src/numbering.rs:96`
`crates/rdocx-oxml/src/numbering.rs:919`
`crates/rdocx-oxml/src/numbering.rs:947`
`crates/rdocx-oxml/src/numbering.rs:961`
`crates/rdocx-oxml/src/numbering.rs:1060`
`.claude/plans/F-X007-design.md:93`

The expanded `RPR_CHILDREN` table stops at `oMath`, but the
WordprocessingML `CT_RPr` sequence has schema-final `rPrChange`. The overlay
therefore handles a valid `w:rPrChange` as an unknown pending child instead of
assigning it the final schema slot. If the source contains only `rPrChange` and
a typed edit adds bold, the pending child is assigned boundary zero and is
written before `w:b`. If it follows an existing modeled child and a later typed
child is added, it is likewise replayed before that new child. Both outputs
violate the `CT_RPr` sequence. The ordering fixture covers the middle sequence
around strike, vanish, highlight, and underline, but not the schema-final
revision child.

### D3, non-property numbering parsing still treats every unprefixed name as WordprocessingML

`crates/rdocx-oxml/src/numbering.rs:400`
`crates/rdocx-oxml/src/numbering.rs:405`
`crates/rdocx-oxml/src/numbering.rs:423`
`crates/rdocx-oxml/src/numbering.rs:424`
`crates/rdocx-oxml/src/numbering.rs:503`
`crates/rdocx-oxml/src/numbering.rs:1453`
`crates/rdocx-oxml/src/numbering.rs:1458`
`crates/rdocx-oxml/src/numbering.rs:1716`
`crates/rdocx-oxml/src/numbering.rs:1717`
`crates/rdocx-oxml/src/numbering.rs:1721`
`crates/rdocx-oxml/src/numbering.rs:2687`
`crates/rdocx-oxml/src/numbering.rs:2692`

The new property-specific helpers correctly distinguish an unprefixed element
through the default namespace and leave an unprefixed attribute in no
namespace. The rest of the numbering parser still uses `is_word_attribute` for
both element names and attributes, and its first branch accepts any
unprefixed local-name match without consulting namespace scope. A foreign
`<abstractNum xmlns="urn:producer">` is consequently parsed as a modeled
abstract definition. A no-namespace `abstractNumId="7"` placed before
`w:abstractNumId="0"` can control the typed ID and is then removed from the
preserved attribute list. The same collision exists for level, instance, and
scalar value attributes. The new default-foreign test is confined to a child
inside a recognized `w:pPr` and uses the noncolliding unqualified name
`producer`, so it does not cover these model paths. Expanded namespace identity
therefore remains inconsistent outside the property projector.

### D4, the published struct-literal migration names a field CT_Numbering does not have

`crates/rdocx-oxml/README.md:26`
`crates/rdocx-oxml/README.md:27`
`crates/rdocx-oxml/README.md:30`
`crates/rdocx-oxml/README.md:31`
`crates/rdocx-oxml/src/numbering.rs:1645`
`crates/rdocx-oxml/src/numbering.rs:1649`
`crates/rdocx-oxml/src/numbering.rs:1651`
`docs/hld/10-bindings-spec.md:211`
`docs/hld/10-bindings-spec.md:214`

The new migration section applies one literal recipe to `CT_Lvl`,
`CT_AbstractNum`, `CT_Num`, and `CT_Numbering`, telling callers to initialize
`extra_xml` and `extra_attributes`. `CT_Numbering` has no `extra_attributes`
field. Its added fields are `root_attributes` and `extra_xml`. Following the
published recipe for a required `CT_Numbering` literal therefore fails to
compile and still leaves the real new field unexplained. The HLD records the
0.5.0 boundary but does not correct the field-level recipe. The migration needs
per-struct field guidance, or an explicit `CT_Numbering::new` update pattern,
to satisfy the approved public API rider.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 30 focused tests passed.
- `cargo test -p rdocx-oxml`: all 128 unit tests and the crate README doctest
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
- `gh pr view 25` confirms the PR targets `sprint/s38`, is merged at
  `6aade64`, credits `@jonstokes`, and retains the public valuable-fix note.
  Local history retains Jon Stokes as author of all three contributor commits.

## Not found

The projector now rejects property nesting beyond its 64-element bound with a
normal `OxmlError::InvalidValue`. Default-foreign property descendants and
noncolliding no-namespace property attributes remain raw. The modeled
`CT_PPr` writer order and the covered middle of the `CT_RPr` writer order match
their schema slots. HLD14, `CURRENT_SPRINT.md`, `SPRINT_PLAN.md`, and
`BACKLOG.md` consistently name F-X008 as `Tag v0.5.0` and require the 0.5.0
release boundary. Manifest README wiring, stable archive inventories, README
compilation, package size evidence, public authoring bounds, table geometry,
hash stability, and PR credit show no additional defect. No smell or nitpick
was found.
