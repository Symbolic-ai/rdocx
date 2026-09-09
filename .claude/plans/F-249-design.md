# F-249, Deterministic package identifier allocation

**Status**: completed
**Sprint**: S71
**Size**: M
**Depends on**: F-243

## Problem

Identifier allocation is distributed among relationships, media naming,
comments, bookmarks, drawings, numbering, TOC targets, and individual part
helpers. For example, relationships scan only their own list
(`crates/oxml-opc/src/relationship.rs:265`), settings have a separate part-name
scan (`crates/rdocx/src/document.rs:1489`), and numbering owns separate next-id
functions (`crates/rdocx-oxml/src/numbering.rs:3242`). These local allocators do
not share a preflight that detects collisions across imported, preserved, and
pending document state.

Capability row `DOCX-014` remains partial
(`docs/hld/02-scope-and-non-goals.md:221`). Equivalent document construction
must allocate stable ids in document order and reject a collision before any
mutation becomes visible.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability row `DOCX-014`.
- `docs/hld/03-architecture.md`, facade ownership and transaction boundaries.
- `docs/hld/04-opc-and-packaging.md`, "The package", "Part naming", and
  "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, determinism and private authoring gates.
- `docs/hld/13-risks-and-open-questions.md`, R6 and R13.
- `docs/hld/14-development-backlog.md`, "F-249, Deterministic package
  identifier allocation".

## Approach

Add one private `DocumentIdentifiers` value inside the existing document module.
It scans the package, typed document tree, preserved XML owners, relationships,
part names, and content types into category-specific occupied sets. Categories
cover relationship ids per owner, bookmark and comment ids, drawing ids,
abstract-numbering and numbering-instance ids, part suffixes, and content-type
entries.

Every facade mutation reserves its complete identifier bundle from a staged
clone in final semantic document order, independent of caller request and map
iteration order. It validates imported and preserved collisions, then
publishes the candidate once. Keep category namespaces separate where OOXML
defines separate scopes. Return stable contextual errors on duplicate existing
identity, overflow, or a pending collision.

Add parsed `docPr` ids to the pre-1.0 `CT_Inline` and `CT_Anchor` models so new
drawings stop hard-coding one identifier. Replace local facade allocation paths with this owner while retaining
`oxml-opc`'s general package primitives. Rescan after open and clone allocation
state during a transaction. Derive final allocation from semantic document
order so equivalent construction orders declared by the conformance fixture
produce the same ids and package bytes.

## Rejected alternatives

- Use UUIDs or random ids. They violate reproducible output.
- Share one integer sequence across unrelated OOXML scopes. That changes valid
  existing identifiers without improving collision safety.
- Repair imported duplicates silently. A preserved collision must fail before
  mutation so no producer content is rewritten unexpectedly.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `equivalent_construction_orders_allocate_declared_stable_identifiers` | Semantically equivalent public operations produce the same relationship, bookmark, comment, drawing, numbering, part, and content-type identities. |
| regression | `repeated_saves_are_byte_identical_after_allocation` | Allocation is not advanced by serialization or reopen. |
| regression | `imported_and_preserved_collisions_fail_before_mutation` | A duplicate in typed or raw retained state returns a contextual error and leaves package bytes unchanged. |
| integration | `identifier_scopes_do_not_alias_or_overreach` | Per-owner relationship ids and separate OOXML id categories remain independent while each is collision-free. |
| regression | `internal_reopen_mutations_preserve_minimal_bundle_history` | Every internal staged reopen, including building-block replacement, preserves authored bundle provenance and produces order-equivalent bytes. |
| unit | `cross_type_header_footer_references_never_mutate_the_target_story` | Header and footer replacement accepts only a strict internal relationship with the type implied by the section reference. |
| unit | `content_type_identities_are_ascii_case_insensitive` | Default extensions and override part names reject case-variant duplicates and resolve sole variants without changing authored spelling. |

The **test gate is regression**. Equivalent construction orders produce the
declared stable identifiers and repeated saves are byte-identical.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serializer**. Prove prefix-tolerant collision scans, fixed
  write prefixes, schema order, and byte-exact preservation of raw owners.
- **Public API of a published crate**. Although the allocator stays private,
  existing fallible facade methods gain deterministic collision errors. Run the
  verified workspace package dry-run and archive-size assertions.

## Hash harness

Two existing sample document parts change because they contain colliding
authored drawing identifiers and relationships authored out of final story
order. `feature_showcase:word/document.xml` changes from
`3afc92178fe9e0e932d2685988ed6cc541124d3d8c806504b429b6db78990762` to
`7c38b482fb5611c39edaf3a7931e62a49fef81861e76e05d5039830db7cc7053`.
Its second `wp:docPr` id changes from 1 to 2, image references change from
`rId6` and `rId7` to `rId2` and `rId3`, and default-before-first section
references receive `rId4` through `rId7`. `report:word/document.xml` changes
from `5a834b2ebe01156c082f25ea05483c135d3718beae7d80b2224e0a02f9b93365` to
`a879be8fcb630c39824270e40b0a34dba56fb5c04010f27720da4b96df76f12a`.
Its three `wp:docPr` ids become 1 through 3, image references become `rId1`
through `rId3`, and default-before-first header and footer references become
`rId4` through `rId6`. Every other harness entry remains unchanged. The
baseline update belongs to the labeled F-249 behavior commit.

## Implementation checklist

- [x] Inventory every story-owned identifier category and scope.
- [x] Add one private allocation owner in the existing document module.
- [x] Scan typed, package, and preserved XML state for occupied identities.
- [x] Reserve complete mutation bundles on staged candidates.
- [x] Replace distributed facade allocation paths without changing general OPC
  primitives.
- [x] Add construction-order, repeated-save, collision, overflow, and scope
  regressions.
- [x] Run the full gate and routed package dry-run.
- [x] Update exactly the listed HLD files.

## Verification evidence

The exact workspace publish dry run must pass on the committed worker SHA. Its
pre-commit run reached crates.io and stopped at Cargo's dirty-tree check, so the
gate remains open. The same command with `--allow-dirty` passed for every
publishable workspace crate as supplemental evidence. The archive-size assertion
also passed, with `oxml-layout-0.11.0.crate` largest at 4,603,492 bytes. The exact
command without `--allow-dirty` must be rerun before this checklist item is
checked.

Twelfth-pass remediation retains the header or footer kind through replacement
enumeration and validates the expected relationship type before load or save.
Building-block replacement now uses the canonical staged preparation and
provenance-reconciling reopen path. Content-type identity is ASCII
case-insensitive for parsing and API operations while the first authored key
spelling remains stable. Focused, scoped, OPC all-features, and routed workspace
test suites pass. The hash harness still reports only the two approved document
XML deltas, and the checked-in baseline remains untouched.

Thirteenth-pass remediation applies the exact relationship-type gate before any
header or footer setter reuses a referenced part. ZIP entries, document part
reservations, and relationship-target occupancy now use ASCII-case-insensitive
part identity while retaining package spelling. Content-type parsing uses
normalized sets, unchanged producer XML is retained under every feature set,
and serialization rejects case-conflicting direct public-map mutations before
writing output. Scoped suites pass with only the documented host oracles
filtered. The hash harness still reports only the two approved document XML
deltas, and the checked-in baseline remains untouched.
Current package relationships absent from the initial producer-preserved set
also join authored canonicalization. This restores the SHA-bound F-159 chart
candidate by ordering its authored chart before a directly added theme edge,
while a theme captured on package open keeps its producer id. Unknown internal
and external authored edges retain their targets and modes.

Fourteenth-pass remediation treats every office relationship namespace
attribute in retained body XML and raw drawing payloads as a fixed relationship
occupant. Package part and relationship-owner APIs, Word and PowerPoint required
part resolution, and digital-signature discovery and coverage now share
normalized ASCII-case-insensitive identity while preserving stored spelling.
Duplicate relationship ids and direct public-map case conflicts fail before any
package output starts. Path saves serialize and validate all fallible graph
state before opening the destination, so validation cannot truncate an existing
file.
Focused package, Word, PowerPoint, regression, and integration suites pass.
Workspace clippy, formatting, diff, prose, and generated-skill gates pass. The
hash harness still reports only the two approved document XML deltas, and the
checked-in baseline remains untouched.

Fifteenth-pass remediation derives fixed opaque relationship occupants from the
complete serialized current Word story before semantic remapping. This covers
retained paragraph and run children whose relationship prefix is bound on a
local ancestor, including character-reference encoded ids, while leaving their
bytes and edges unchanged. Relationship and content-type parsers decode XML
attribute values before duplicate validation and storage, so encoded aliases
collide and ordinary targets serialize with one escaping pass.

Document relationship-owner registries use normalized ASCII-case identities
through scan, reserve, canonicalization, pruning, and provenance reconciliation.
PowerPoint media mutation writes preserve the stored slide-owner spelling, core
collision checks and media reachability use case-insensitive part identity, and
Flat OPC import applies that identity to duplicate parts, relationship owners,
special relationship-part names, and owner existence. Flat OPC also validates
the completed package graph before publication.

Focused regressions and the full OPC suite pass. The `rdocx` library suite
passes 406 tests with 4 ignored apart from the unavailable external chart pixel
rasterizer. The `rpptx` library suite passes 81 tests with 4 ignored. The
PowerPoint integration suite passes 200 tests with 13 ignored apart from four
installed SmartArt resource SHA drift failures. Workspace clippy, formatting,
diff, prose, and generated-skill gates pass. The hash harness still reports only
the two approved document XML deltas, and the baseline remains untouched at
`7ab4133f8caf5b0f9593578c4c7ed908079c3ba530a717510d8b062d15abbe46`.

Final completion evidence is bound to reviewed worker SHA
`f1dd88472ee76be577a9d3be60a280b8fed589f0`. The exact `/verify --full` gate
passed in the pinned environment, including the canonical 50-deck corpus. The
exact 22-patch workspace publish dry run passed without `--allow-dirty`, and all
22 archives passed the size ceiling. The largest archive was
`oxml-layout-0.11.0.crate` at 4,603,515 bytes. The hash harness matched all 49
reviewed entries. Regression gates
`equivalent_construction_orders_allocate_declared_stable_identifiers` and
`repeated_saves_are_byte_identical_after_allocation` passed.

## Open questions

None. The user approved equivalence by final document order and the pre-1.0
`docPr` id additions to `CT_Inline` and `CT_Anchor`.
