# F-X186, Move comment anchors without losing threads

**Status**: approved
**Sprint**: S90
**Size**: M
**Depends on**: F-X185

## Problem

Issue 284 requires moving a review thread before removing its paragraph. Existing add and remove operations cannot preserve thread identity through recreation. `crates/rdocx/src/comments.rs:483` already stages a checked story-range move, including a comment reference, but there is no checked root-id convenience API or CLI move command. `document.rs:15802` removes marker spans without pruning emptied Google wrappers.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X186, Move comment anchors without losing threads (M)", complete [Issue 284](https://github.com/tensorbee/rdocx/issues/284) acceptance.
- `docs/hld/03-architecture.md`, "Facade conventions", comment identity, accepted-view story ranges and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Relationship types", checked story-owner mutation and comment companion preservation.
- `docs/hld/10-bindings-spec.md`, "The chosen design", "The invalidation problem, handled loudly" and "Python API shape", detached snapshots, checked paths and mutation errors.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "Binding tests" and "The hash harness", positive regression, round-trip and binding gates.

## Intake and sequencing

Reporter `hadim` opened Issue 284 on 2026-10-08 at 13:35 UTC. The complete issue body was read through GitHub against canonical source SHA `0a775842a8fd12f088bf4f2b3d0a049ddc3e976b`. No matching PR or contribution commit exists at intake. No contribution is accepted by this design. Preserve the issue URL and reporter attribution in final acceptance records. Issue 264 and F-X178 remain untouched.

This is a batch draft under `/run-sprint`. The batch may describe unfinished dependencies, but implementation cannot begin before approval and completion of every formal prerequisite. F-X179 and F-271 are already done. F-282 must pause at an explicit saved external checkpoint before exclusive waves 11 through 14. F-X186 owns wave 13. F-X187 is formally independent but file-exclusive after F-X186. Resume full-catalogue F-282 after this intake, then F-283 in existing wave 10 only after F-282 completion. No concurrent source or Cargo ownership.

## Approach

Add `Document::move_comment(&mut self, id: i32, range: StoryRunRange) -> Result<()>` and `move_comment_to_text(&mut self, id: i32, anchor: &str, occurrence: usize) -> Result<()>`. Python exposes `move_comment(id, range)` and `move_comment_to_text(id, anchor, *, occurrence=0)`. Add `rdocx comment move <file> <id> --text <anchor> --occurrence <n>` with the existing output, JSON and atomic save conventions.

Check that id denotes an existing root rather than a reply. Locate its exact complete range/reference graph using F-X185. Reuse existing staged `move_story_range` at `comments.rs:483`, preserving numeric id, author, initials, date, comment text, reply descendants, resolved flag, companion identifiers and unrelated XML. Move the reference run without dropping neighboring content or properties. Support cross-story placement wherever existing checked comment placement permits it. Unknown ids, replies, ambiguous graphs, unsupported destinations and invalid paths refuse before publication. A source without the complete movable range/reference graph refuses clearly rather than inventing a point-to-range policy.

Find move-to-text targets with the same recursive main-body accepted literal search and exact display-span check as add_comment_on_text at `comments.rs:1205`. Separate finding and splitting from new-comment allocation. Do not create and delete a temporary thread. Rebase destination positions after old reference removal, including source and destination in the same paragraph and block-control two-segment paths.

Prune only goog_rdk wrappers whose sole content was the moved marker and which become empty from this move. Preserve wrappers with text, unrelated markers or unmodeled payload and preserve their namespace declarations. Stage all source removal, wrapper pruning, destination placement and package reopen before one publication. Comment parts and thread metadata should remain byte-identical when no unavoidable serializer change is earned. Python revisions advance once after success and remain unchanged on refusal. CLI writes through the existing sibling-file atomic publication route.

## Rejected alternatives

Delete and recreate loses identity and replies. Artificially rejecting all cross-story moves narrows existing supported placement. Pruning every empty SDT can destroy unrelated producer controls.

## Test plan

**Test gate**: regression. `comment_moves_preserve_thread_identity` proves the reported failure before implementation and exact successful or refused behavior after save and reopen. Existing Rust, Python, typing and CLI entrypoints cover the complete issue criteria, with unrelated package members and opaque XML preserved. All 49 hash entries remain unchanged. Record reporter provenance and full acceptance for sprint close.

| Category | Test | Asserts |
|---|---|---|
| regression | `comment_moves_preserve_thread_identity` | Same- and cross-story supported moves preserve all metadata, replies, resolved state and exact companion identity |
| round-trip | `comment_move_prunes_only_empty_google_marker_wrappers` | Source/destination same paragraph, marker-only SDTs, unrelated raw wrappers and surviving run content |
| Python | `comment_moves_refuse_unknown_reply_and_invalid_ranges_atomically` | Root lookup, stale ranges, unsupported stories and text occurrence failures preserve bytes/revisions |
| CLI | `comment_move_outputs_reopen_with_unchanged_thread` | Text/occurrence selection, atomic output and exact root-id continuity |

Use source-built fixtures and the existing `crates/rdocx/tests/regression_test.rs`, `integration_test.rs`, existing comments/document unit modules, `crates/rdocx-py/tests/test_core.py`, `tests/typing_smoke.py` and `crates/rdocx-cli/tests/integration.rs`. No new test binary. Prove each positive named gate fails against the exact Base before production implementation. Record real compiled gate outcomes and distinguish failure causes from missing test selection. Test namespace aliases, foreign lookalikes, schema order, exact opaque retention, malformed or ambiguous sources and prepare/reopen refusal without partial publication. Count runtime and typing acceptance separately.

## HLD impact

- `docs/hld/03-architecture.md`
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
- [ ] Capture fail-before evidence for the named positive gate and every reported operation.
- [ ] Implement only the concrete existing-file API and source ownership contract.
- [ ] Pass runtime, typing, CLI, exact preservation and atomic refusal controls.
- [ ] Pass scoped risk riders, archive/README checks and zero-finding microscope.
- [ ] Update exactly the HLD impact files and prepare the structured handoff.
- [ ] Reconcile complete issue acceptance at verified sprint close.

## Open questions

None requiring a new user decision. The user approved the workflow records and the orchestrator selected the issue-permitted semantics above. Root reviewed and approved the contract with no unresolved material question. A new material unsupported source case must be reported with exact source evidence rather than silently narrowing the full issue contract.
