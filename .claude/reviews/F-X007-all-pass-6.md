# F-X007, all aspects, pass 6

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all five earlier remediation
rounds. The reviewed state is 38 files and 3,381 changed or new line entries:
27 tracked files with 2,382 additions and 255 deletions, plus 744 lines in
eleven untracked files.
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, a descendant namespace declaration can shadow the selected generated prefix

`crates/rdocx-oxml/src/numbering.rs:124`
`crates/rdocx-oxml/src/numbering.rs:835`
`crates/rdocx-oxml/src/numbering.rs:1010`
`crates/rdocx-oxml/src/numbering.rs:1122`
`crates/rdocx-oxml/src/numbering.rs:1248`

`generated_prefix` checks only root attributes. The modeled container writers
then append every preserved local namespace declaration without checking the
chosen prefix. For example, a root with foreign `w`, occupied `w1`, and a
modeled `q:abstractNum` carrying `xmlns:w2="urn:producer"` makes the selector
choose `w2`. The writer emits `w2:abstractNum` and `w2:abstractNumId` on the
same start tag that locally rebinds `w2` to the producer namespace. XML
namespace declarations apply to the complete start tag, so those generated
names are no longer WordprocessingML. The same failure is available on level
and instance containers, and inside a changed property overlay. The root-only
collision test does not exercise a declaration on a modeled descendant. This
violates the expanded-identity contract in
`.claude/plans/F-X007-design.md:46`.

### D2, property parsing and overlay exclusion still use local names instead of expanded identity

`crates/rdocx-oxml/src/numbering.rs:377`
`crates/rdocx-oxml/src/numbering.rs:423`
`crates/rdocx-oxml/src/numbering.rs:460`
`crates/rdocx-oxml/src/numbering.rs:554`
`crates/rdocx-oxml/src/properties.rs:123`
`crates/rdocx-oxml/src/properties.rs:166`
`crates/rdocx-oxml/src/properties.rs:551`
`crates/rdocx-oxml/src/properties.rs:574`

The numbering-level container is selected with the namespace-aware resolver,
but its contents are handed to the ordinary `CT_PPr` and `CT_RPr` parsers.
Those parsers, `property_has_producer_xml`, and `property_overlay` all classify
children with `matches_local_name`. A foreign `<ext:ind ext:left="720"/>` or
`<ext:b/>` whose namespace is inherited from the numbering root is therefore
parsed as a typed Word property. `property_has_producer_xml` also calls it
modeled and can omit the sidecar, so an unchanged round trip replaces the
foreign child with generated `w:ind` or `w:b`. If a sidecar exists for another
producer item, a typed mutation makes the overlay discard the colliding
foreign child as if it were the modeled one. The current regression uses
`ext:pPrData` and `ext:rPrData`, whose local names do not collide with the
modeled lists. This contradicts the producer-extension preservation contract
at `docs/hld/04-opc-and-packaging.md:107`.

### D3, property extras are attached to a mutable child count instead of schema slots

`crates/rdocx-oxml/src/numbering.rs:548`
`crates/rdocx-oxml/src/numbering.rs:550`
`crates/rdocx-oxml/src/numbering.rs:563`
`crates/rdocx-oxml/src/numbering.rs:612`
`crates/rdocx-oxml/src/numbering.rs:616`

The overlay records each producer subtree by the number of modeled children
that preceded it in the source, then applies that number to the newly generated
child vector. Adding or removing an earlier typed property changes the meaning
of that count. For example, clearing `pStyle` in
`<w:pStyle/><ext:data/><w:keepNext/>` generates only `w:keepNext`, then emits
`ext:data` after it rather than before it. Adding an earlier property causes
the inverse movement. The subtree bytes survive, but their relative schema
boundary does not. The regression changes values of existing `ind` and `b`
children, so its modeled child counts remain constant and mask this case. This
violates the requirement that list mutations preserve producer extensions at
`docs/hld/04-opc-and-packaging.md:110`.

### D4, the compatibility test uses the new public fields and does not preserve old struct literals

`crates/rdocx-oxml/src/lib.rs:27`
`crates/rdocx-oxml/src/numbering.rs:667`
`crates/rdocx-oxml/src/numbering.rs:684`
`crates/rdocx-oxml/src/numbering.rs:912`
`crates/rdocx-oxml/src/numbering.rs:1042`
`crates/rdocx-oxml/src/numbering.rs:1143`
`crates/rdocx-oxml/src/numbering.rs:2049`
`crates/rdocx-oxml/src/numbering.rs:2064`

At the sprint base, the re-exported `CT_Lvl` had seven public fields. The
feature adds required `extra_xml` and `extra_attributes` fields, so every
existing external struct literal that names the original seven fields now
fails with missing fields. The same source break exists for `CT_AbstractNum`,
`CT_Num`, and `CT_Numbering`. The new compatibility test is not a test of the
old source surface because its literal explicitly supplies both new fields.
Moving the private property snapshots into the newly added `extra_xml` field
removes the pass-5 private-field break, but it does not make the released
struct literal source-compatible. This contradicts the compatibility row at
`.claude/plans/F-X007-design.md:77` and the patch release's additive API claim
at `docs/hld/10-bindings-spec.md:206`.

### D5, canonical numbering output now fails the mandatory hash harness

`crates/rdocx-oxml/src/numbering.rs:591`
`crates/rdocx-oxml/src/numbering.rs:598`
`crates/rdocx-oxml/src/numbering.rs:599`
`crates/rdocx-oxml/src/numbering.rs:1247`
`scripts/hash_harness.py:35`
`.claude/plans/F-X007-design.md:105`

The no-sidecar property path serializes a canonical property into an
unindented byte buffer and writes those bytes directly into the indented root
writer. This bypasses the parent writer's indentation state. The generated
samples now place `w:pPr` directly after `w:lvlJc` instead of on the established
indented line. `python3 scripts/hash_harness.py --check` reports changed
`word/numbering.xml` hashes for `contract`, `feature_showcase`, `letter`,
`proposal`, `quote`, and `report`. The plan states that the harness is expected
to be unchanged, and an unexplained delta blocks completion even when the XML
change is only lexical.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 22 focused tests passed.
- `cargo test -p rdocx-oxml`: all 120 unit tests and one README doctest passed.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and the shell and dependency contracts passed.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: each inventory contains exactly one intended README.
- `cargo fmt --all --check`, `git diff --check`, and `python3
  scripts/prose_check.py`: passed.
- `python3 scripts/hash_harness.py --check`: failed with the six numbering-part
  deltas stated in D5. The other 22 harness entries were unchanged.
- The local feature history retains Jon Stokes as author of `e2bff75`,
  `7ac5874`, and `f4d09c7`. The recorded GitHub merge commit and public note in
  `.claude/scratch/F-X007-progress.md:7` credit `@jonstokes` and explain the
  value of the fix.

## Not found

The pass-5 remediation removes stale aliases after local prefix shadowing and
uses namespace-aware boundaries for root, abstract, level, and instance raw
children. Canonical root `w` and `r` bindings and the tested root-only foreign
collisions retain their intended expanded names. Unchanged extended `pPr` and
`rPr` containers reuse their raw bytes, while the tested same-child value edits
retain the producer attributes and noncolliding subtrees. No additional defect
was found in scalar expanded forms, fixed abstract and root insertion slots,
list and paragraph bounds, ID allocation, table geometry, hyperlink or break
behavior, README content, package inventory, HLD file scope, resource bounds,
or contributor credit. No structural-rule violation, smell, or nitpick was
found.
