# F-X185, Expose comment anchor text and story location

**Status**: completed
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

Issue [289](https://github.com/tensorbee/rdocx/issues/289), also reported by `hadim` on 2026-10-08, identifies empty text and XML on paragraph endpoints inside block controls. This is covered by the existing checked paragraph snapshot contract, including both endpoints, accepted paragraph text, nonempty paragraph XML, two-segment path and actual containing body index. Existing body and header block controls remain positive. The built-runtime reproduction checks body path `(1, 0)` before and after reopen without source mutation. Optional expansion of ordinary story listing is not required for these endpoint APIs. No additional production behavior or global inventory expansion is introduced.

## Approach

Expose `Document::comment_anchor(&self, id: i32) -> Result<Option<StoryRunRange>>` and `Document::comment_anchor_text(&self, id: i32) -> Result<Option<String>>`, with corresponding checked `CommentRef::anchor()` and `CommentRef::anchor_text()` accessors. Keep existing CommentRef metadata and comments ordering. Unknown ids and malformed or ambiguous graphs return errors. A known orphan returns no range and no text. A reference-only point comment returns no paired range and `Some("")`. Replies have no independently invented or copied parent range.

Use the shared checked comment ownership inventory from F-X184 and existing `StoryRangeRef` and `story_range_paragraphs` at `comments.rs:104` and `document.rs:16189`. Extract only accepted-view span text with Paragraph.text semantics, including supported tracked insertions and Google block and inline goog_rdk wrappers. Join intervening paragraphs with exactly one newline. Do not substitute whole containing paragraphs, accepted_literal_text or the single-paragraph local-name scanner `rdocx-oxml/src/text.rs:5932` for the required accepted-view range projection.

Use the bounded hidden existing-file helper `CT_P::accepted_comment_projection(&self, id: i32, open_at_start: bool) -> Result<(String, bool, Option<usize>, Option<usize>)>`. Namespace-qualified source marker spans select private temporary paragraph projections, with original source untouched. Reuse the existing parser, accepted text and accepted run-path axis for display and endpoint coordinates. Validate re-anchoring fidelity at exact accepted display offsets, not merely successful insertion or literal-text searching. Unknown, duplicate, hidden, mid-run or unrepresentable selected endpoints return explicit checked errors. A measured wrapper-only paired source displays WRAPPER with zero accepted runs, while re-adding 0..0 yields empty text, so this existing-axis representational gap is an unsupported-range error rather than orphan or point state. Representable surrounding wrappers, ordinary fields, tracked insertions and Google controls remain positive cases. Do not widen the global accepted axis.

PR [287](https://github.com/tensorbee/rdocx/pull/287), by `hadim`, has immutable head `e4b216934eb3abc58ea1a42d72a902f90aa8e120` and patch SHA-256 `6b3a4192641616d1f5e1a7e761c58cc5219de5ff4531ea129e0d3e49bcef025f`. Its accepted-operation traversal, typed Python scaffolding and source-built fixtures inform adaptation. The approved checked API and ownership/error semantics remain authoritative. No broad stacked patch, lazy document mirror, swallowed extraction error, synthetic bookmark collision or raw-count fallback is accepted. Complete PR acceptance waits for all associated issue contracts and the final integrated gate.

The shared private owned-graph proof validates relationship-resolved qualified definitions and every companion before listing. Demonstrably undefined raw marker IDs do not invent records or block established comparison reads, and their bytes remain unchanged. Strict destructive ownership checks and CLI validation still reject undefined markers. Missing, external or ambiguous owned linkage and malformed definitions remain errors. Candidate marker and ordered owner paragraph indices reuse that one inventory, limiting extraction to the selected source owner and span.

Hidden `Document::comment_anchor_snapshots(&self) -> Result<BTreeMap<i32, (Option<StoryRunRange>, Option<String>)>>` serves the actual Python and CLI listing consumers with one shared qualified source inventory and owned checked snapshots. Single-id methods select only the requested extraction after shared graph proof. No lazy cache or public anchor wrapper is added.

Hidden `Document::story_range_paragraph_snapshot(&self, location: &ContentLocation) -> Result<StoryItemSnapshot>` reuses checked range paragraph source and namespace closure for the actual Python and CLI endpoint consumers. It returns exact owner/path, accepted paragraph text and XML, with the containing direct body index only for actual body owners, including two-segment block-control paragraph paths. Ordinary story inventory remains unchanged. The same checked snapshot supplies supported two-segment paragraph handles returned through the public Python StoryRunPosition constructor, with existing stale-handle guards and ordinary paragraph behavior retained.

Returned multi-paragraph StoryRunRange values reuse the existing qualified raw endpoints-first insertion path. The observed old typed setter changes the first paragraph before checking the second, invalidating its own whole-owner fingerprint. Capture both exact source spans before one publication, including paragraphs in a body block control, while retaining genuine stale-range refusal. No global fingerprint normalization or refreshed identity after partial mutation is allowed.

Add optional anchor_text and anchor fields to frozen PyComment, retaining the existing seven constructor arguments and defaulting the new values to None for detached manually constructed records. Preserve existing frozen Comment equality over the original seven metadata fields. Derived anchor snapshots carry document revisions and do not change comment metadata identity. Assert new fields directly and typed range equality and stale guards separately. Materialize the existing typed PyStoryRunRange with StoryItem snapshots, revision and checked index paths. Preserve direct_body_index where available and actual part/owner identity for cells, headers, footers and notes. No originating-document mirror is added.

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
- [x] Capture actual compiled accepted-text and CLI failures, built Python failure, and the Issue289 constructor before control.
- [x] Implement only the concrete existing-file API and source ownership contract.
- [x] Pass runtime, typing, CLI, exact preservation and atomic refusal controls.
- [x] Pass scoped risk riders, archive/README checks and zero-finding microscope.
- [x] Update exactly the HLD impact files and prepare the structured handoff.
- [x] Record implemented complete issue acceptance for root reconciliation at verified sprint close.

GitHub closure and final integrated acceptance remain root obligations after the full sprint gate and sprint review. Worker prepare does not close Issues 283 or 289.

## Open questions

None requiring a new user decision. The user approved the workflow records and the orchestrator selected the issue-permitted semantics above. Root reviewed and approved the contract with no unresolved material question. A new material unsupported source case must be reported with exact source evidence rather than silently narrowing the full issue contract.
