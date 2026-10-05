# F-277, Glossary and building-block creation

**Status**: completed
**Sprint**: S89
**Size**: L
**Depends on**: F-237, F-253, F-276

## Problem

`crates/rdocx/src/building_block.rs:330` can inventory producer entries and
replace one. It cannot create, classify, insert or remove entries through the
public facade. `CT_GlossaryDocument::to_xml` in
`crates/rdocx-oxml/src/glossary.rs:460` edits only spans of existing parts, so
the current model cannot structurally add or remove `w:docPart` children while
preserving unsupported siblings.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", row DOCX-044.
- `docs/hld/03-architecture.md`, "What stays put", glossary root ownership and "Facade conventions", staged fragment insertion.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", glossary relationship validation.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", public-created glossary round-trip gate.
- `docs/hld/14-development-backlog.md`, "F-277, Glossary and building-block creation".

## Approach

Extend the existing glossary OXML model with structural `w:docPart` insertion,
update and removal in schema order. Retain raw spans for untouched entries and
unsupported siblings. Expose checked facade operations to create, classify,
update, insert and remove entries with name, gallery, category, behavior and
owned content. A placeholder entry has the glossary `placeholder` gallery and
can be bound to an existing content control through its `w:placeholder` and
document-part selection properties. This story does not create the content
control itself. Use existing `BuildingBlockInfo` part and ordinal identity
validation, and one staged package transaction for each mutation.

Insertion uses F-276's fragment transaction to bring content and its reachable
relationships into the destination story. Create the single safe internal
glossary relationship, part and content-type override on first use. Retain a
valid empty glossary part after the last entry is removed. Preserve unrelated
package parts and producer prefixes. Reject unsafe,
duplicate, stale or incomplete graphs before publishing.
Teach the glossary reader and writer to accept and produce one `w:docParts`
container with zero entries for this retained empty-part state.

## Rejected alternatives

- Rebuilding the entire glossary root from typed entries would discard producer siblings and prefixes.
- Copying only entry XML during insertion would strand images, notes and other related content.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `public_created_building_blocks_insert_and_reopen` | **Test gate.** New AutoText, building block and placeholder entries retain category, behavior, content, relationships, content-control placeholder binding and unsupported siblings after insertion and reopen. |
| regression | `glossary_lifecycle_preserves_untouched_docparts` | Add, update and remove preserve untouched entry and root bytes in schema order. |
| round-trip | `last_entry_removal_keeps_a_reopenable_empty_glossary` | The one-entry to zero-entry transition keeps its safe relationship and content type and reopens as an empty inventory. |
| regression | `invalid_glossary_graph_rolls_back_atomically` | Stale identity, unsafe relationship, bad content or failed fragment import leaves the package unchanged. |

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and `06-presentationml-model.md`. Check schema order, prefix tolerance, fixed-prefix changed output and byte-exact unknown subtree retention.
- **Public API of a published crate**. Read `docs/hld/10-bindings-spec.md`. State additive pre-1.0 semver impact, run `cargo publish --dry-run` and assert the `.crate` size.

## Hash harness

Expected unchanged. Existing producer glossary documents retain their current bytes unless deliberately edited.

## Implementation checklist

- [x] Consume F-276's completed dependency import transaction.
- [x] Add structural entry edits in the existing glossary OXML file.
- [x] Add checked facade creation, classification, update, insertion and removal.
- [x] Wire first-use package relationship, part and content-type ownership.
- [x] Prove public-created round trips, relationship closure, raw preservation and atomic failures.
- [x] Run scoped verification and obtain a zero-finding microscope review.

## Open questions

- Answered: first creation adds the glossary part, and last removal retains a valid empty part.
- Answered: placeholder covers both glossary gallery classification and binding an existing content control. F-285 owns content-control creation and lifecycle.
