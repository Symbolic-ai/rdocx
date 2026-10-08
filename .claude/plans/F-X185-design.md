# F-X185, Expose comment anchor text and story location

**Status**: approved
**Sprint**: S90
**Size**: M
**Depends on**: F-X184

## Problem

`crates/rdocx/src/comments.rs:350` exposes metadata without an anchor. The frozen Python Comment at `crates/rdocx-py/src/document.rs:167` and CLI JSON at `crates/rdocx-cli/src/commands.rs:1353` repeat that gap. Issue 283 requires accepted-view text and a usable typed story range rather than external XML inspection.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X185, Expose comment anchor text and story location (M)", complete [Issue 283](https://github.com/tensorbee/rdocx/issues/283) acceptance.
- `docs/hld/03-architecture.md`, "Facade conventions", comment identity, accepted-view story ranges and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Relationship types", checked story-owner mutation and comment companion preservation.
- `docs/hld/10-bindings-spec.md`, "The chosen design", "The invalidation problem, handled loudly" and "Python API shape", detached snapshots, checked paths and mutation errors.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "Binding tests" and "The hash harness", positive regression, round-trip and binding gates.

## Intake and sequencing

Reporter `hadim` opened Issue 283 on 2026-10-08 at 13:35 UTC. The complete issue body was read through GitHub against canonical source SHA `0a775842a8fd12f088bf4f2b3d0a049ddc3e976b`. No matching PR or contribution commit exists at intake. No contribution is accepted by this design. Preserve the issue URL and reporter attribution in final acceptance records. Issue 264 and F-X178 remain untouched.

This is a batch draft under `/run-sprint`. The batch may describe unfinished dependencies, but implementation cannot begin before approval and completion of every formal prerequisite. F-X179 and F-271 are already done. F-282 must pause at an explicit saved external checkpoint before exclusive waves 11 through 14. F-X185 owns wave 12. F-X187 is formally independent but file-exclusive after F-X186. Resume full-catalogue F-282 after this intake, then F-283 in existing wave 10 only after F-282 completion. No concurrent source or Cargo ownership.

## Approach

Expose `Document::comment_anchor(&self, id: i32) -> Result<Option<StoryRunRange>>` and `Document::comment_anchor_text(&self, id: i32) -> Result<Option<String>>`, with corresponding checked `CommentRef::anchor()` and `CommentRef::anchor_text()` accessors. Keep existing CommentRef metadata and comments ordering. Unknown ids and malformed or ambiguous graphs return errors. A known orphan returns no range and no text. A reference-only point comment returns no paired range and `Some("")`. Replies have no independently invented or copied parent range.

Use the shared checked comment ownership inventory from F-X184 and existing `StoryRangeRef` and `story_range_paragraphs` at `comments.rs:104` and `document.rs:16189`. Extract only accepted-view span text with Paragraph.text semantics, including supported tracked insertions and Google block and inline goog_rdk wrappers. Join intervening paragraphs with exactly one newline. Do not substitute whole containing paragraphs, accepted_literal_text or the single-paragraph local-name scanner `rdocx-oxml/src/text.rs:5932` for the required accepted-view range projection.

Add optional anchor_text and anchor fields to frozen PyComment, retaining the existing seven constructor arguments and defaulting the new values to None for detached manually constructed records. Materialize the existing typed PyStoryRunRange with StoryItem snapshots, revision and checked index paths. Preserve direct_body_index where available and actual part/owner identity for cells, headers, footers and notes. No originating-document mirror is added.

CLI comment list JSON retains existing fields and adds anchor_text plus an anchor object containing typed start/end locations: story kind, normalized part name, owner index, item kind, index_path, run_index and direct_body_index when available. Null anchor is distinct from empty text. Replace misleading main-only scope wording where necessary. Point and orphan distinctions survive JSON. Document all-story discovery without claiming opaque or unsupported markers are writable.

## Rejected alternatives

Untyped dictionaries discard the existing range API. Literal-only text omits accepted display content. Giving every reply its parent range invents an anchor the source did not author.

## Test plan

**Test gate**: regression. `comment_anchor_snapshots_match_accepted_story_spans` proves the reported failure before implementation and exact successful or refused behavior after save and reopen. Existing Rust, Python, typing and CLI entrypoints cover the complete issue criteria, with unrelated package members and opaque XML preserved. All 49 hash entries remain unchanged. Record reporter provenance and full acceptance for sprint close.

| Category | Test | Asserts |
|---|---|---|
| regression | `comment_anchor_snapshots_match_accepted_story_spans` | Exact single/two-paragraph, cell, tracked insertion and block/inline goog_rdk text and typed range |
| round-trip | `comment_anchor_point_orphan_and_reply_states_remain_distinct` | Point empty text, orphan None and no invented reply range, with normalized owner paths |
| Python | `comment_anchor_fields_preserve_constructor_compatibility` | Seven original constructor fields, optional defaults, typed snapshots and usable add_comment range |
| CLI | `comment_list_json_reports_typed_anchor_locations` | Exact body index 1 reproduction plus non-body/index-path and point/orphan JSON |

Use source-built fixtures and the existing `crates/rdocx/tests/regression_test.rs`, `integration_test.rs`, existing comments/document unit modules, `crates/rdocx-py/tests/test_core.py`, `tests/typing_smoke.py` and `crates/rdocx-cli/tests/integration.rs`. No new test binary. Prove each positive named gate fails against the exact Base before production implementation. Record real compiled gate outcomes and distinguish failure causes from missing test selection. Test namespace aliases, foreign lookalikes, schema order, exact opaque retention, malformed or ambiguous sources and prepare/reopen refusal without partial publication. Count runtime and typing acceptance separately.

## HLD impact

- `docs/hld/03-architecture.md`
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
