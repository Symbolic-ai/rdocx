# F-X186, Move comment anchors without losing threads

**Status**: completed
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

### Implementation qualification

PR287 by `hadim`, immutable head `e4b216934eb3abc58ea1a42d72a902f90aa8e120`, supplies compatible move-routing and shared literal-finder reference material. Implementation adapts those ideas to the existing strict F-X184 ownership and F-X185 accepted-source contracts. It does not import the contribution's partial-range reanchoring, pop carrier or temporary-thread allocation alternatives.

The existing typed `CommentReference` parser discarded unmodeled reference attributes before a move could capture them. Preserve paired source and attributed empty references through a private raw carrier in the existing `CT_R.extra_xml` and position mechanism, owned by the exact typed child boundary and qualified identity. Serialization emits the carrier once, removal and replacement discard it, and run splitting and segment writing keep its ownership. Only its private flag and exact child boundary establish carrier ownership, never numeric identity alone. Unordered independent raw references remain verbatim, including when private positions have been erased. The own Word alias declaration does not turn an otherwise plain id-only reference into opaque payload. Foreign declarations, attributes and paired payload remain preserved. Plain authored references retain their established serialization. Capture standalone run comments and processing instructions at their existing raw child boundary so mixed-run transport cannot discard their source order. This earned existing `text.rs` repair adds affected oxml verification and archive measurement and requires semantic reconciliation with paused F-282.

Moves preserve neighboring children of mixed reference runs in their original source order while copying the selected direct reference, run attributes, properties and required namespace closure. Destination rebasing uses physical paragraph source offsets and the actual accepted reference-run index. Only whole-run removal decrements that index. Newly empty Google controls prune only when their full raw skeleton proves no unmodeled payload, attributes or namespace declarations. Internal destination fingerprints refresh after staged anchoring, while caller stale identities remain checked.

The shared literal finder uses hidden `CT_P::comment_range_text_from_source(xml: &[u8], id: i32) -> Result<Option<String>>` through private `Document::comment_literal_range_text(&mut self, id: i32) -> Result<Option<String>>`. After the mutable finder releases its borrow, the unpublished candidate flushes only its document model through existing namespace replay. The safeguard reads that actual main-part source with its root, body, table, cell, paragraph and control scopes. It validates all element and attribute namespace prefixes across the complete XML before extracting qualified Word `t` text, so neither a selected range end nor a skipped text subtree hides invalid namespace loss. Tabs and breaks remain zero width. Valid foreign names and local shadowing remain distinct from unresolved prefixes. The legacy helper and rich accepted-display contract remain distinct. Hidden `CT_P::accepted_comment_source_projection(xml: &[u8], id: i32, open_at_start: bool) -> Result<(String, bool, Option<usize>, Option<usize>)>` shares the existing rich projection and endpoint-fidelity routine between the original instance method and facade listing. The facade passes its already namespace-closed original paragraph source. The private fidelity probe retains that source's root bindings and local shadowing without normalizing text or changing the accepted axis. Direct move controls cover namespace contexts, foreign lookalikes, tabs, breaks, field refusal, reopen, metadata and no temporary thread allocation.

Comment move replays existing qualified nested-owner namespace declarations before each of its three edited main-source write boundaries and before the literal safeguard, using the existing owner proof and declaration replay. Only an owner proved to contain the selected qualified comment pair before or after the move may refresh its private logical snapshot. That refresh requires a unique same-kind owner with the exact preserved raw namespace-marker multiset and matching namespace facts. Original declarations and markers remain authoritative, ambiguous owners refuse and all unselected owners retain the strict existing snapshot match. During source removal, the existing qualified marker inventory and exact comment-move edit routine also remove the selected source from the original package namespace authority before deriving retained owners. Entire transported runs disappear from that authority, while surviving mixed runs retain their declarations and every non-reference child. The namespace-closed captured reference carries its own properties and opaque attributes to restoration. No unrelated absent owner is discarded. This preserves package namespace authority without changing global story-source fingerprints. Exact Base paragraph and block-control alias-field sources are saveable, so adjacent literal moves must succeed with the raw field and its declaration retained after reopen. Crossing the field remains an exact-display refusal. Initially unsaveable table and cell alias-field fixtures retain their existing serialization failure, with earlier atomic add refusal under the checked publication contract. Three raw alias SDT forms remain outside the existing accepted literal axis and refuse with no occurrence, while representable block and modeled text contexts remain positive.

Rich comment query restores main-part namespace context once through the existing strict declaration replay. Paired XML events prove that names, non-namespace attributes and payload remain identical, yielding exact physical boundary correspondence. The mapped canonical and restored story-owner inventories must be bijective in physical spans, kind and owner index before canonical paragraph paths receive actual namespace scopes. Additional or reinterpreted owners refuse instead of redirecting a query. The existing checked paragraph span traversal serves both discovery and projection, with paragraph scopes inventoried in a bounded pass. Canonical locations, fingerprints and accepted run indices remain authoritative. Imported paragraph and block-local alias fields retain rich A7B, source-byte purity and usable move/reopen endpoints. Comment destination anchoring replays retained declarations before replacing package authority, while other anchor kinds keep their existing path. Redundant declaration replay is omitted only for a unique exact owner whose complete non-namespace event payload and local declarations match, with identical resolved element and attribute namespace facts throughout that owner. Redundant child declarations on a transported reference do not require global replay to reinterpret its shape. This owner-local proof permits unrelated modeled source normalization. Needed declarations still use the unchanged strict replay. The private pre-removal endpoint probe validates the caller's exact accepted endpoints without publishing destination namespace context, while the real Comment destination always preserves that authority before replacing source. Same-owner modeled alias plus local raw-field fixtures that already fail initial save remain outside this correction. Supported controls include separate unrelated root-bound modeled aliases and normalized properties in the same package.

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
- [x] Capture fail-before evidence for the named positive gate and every reported operation.
- [x] Implement only the concrete existing-file API and source ownership contract.
- [x] Pass runtime, typing, CLI, exact preservation and atomic refusal controls.
- [x] Pass scoped risk riders, archive/README checks and zero-finding microscope.
- [x] Update exactly the HLD impact files and prepare the structured handoff.
- [x] Record implemented complete issue acceptance for root reconciliation at verified sprint close.

GitHub closure and final integrated acceptance remain root obligations after the full sprint gate and sprint review. Worker prepare does not close Issue 284 or accept all of PR287.

## Open questions

None requiring a new user decision. The user approved the workflow records and the orchestrator selected the issue-permitted semantics above. Root reviewed and approved the contract with no unresolved material question. A new material unsupported source case must be reported with exact source evidence rather than silently narrowing the full issue contract.
