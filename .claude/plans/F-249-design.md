# F-249, Deterministic package identifier allocation

**Status**: approved
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

Expected unchanged for existing harness fixtures. Existing collision-free
document-order allocations retain their current ids. Any deliberate changed id
would require an isolated expected-delta commit before integration.

## Implementation checklist

- [ ] Inventory every story-owned identifier category and scope.
- [ ] Add one private allocation owner in the existing document module.
- [ ] Scan typed, package, and preserved XML state for occupied identities.
- [ ] Reserve complete mutation bundles on staged candidates.
- [ ] Replace distributed facade allocation paths without changing general OPC
  primitives.
- [ ] Add construction-order, repeated-save, collision, overflow, and scope
  regressions.
- [ ] Run the full gate and routed package dry-run.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved equivalence by final document order and the pre-1.0
`docPr` id additions to `CT_Inline` and `CT_Anchor`.
