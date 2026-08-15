# F-X007, all aspects, pass 7

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all six earlier remediation
rounds. The reviewed state is 39 files and 3,882 changed or new line entries:
27 tracked files with 2,723 additions and 258 deletions, plus 901 lines in
twelve untracked files.
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, property projection still equates Word namespace membership with modeled content

`crates/rdocx-oxml/src/numbering.rs:425`
`crates/rdocx-oxml/src/numbering.rs:446`
`crates/rdocx-oxml/src/numbering.rs:482`
`crates/rdocx-oxml/src/numbering.rs:507`
`crates/rdocx-oxml/src/numbering.rs:579`
`crates/rdocx-oxml/src/properties.rs:166`
`crates/rdocx-oxml/README.md:3`

The projection correctly excludes a foreign-prefixed collision, but it treats
every Word-qualified attribute on a modeled child as typed and every
Word-qualified descendant below depth zero as modeled. The ordinary property
parsers do not model all of that content. For example,
`<w:ind w:left="720" w:producer="keep"/>` is projected with both attributes,
then `CT_PPr` reads `left` and silently ignores `producer`. Because projection
never marks the ignored Word attribute as producer XML, `ppr_raw` remains
empty and the attribute is lost even without a mutation. The same failure
applies to an unknown Word child nested inside a recognized `numPr`, `pBdr`,
`tabs`, nested `rPr`, or other composite child. The regression covers foreign
`ext:ind` and `ext:b`, not unmodelled content in the Word namespace. This
contradicts the crate README's verbatim preservation claim and the modeled
container compatibility contract at `docs/hld/04-opc-and-packaging.md:107`.

### D2, unprefixed and default-foreign property names are classified as WordprocessingML

`crates/rdocx-oxml/src/numbering.rs:310`
`crates/rdocx-oxml/src/numbering.rs:315`
`crates/rdocx-oxml/src/numbering.rs:416`
`crates/rdocx-oxml/src/numbering.rs:418`
`crates/rdocx-oxml/src/numbering.rs:482`
`crates/rdocx-oxml/src/numbering.rs:694`

The namespace scope helper tracks only `xmlns:prefix` declarations, while
`is_word_name` accepts every unprefixed name without consulting a default
namespace. A valid foreign element such as
`<ind xmlns="urn:producer" left="999"/>` inside a Word `pPr` is therefore
projected and overlaid as `w:ind`. On typed mutation the raw foreign element is
discarded and replaced by the generated Word child. Unprefixed attributes are
also always no-namespace in XML, but the same helper treats them as Word
attributes. The prefixed `ext:` fixture at
`crates/rdocx-oxml/src/numbering.rs:2300` cannot detect either case. This is an
expanded-name identity failure under the contract at
`.claude/plans/F-X007-design.md:46`.

### D3, the property slot tables are serializer order rather than OOXML schema order

`crates/rdocx-oxml/src/numbering.rs:16`
`crates/rdocx-oxml/src/numbering.rs:23`
`crates/rdocx-oxml/src/numbering.rs:26`
`crates/rdocx-oxml/src/numbering.rs:35`
`crates/rdocx-oxml/src/numbering.rs:44`
`crates/rdocx-oxml/src/numbering.rs:53`
`crates/rdocx-oxml/src/numbering.rs:614`
`crates/rdocx-oxml/src/properties.rs:269`
`crates/rdocx-oxml/src/properties.rs:290`
`crates/rdocx-oxml/src/properties.rs:717`
`crates/rdocx-oxml/src/properties.rs:720`

The fixed positions mirror the current typed writers, not the
WordprocessingML sequences. In `CT_PPr`, `numPr` precedes `pBdr` and
`suppressAutoHyphens` follows `tabs`, but the table and writer put
`suppressAutoHyphens` before the borders and `numPr` after the tabs. In
`CT_RPr`, `strike` and `dstrike` precede `vanish`, but the table and writer put
`vanish` first. The highlight and underline positions are also reversed from
the schema. A typed edit to a valid source such as `numPr`, producer subtree,
then `pBdr` replays both modeled children in the wrong order and moves the
subtree to the wrong effective boundary. The new slot regression uses only
`pStyle`, `keepNext`, and `ind`, whose table order happens to match the schema.
This violates the mandatory `xsd:sequence` rule and the schema-order rider at
`.claude/plans/F-X007-design.md:93`.

### D4, the property projector recurses without a depth bound on input XML

`crates/rdocx-oxml/src/numbering.rs:460`
`crates/rdocx-oxml/src/numbering.rs:466`
`crates/rdocx-oxml/src/numbering.rs:482`
`crates/rdocx-oxml/src/numbering.rs:489`
`crates/rdocx-oxml/src/numbering.rs:495`

`project_word_element` recursively calls itself for every Word-qualified child
below the first modeled property child. Neither the XML parser nor this helper
limits depth. A deeply nested property subtree can therefore exhaust the Rust
stack while parsing a document, instead of being preserved or rejected with a
normal error. The `depth` value controls classification but does not bound the
recursion. This is a new non-total path on untrusted package XML and is not
covered by the shallow collision fixtures.

### D5, the promised 0.5.0 numbering-model migration is not documented

`crates/rdocx-oxml/README.md:3`
`crates/rdocx-oxml/README.md:19`
`crates/rdocx-oxml/src/numbering.rs:829`
`crates/rdocx-oxml/src/numbering.rs:833`
`docs/hld/10-bindings-spec.md:190`
`.claude/plans/F-X007-design.md:78`
`.claude/plans/F-X007-design.md:95`

The approved plan requires a migration to constructors or the new public
fields. The package README still shows only ordinary low-level use and a 0.4
dependency, while HLD10 describes the additive facade APIs without recording
the breaking numbering-model boundary. The field comments identify the tuple
contents but do not tell an existing struct-literal user to add empty
`extra_xml` and `extra_attributes`, set `ppr_raw` and `rpr_raw` to `None`, or
prefer the available constructors. The implementation test proves the new
literal and canonical equality, but no published documentation satisfies the
migration row of the plan.

### D6, authoritative sprint and backlog records still require a 0.4.2 patch release

`docs/hld/14-development-backlog.md:1199`
`docs/hld/14-development-backlog.md:1200`
`docs/hld/14-development-backlog.md:1205`
`docs/sprints/CURRENT_SPRINT.md:8`
`docs/sprints/CURRENT_SPRINT.md:50`
`docs/sprints/CURRENT_SPRINT.md:52`
`docs/sprints/SPRINT_PLAN.md:579`
`docs/sprints/SPRINT_PLAN.md:582`
`.claude/plans/F-X007-design.md:52`
`.claude/plans/F-X007-design.md:88`

The revised plan and owner approval select 0.5.0, but HLD14 still defines
F-X008 as preparation, release, and registry verification at 0.4.2. The
current sprint definition of done repeats that exact tag and version, while
the long-range sprint plan still calls the release a fresh patch. HLD14 is
explicitly listed in this feature's HLD impact and has no working diff. Leaving
these records unchanged would send the next workflow into the release boundary
that the revised design rejected. This is spec and delivery-state drift, not
work that can be deferred to version preparation.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml numbering::tests`: all 26 focused tests passed.
- `cargo test -p rdocx-oxml`: all 124 unit tests and one README doctest passed.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and the shell and dependency contracts passed.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: each inventory contains exactly one intended README.
- `cargo fmt --all --check`, `git diff --check`, and `python3
  scripts/prose_check.py`: passed.
- The local feature history retains Jon Stokes as author of all three
  contributor commits. The GitHub merge commit and public note recorded at
  `.claude/scratch/F-X007-progress.md:7` retain the requested credit.

## Not found

The descendant declaration scan prevents locally preserved `w` and `r`
bindings from shadowing the generated model prefixes in the covered prefixed
cases. Foreign prefixed direct property collisions remain raw, and the tested
earlier-child additions and removals retain a producer element before the same
successor. Explicit public `ppr_raw` and `rpr_raw` fields make the approved
breaking state visible, while canonical containers keep those fields empty and
retain canonical equality. Generated property events now pass through the
parent writer and restore exact sample bytes. No additional defect was found
in scalar forms, abstract and root boundaries, list and paragraph bounds, ID
allocation, table geometry, hyperlinks, hard breaks, README compilation,
package inventory, hash stability, or contributor credit. No smell or nitpick
was found.
