# F-240, Modern DOCX completeness audit and private corpus matrix

**Status**: completed
**Sprint**: S70
**Size**: L
**Depends on**: none

## Problem

The public `rdocx` facade exposes a broad Word package, authoring, conversion,
and rendering surface, but there is no property-level contract that separates
complete authoring from partial, preservation-only, unsupported, and permanent
non-goal behavior. The facade exports many concrete authoring types while
`Document` retains substantial package state privately
(`crates/rdocx/src/lib.rs:24` and `crates/rdocx/src/document.rs:1382`). The
narrower WASM surface further demonstrates that native and binding capability
cannot be inferred from one API list (`crates/rdocx-wasm/src/lib.rs:20`).

The provisional M23 and M24 roadmap contains 71 Word stories and explicitly
allows this audit to split, merge, resize, or archive them
(`docs/hld/14-development-backlog.md:2201`). Without a closed matrix, duplicate
scope, dangling dependencies, private-corpus leaks, and unsupported claims can
survive into implementation. The five supplied private DOCX references are now
available only under ignored anonymous paths, as required by the private corpus
contract (`docs/hld/12-testing-strategy.md:1299`).

## Spec reference

- `docs/hld/00-vision.md`, the M23 and M24 post-v1 authoring boundaries.
- `docs/hld/02-scope-and-non-goals.md`, "Beyond v1" and "Still non-goals, and
  still permanent".
- `docs/hld/03-architecture.md`, native facade ownership and preservation of
  unmodeled state.
- `docs/hld/04-opc-and-packaging.md`, package ownership, relationships, content
  types, and loss-free retention.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and the Python,
  WASM, and CLI surface boundaries.
- `docs/hld/12-testing-strategy.md`, "The private from-scratch DOCX conformance
  corpus".
- `docs/hld/13-risks-and-open-questions.md`, R11 through R13.
- `docs/hld/14-development-backlog.md`, "F-240, Modern DOCX completeness audit
  and private corpus matrix".

## Approach

Add the canonical capability matrix to the existing scope HLD. Use one stable
row per capability or property with a stable id, family, create, read, mutate,
remove, save-reopen, story placement, layout, render, determinism, native,
Python, WASM, CLI, classification, evidence, and owner fields. Restrict
classification to `complete`, `partial`, `preserve-only`, `unsupported`, and
`permanent-non-goal`. Every incomplete in-scope row names exactly one live F-ID.

Audit public facade exports, modeled OXML properties, package ownership,
existing tests, HLD claims, and bindings. Treat `Document::new()` as the M23
construction boundary. Public OXML leakage is not public-facade authoring.
Preservation of parsed raw XML is distinct from modeled mutation.

Inspect the five ignored references as anonymous P1 through P5. Record only
non-identifying requirements in tracked prose. Keep source filenames, hashes,
text, XML, media, renders, and detailed differentials in the ignored corpus
area. The initial package inventory requires the audit to cover at least custom
XML and bindings, custom properties, web extensions, task panes, multi-variant
headers and footers, scoped relationships and media, numbering, notes, modern
comment metadata, themes, fonts, settings, and web settings.

Reconcile F-243 through F-310 against the completed matrix. Split, merge,
resize, archive, reorder, or correct dependencies where evidence requires it,
while preserving the M23 five-document outcome and M24 modern-authoring outcome.
Update the existing HLD and shared sprint records, not a second backlog or
tracker. Add repository-document regressions to
`scripts/test_sprint_workflow.py` for classifications, ownership, unique
placement, dependency expansion, cycles, status alignment, and private-summary
redaction.

Correct `CURRENT_SPRINT.md` to cite `docs/hld/08-rendering-spec.md`, the actual
file, instead of the nonexistent rendering-pipeline path.

## Rejected alternatives

- Create a second capability backlog. The repository permits exactly one
  backlog and sprint plan.
- Put the matrix in a new tracked file. The existing scope HLD is the canonical
  classification owner and avoids an unapproved file.
- Treat every modeled OXML type as facade-complete. Many invariants span parts,
  relationships, stories, or bindings.
- Track private source identities or extracted facts. That would violate the
  corpus confidentiality boundary.
- Keep provisional stories unchanged after contradictory evidence. F-240 exists
  to make the roadmap evidence-based.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_modern_docx_capability_matrix_has_closed_classifications_and_evidence` | Every row uses one allowed classification and cites implementation, test, boundary, or non-goal evidence. |
| regression | `test_every_incomplete_modern_docx_row_has_one_live_owner` | Each incomplete in-scope row maps to exactly one live story. |
| regression | `test_m23_m24_roadmap_has_no_duplicate_or_dangling_story` | Expanded dependency ranges contain unique F-IDs, valid endpoints, no cycles, and one sprint placement. |
| regression | `test_private_corpus_summary_contains_no_private_identity` | Tracked changes contain only anonymous P1 through P5 requirements and no private filename, digest, text, XML, or media identity. |
| integration | public facade and modeled-property audit | Every reachable modeled property has a classified matrix row and binding status. |

The **test gate is regression**. Every in-scope matrix row has evidence, an
owner story, an explicit preservation boundary, or a permanent non-goal, and
every roadmap duplicate or dangling dependency is rejected.

## HLD impact

- `docs/hld/00-vision.md`
- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

none. The matrix and its regressions stay in existing documentation and test
files. The audit changes no parser, serializer, public API, binding,
dependency, feature, asset, release carrier, or rendered output.

## Hash harness

Expected unchanged across all 49 entries. This story audits and reshapes the
contract without changing document generation or rendering.

## Implementation checklist

- [x] Inventory the five private references under anonymous local identities.
- [x] Enumerate modeled properties, public authoring, package ownership, layout,
  rendering, determinism, diagnostics, and binding coverage.
- [x] Write the closed capability matrix in the existing scope HLD.
- [x] Map every incomplete in-scope row to exactly one live story.
- [x] Reconcile F-243 through F-310 and all S71 through S80 dependencies.
- [x] Add duplicate, dependency, classification, evidence, and privacy tests.
- [x] Correct the nonexistent current-sprint spec reference.
- [x] Run prose, sync-status, focused regressions, full verification, and hash
  checks.
- [x] Update exactly the listed HLD files and leave shared sprint records to the
  integrator.

## Open questions

None. The private corpus is available under ignored anonymous paths, and the
existing scope HLD provides a canonical home without creating another record.
