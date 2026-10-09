# F-X187, Scoped paragraph and cell text replacement

**Status**: completed
**Sprint**: S90
**Size**: M
**Depends on**: none

## Problem

The global replacement at `crates/rdocx/src/document.rs:25484` searches every owner. Issue 285's identical clause appears twice, so an expected count of one fails. Python Paragraph and Cell have no scoped run-aware replacement. Whole-text setters at `paragraph.rs:868` and `table.rs:1635` are not the existing cross-run placeholder operation and cannot substitute for it.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X187, Scoped paragraph and cell text replacement (M)", complete [Issue 285](https://github.com/tensorbee/rdocx/issues/285) acceptance.
- `docs/hld/03-architecture.md`, "Facade conventions", comment identity, accepted-view story ranges and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Relationship types", checked story-owner mutation and comment companion preservation.
- `docs/hld/10-bindings-spec.md`, "The chosen design", "The invalidation problem, handled loudly" and "Python API shape", detached snapshots, checked paths and mutation errors.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "Binding tests" and "The hash harness", positive regression, round-trip and binding gates.

## Intake and sequencing

Reporter `hadim` opened Issue 285 on 2026-10-08 at 13:35 UTC. The complete issue body was read through GitHub against canonical source SHA `0a775842a8fd12f088bf4f2b3d0a049ddc3e976b`. No matching PR or contribution commit exists at intake. No contribution is accepted by this design. Preserve the issue URL and reporter attribution in final acceptance records. Issue 264 and F-X178 remain untouched.

Subsequent intake includes [PR 286](https://github.com/tensorbee/rdocx/pull/286), opened by `hadim` on 2026-10-08 at 14:15 UTC. Its immutable observed intake head is `08361f6af99110cbae71bf9f0498a12feadc01db`, based on main `20888b7a2c636e322ad23dc611f494beaac1c09b`. It claims to close Issue 285. Preserve and assess the complete contribution before implementation. Its paragraph and cell matcher reuse and source-built controls are candidates, not verified acceptance. The approved owned-document staging, checked block-control paragraph paths, exact preservation and complete scoped risk riders remain the contract. PR naming differs from the approved Document API and does not change it automatically. Reconcile the patch against the integrated prefix, extend missing criteria, remeasure archives locally and retain contributor attribution. Merge or closure requires complete verified issue acceptance at sprint close.

Subsequent approved remediation includes the direct-control ordinal correction from Hadrien Mary (`hadim`) in [PR 287](https://github.com/tensorbee/rdocx/pull/287), commit `64854a40c9e44505dfac98aa975b4ad3505a2ec8` at observed updated head `d823ba2ae205b6e9b9a71a75c4519031664d19d6`. Formal microscope pass 1 and actual native/current Python source-built controls demonstrated that a recursive paragraph ordinal could select a later direct sibling. Adapt the bounded approach by mapping actual facade paragraph identity to its direct outer-control ordinal and resolving the same direct ordinal for comment mutation. Nested control or table descendants return no two-segment location. This corrective behavior preserves the existing path axis and current namespace-closed snapshot API. Empty direct paragraph scanning already works and is not a new fix. Reopened comment anchors and scoped Paragraph replacement must select the same exact direct paragraph. Preserve contributor attribution and distinguish this measured pre-remediation failure from the exact-Base issue discriminator and after-only controls.

This is a batch draft under `/run-sprint`. The batch may describe unfinished dependencies, but implementation cannot begin before approval and completion of every formal prerequisite. F-X179 and F-271 are already done. F-282 must pause at an explicit saved external checkpoint before exclusive waves 11 through 14. F-X187 owns wave 14. F-X187 is formally independent but file-exclusive after F-X186. Resume full-catalogue F-282 after this intake, then F-283 in existing wave 10 only after F-282 completion. No concurrent source or Cargo ownership.

## Approach

Required Python signatures are `Paragraph.replace_text(old, new, *, expect=None) -> int`, `Cell.replace_text(old, new, *, expect=None) -> int` and `Document.replace_text_at(item: StoryItem, old, new, *, expect=None) -> int`. Keep StoryItem detached, constructible and frozen. Do not attach an originating Document or mutate through a disconnected snapshot.

Add the concrete owned native `try_replace_text_at(&mut self, location: &ContentLocation, placeholder: &str, replacement: &str, expected: Option<usize>) -> Result<Result<usize, ReplacementCountMismatch>>`. Reuse the existing mismatch type with singular index zero internally, mapped to the existing non-batch Python error attributes. Resolve owner fingerprint, item kind, bounds, revision and namespace closure before mutation. Support checked paragraph, table and content-control items through existing traversal, including header/footer and normal note paragraphs. Refuse fields, drawings and preserved nodes unless existing owner-qualified matching establishes the same boundary contract.

Python Paragraph and Cell route through their owning PyDocument, not borrowed Rust model wrappers. Body paragraph location resolution reuses paragraph_story_location, including block-control two-segment paths. Cell handles currently have no story-location getter, so add a concrete hidden native coordinate entrance `try_replace_text_in_cell(&mut self, cell: (usize, usize, usize), paragraph: Option<usize>, placeholder: &str, replacement: &str, expected: Option<usize>) -> Result<Result<usize, ReplacementCountMismatch>>`, or a smaller equivalent checked physical cell-location projection. The two actual consumers are cell-owned PyParagraph and PyCell. Both routes share the staging/count/publication implementation and matcher. No sentinel path or text-derived cell identity.

Reuse `rdocx-oxml/src/placeholder.rs` run-aware literal matcher and existing table/cell/control walkers. Preserve first matched run formatting and untouched prefixes, suffixes, comments, bookmarks and raw XML. Respect current wrapper and field stretch boundaries, accepted insertions and excluded deletions. Cell scope includes its paragraphs and supported nested tables and controls, never neighboring cells. Paragraph scope never joins separate paragraphs or silently visits a separate textbox story. Document.replace_text_at changes only the selected checked owner.

Validate XML characters, stage all replacements and compare the local count before committing. On mismatch, preflight failure or reopen failure, all document/package bytes and Python revisions stay unchanged. On a positive committed count, publish once and bump revision once. Zero matches remain a no-op and do not stale handles. Keep global document and CLI replacement behavior unchanged. Update existing Python stubs and typing tests without adding a test binary.

## Rejected alternatives

Global replacement followed by count filtering already mutates other owners. Whole-paragraph setters lose run formatting. An origin-document StoryItem mirror violates the existing detached snapshot contract.

## Test plan

**Test gate**: regression. `scoped_text_replacement_preserves_unselected_content` proves the reported failure before implementation and exact successful or refused behavior after save and reopen. Existing Rust, Python, typing and CLI entrypoints cover the complete issue criteria, with unrelated package members and opaque XML preserved. All 49 hash entries remain unchanged. Record reporter provenance and full acceptance for sprint close.

| Category | Test | Asserts |
|---|---|---|
| regression | `scoped_text_replacement_preserves_unselected_content` | Two identical clauses, split runs, nested cell descendants, header/footer/note owner scope and untouched owner bytes |
| round-trip | `scoped_replacement_preserves_anchors_properties_and_opaque_xml` | Formatting outside match, comments/bookmarks, namespaces and field/wrapper boundaries after reopen |
| Python | `scoped_replacement_count_guards_are_atomic` | Expect mismatch/error attributes, invalid text, stale handles, zero/no-op and one successful revision |
| typing | `scoped_replacement_signatures_cover_paragraph_cell_and_story_item` | Existing stub/runtime entrypoints use the three typed call shapes, detached StoryItem stays detached |

Use source-built fixtures and the existing `crates/rdocx/tests/regression_test.rs`, `integration_test.rs`, existing comments/document unit modules, `crates/rdocx-py/tests/test_core.py`, `tests/typing_smoke.py` and `crates/rdocx-cli/tests/integration.rs`. No new test binary. Prove each positive named gate fails against the exact Base before production implementation. Record real compiled gate outcomes and distinguish failure causes from missing test selection. Test namespace aliases, foreign lookalikes, schema order, exact opaque retention, malformed or ambiguous sources and prepare/reopen refusal without partial publication. Count runtime and typing acceptance separately.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Parser or serializer: read HLD04 and HLD06 and preserve schema child order, namespace-qualified identity and exact unmodeled subtree bytes. Round-trip and refused/no-op package comparisons cover aliases and shadowed namespaces.
- Public API of a published crate: read HLD10. Declare additive pre-1.0 API and the documented corrective behavior. Re-measure affected package archives and README inventories, enforce the 10 MiB ceiling and run actual locally patched `cargo publish --dry-run` checks without upload.
- PyO3 bindings: read HLD10. Compile affected bindings, run the actual built Python runtime and typing gates, and check `wasm32-unknown-unknown` for `rdocx-wasm` and `rpptx-wasm`. Linked Rust workspace tests exclude `rdocx-py` and `rpptx-py` as required.
- Run affected native all-target checks, focused tests, Clippy, fmt, denied-warning rustdoc, prose, adapter drift and workflow/README validation under `/verify --scoped`. The final integrated full gate and sprint review remain due.
- New workflow records are explicitly user-approved. No new production file, module, crate, dependency, trait or generic is planned. No external native oracle is required for these API, source ownership and atomicity contracts.

## Hash harness

Expected unchanged: all 49 deterministic harness entries, existing PDF resources and golden PNGs. No baseline movement is authorized. Any unexpected rendering or package fingerprint delta stops implementation for attribution and independent review rather than recording a replacement baseline. Source-built issue fixtures separately prove intentional editing behavior and untouched source retention.

## Implementation checklist

- [x] Approve the batch design. Confirm dependency completion and sole writer ownership before implementation.
- [x] Capture genuine fail-before evidence for the named issue gate and measured attribute-loss and control-ordinal defects. Qualify remaining controls as after-only.
- [x] Implement only the concrete existing-file API and source ownership contract.
- [x] Pass runtime, typing, CLI, exact preservation and atomic refusal controls.
- [x] Pass scoped risk riders, archive/README checks and zero-finding microscope.
- [x] Update exactly the HLD impact files and prepare the structured handoff.
- [x] Record implemented complete issue acceptance ready for root reconciliation at verified sprint close.

GitHub closure and final integrated acceptance remain root obligations after the full sprint gate and sprint review. Worker prepare records implementation evidence and does not close Issues 285 or 289.

## Open questions

None requiring a new user decision. The user approved the workflow records and the orchestrator selected the issue-permitted semantics above. Root reviewed and approved the contract with no unresolved material question. A new material unsupported source case must be reported with exact source evidence rather than silently narrowing the full issue contract.
