# F-276, Complete fragment conflict and dependency policy

**Status**: completed
**Sprint**: S89
**Size**: L
**Depends on**: F-256, F-270, F-271, F-272, F-273, F-274, F-275

## Problem

`DocumentFragment::from_range` in `crates/rdocx/src/document.rs:967` and
`Document::import_fragment` at line 15837 accept only main-body ranges. The
dependency closure in `crates/rdocx/src/field.rs:1901` handles the selected
body, styles, numbering, comments and related parts, but it does not map the
complete note, range, revision, custom XML, diagram, embedding and extension
graph listed in the backlog contract. A fragment can therefore leave a
dependency dangling or collide with destination identities.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", row DOCX-043.
- `docs/hld/03-architecture.md`, "Facade conventions", `DocumentFragment` closure and staged import.
- `docs/hld/04-opc-and-packaging.md`, "Package integrity", reachable dependency remapping.
- `docs/hld/10-bindings-spec.md`, native `DocumentFragment` and conflict policy boundaries.
- `docs/hld/13-risks-and-open-questions.md`, package and fragment import integrity risks.
- `docs/hld/12-testing-strategy.md`, "The Word corpus", cross-document fragment gate.
- `docs/hld/14-development-backlog.md`, "F-276, Complete fragment conflict and dependency policy".

## Approach

Extend the existing `DocumentFragment` source selection and
`Document::import_fragment` destination to every compatible supported story
owner, including body, headers, footers, notes, comments and nested containers.
Compatibility is decided by block items at supported `ContentLocation`
boundaries, the target owner's admitted child kinds and the target part's
relationship scope. Existing two-segment block-control paragraph paths select
their enclosing control content. Inline selections remain invalid because no
inline fragment boundary is exposed.
Test every supported source and destination owner class and reject an
incompatible pairing atomically. Begin the dependency closure at references in
selected owner XML and required identity companions, not unrelated source
parts. Close styles and aliases, numbering and overrides,
custom XML binding stores, notes, comment threads, revision and paired-marker
identities, charts, diagrams, embeddings and package extensions. Include custom
XML item properties and chart or diagram companion parts. Traverse internal
relationships, including cycles, and
copy every well-formed reachable package graph, including opaque extension
parts, rather than excluding a dependency class by default. Preserve opaque
payload bytes and their part-local relationship IDs verbatim by assigning each
copied opaque owner a fresh part name. Rewrite its `.rels` internal targets to
the allocated part names, while retaining external targets, modes and types.
Reject a graph only when its opaque payload itself would need an unsafe rewrite
or an integrity-bound part cannot remain valid. Remap selected-owner references
in a namespace-aware pass where collisions require it. Carry external
relationships as external edges without fetching them. Validate copied typed
companions and every internal edge before package reopen. Apply the existing
signature invalidation policy to changed coverage and never retain an invalid
signature as valid. An unknown payload with an internal name or reference that
cannot be proven intact after remapping fails with a named reason before
publication, leaving the destination unchanged.

Retain `FragmentConflictPolicy` presets for equivalent reuse and rename-all.
Apply equivalent reuse only to dependency classes with a proven structural
comparison: styles, numbering and leaf parts. Allocate fresh destination
identities for notes, comments, revisions and paired markers in deterministic
source encounter order. Reserve all part names, relationship IDs and internal
IDs before rewriting references in a single staged candidate. Serialize and
reopen the candidate before publishing it. This transaction is the insertion
path F-277 will reuse.

## Rejected alternatives

- Importing referenced parts as encountered risks order-dependent IDs and partial results.
- Reusing identity-bearing annotations by apparent text equality could attach a fragment to the wrong producer record.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `full_story_fragment_import_remaps_every_conflicting_dependency` | **Test gate.** Body, header, footer, footnote, endnote, comment, cell, control and text-box owners exercise every grammar-valid source and destination pairing, with invalid pairings rejected atomically. Two imports remap style alias cycles, numbering overrides, custom XML binding stores and item properties, note references, comment threads, paired endpoints, revisions, charts, diagrams and embeddings deterministically without dangling IDs. |
| regression | `opaque_extension_graphs_copy_without_corrupting_parts` | Direct, nested, cyclic and collision-heavy opaque graphs copy with exact payloads, rewritten internal `.rels` targets, retained external target, mode and type, and valid signature handling. Root and nested external edges are never fetched. |
| round-trip | `fragment_import_preserves_unmodeled_xml_and_reopens` | Selected retained XML and unrelated destination XML remain exact, with schema-valid changed content. |
| regression | `fragment_import_rejects_incomplete_graph_atomically` | Invalid section-property placement, split ownership, dangling or tampered internal targets, duplicate or case-equivalent part names, malformed, unsafe, unreconcilable or exhausted graphs leave destination bytes unchanged. |
| regression | `fragment_conflict_policies_reuse_only_equivalent_graphs` | Equivalent-reuse and rename-all yield the declared deterministic style, numbering and leaf-part differences. |

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and `06-presentationml-model.md`. Check schema order, prefix tolerance, fixed-prefix changed output and byte-exact unknown subtree retention.
- **Public API of a published crate**. Read `docs/hld/10-bindings-spec.md`. State additive pre-1.0 semver impact, run `cargo publish --dry-run` and assert the `.crate` size.

## Hash harness

Expected unchanged. Existing fixture imports keep their current serialized output.

## Implementation checklist

- [x] Consume completed F-274 and F-275 note and range contracts.
- [x] Generalize selection and destination owner validation in existing facade files.
- [x] Close the full reachable dependency graph and preallocate all collision maps.
- [x] Rewrite references together, stage, serialize, reopen and publish once.
- [x] Prove deterministic, lossless, conflict and atomic-failure fixtures.
- [x] Run scoped verification and obtain a zero-finding microscope review.

## Open questions

- Answered: any grammar-compatible supported source and destination story pair is valid. Ownership and relationship scope are checked for every pair.
- Answered and revised: copy every well-formed reachable graph, including opaque extensions and external relationship edges, so far as it can be remapped without corrupting any part. Reject only graphs that cannot satisfy that integrity boundary.

- Clarified during implementation: the native fragment API exposes block boundaries, including existing nested block-control paragraph paths. The HLD impact includes the native binding contract that previously said main-body only.
