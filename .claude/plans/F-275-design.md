# F-275, Cross-story bookmarks, ranges, and annotations

**Status**: completed
**Sprint**: S89
**Size**: L
**Depends on**: F-253, F-254

## Problem

`Document::add_bookmark` in `crates/rdocx/src/comments.rs:459` addresses body
paragraphs only. The existing story-qualified comment path at line 561 and
`StoryRunRange` at line 64 show the owner coordinate to reuse, while the
paragraph anchor in `crates/rdocx-oxml/src/text.rs:4042` recognizes only
comment and bookmark markers. Permission and proofing endpoints remain raw.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", row DOCX-042.
- `docs/hld/03-architecture.md`, "Facade conventions", container-neutral story editing and bookmark mutation.
- `docs/hld/04-opc-and-packaging.md`, "The package", story ownership and preserved children.
- `docs/hld/12-testing-strategy.md`, "The Word corpus", story and bookmark range gates.
- `docs/hld/14-development-backlog.md`, "F-275, Cross-story bookmarks, ranges, and annotations".

## Approach

Extend the existing concrete `StoryRunPosition` and `StoryRunRange` path to
bookmarks, comment endpoints, permission ranges and proofing ranges. Correlate
paired markers by expanded name and family-specific identity. A permission
start retains its editor or group metadata. A proofing range retains its error
type and has no fabricated numeric ID. Expose story-qualified immutable range
snapshots and checked add, move and remove operations through `Document`.

Resolve both endpoints against one physical story owner, including nested
table cells, block controls and text boxes. Validate in accepted-view order,
including same-boundary marker order, before staging changes. Reject unmatched,
reversed and crossing-invalid pairs, duplicate names and exhausted IDs without
publishing a partial edit. Preserve unsupported paired XML and all unrelated
raw siblings. Do not add permission user policy, move revision authoring or
modern comment metadata, which have later owners.

## Rejected alternatives

- Reusing body indices for related stories would address the wrong paragraph after a story mutation.
- One ID-only marker enum cannot represent proofing pairs or permission metadata accurately.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `cross_story_ranges_reopen_with_exact_endpoints` | **Test gate.** Supported range families retain endpoints across every valid story and nested container. Nested ranges survive, crossing-invalid ranges reject atomically. |
| regression | `story_marker_allocation_is_family_aware` | Existing producer IDs, aliases and metadata survive checked additions and removals. |
| round-trip | `paired_marker_mutation_preserves_unknown_siblings` | Changed XML is schema ordered and untouched subtrees remain byte-identical. |

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

Expected unchanged. Marker-only edits must not alter existing rendered fixtures.

## Implementation checklist

- [x] Extend marker parsing and anchoring in the existing paragraph model.
- [x] Add checked story-qualified range inventory and mutation to the facade.
- [x] Validate same-owner pairing and crossing before any candidate publication.
- [x] Cover aliases, nested owners, unsupported raw XML, round-trip and atomic negatives.
- [x] Run scoped verification and obtain a zero-finding microscope review.

## Open questions

None. The supported set is bookmarks, comment ranges, permission ranges and proofing ranges. Revision move ranges, permission policy and modern comment metadata remain with F-292 through F-294.
