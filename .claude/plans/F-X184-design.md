# F-X184, Safe comment ownership during content removal

**Status**: completed
**Sprint**: S90
**Size**: L
**Depends on**: F-X179, F-271

## Problem

`crates/rdocx/src/document.rs:19038` removes a direct body child without comment cleanup. The staged row and fragment routes at `document.rs:18731` and `17437` can likewise leave definitions without anchors. `comments.rs:1522` removes typed comments and commentsExtended descendants but does not establish selective commentsIds or commentsExtensible cleanup. Issue 282 reports this through Python and Google Docs wrappers, with no CLI validation error.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X184, Safe comment ownership during content removal (L)", complete [Issue 282](https://github.com/tensorbee/rdocx/issues/282) acceptance.
- `docs/hld/03-architecture.md`, "Facade conventions", comment identity, accepted-view story ranges and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Relationship types", checked story-owner mutation and comment companion preservation.
- `docs/hld/10-bindings-spec.md`, "The chosen design", "The invalidation problem, handled loudly" and "Python API shape", detached snapshots, checked paths and mutation errors.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "Binding tests" and "The hash harness", positive regression, round-trip and binding gates.

## Intake and sequencing

Reporter `hadim` opened Issue 282 on 2026-10-08 at 13:35 UTC. The complete issue body was read through GitHub against canonical source SHA `0a775842a8fd12f088bf4f2b3d0a049ddc3e976b`. No matching PR or contribution commit exists at intake. No contribution is accepted by this design. Preserve the issue URL and reporter attribution in final acceptance records. Issue 264 and F-X178 remain untouched.

This is a batch draft under `/run-sprint`. The batch may describe unfinished dependencies, but implementation cannot begin before approval and completion of every formal prerequisite. F-X179 and F-271 are already done. F-282 must pause at an explicit saved external checkpoint before exclusive waves 11 through 14. F-X184 owns wave 11. F-X187 is formally independent but file-exclusive after F-X186. Resume full-catalogue F-282 after this intake, then F-283 in existing wave 10 only after F-282 completion. No concurrent source or Cargo ownership.

## Approach

Add `Document::try_remove_content(&mut self, index: usize) -> Result<bool>` as the fallible native route used by Python. Keep `remove_content(index) -> bool`, returning false on out-of-bounds or refused removal without a panic. Document the fallible twin as the way to distinguish refusal. Existing valid uncommented removals retain their behavior.

On a staged candidate, inventory qualified source markers and references across actual relationship-resolved stories before removing body content, a whole table, a row, cell content or another supported story child. Correlate the exact removed spans with complete comment graph ownership. A whole safely owned graph removes its root, descendant replies and selectively linked commentsExtended, commentsIds and commentsExtensible rows. A surviving reference or endpoint makes a cut partial. Refuse partial cuts and comment-bearing `remove_content_at` or Python `pop_content` detach atomically, naming the comment. This is the issue's permitted refusal alternative and requires no comment-package ownership in ContentFragment. Unprovable imported linkage also refuses atomically. Preserve valid reference-only comments when their reference survives.

Extend `remove_comment` with the same checked companion cleanup. Resolve companions through actual relationship types and normalized targets, retain unrelated raw entries and attributes, and use last-paragraph ids plus durable-id linkage without assuming filenames. Existing field companion scans at `field.rs:10662` are source references, not an invitation to route generic comment lifecycle through REF evaluation. Do not rewrite unsupported opaque imported graphs. Remove an owned empty part only when its relationship and content-type ownership is established.

Cover `remove_content`, `remove_table_row`, checked story removal, whole table removal and cell destructive paths. The same staged reconciliation covers note owner deletion, section story pruning and revision resolution that discards commented source. Reuse the existing revision.rs candidate boundary without adding a second lifecycle. Namespace-invalid legacy removals refuse at the checked publication boundary and retain the original bytes. The document-unaware `Cell::remove_first_empty_paragraph` at `table.rs:1704` must retain or refuse marker-bearing empty paragraphs. Preserve existing checked Paragraph setter/run semantics and fix document-owned Cell text operations where clearing runs can discard a reference. Root selected additive `Cell::try_set_text(&mut self, text: &str) -> Result<()>` using the checked Paragraph setter, with unchanged legacy cell state on refusal. Hidden `Document::try_set_cell_text(&mut self, table: usize, row: usize, cell: usize, text: &str) -> Result<()>` supplies the real Python consumer with candidate staging and prepare/reopen before publication. Retain the final required direct cell paragraph. Reorder the existing rtf.rs import caller and two existing test builders to remove the default empty paragraph only after adding its replacements, preserving empty and formatted multi-paragraph imports. No blanket cleanup on save and no silent automatic repair of preexisting orphan documents. Destructive reconciliation first compares qualified raw marker counts, retaining untouched producer graph metadata when no ownership decreased. Additive story cloning keeps its established behavior. When decreased ids are all demonstrably undefined in actual relationship-resolved qualified raw definition entries, preserve their legacy editing and comparison lifecycle because no owned definition can be orphaned, without repair. Missing, external or ambiguous owned definition relationships and malformed definition sources remain unprovable and refuse. Full strict definition/companion proof applies to decreased defined ids, and explicit comment deletion and CLI validation remain strict.

Expose hidden `Document::validate_comment_ownership(&self) -> Result<()>` for the concrete CLI consumer, using the same qualified ownership inventory as destructive edits. This is the root-selected existing-file implementation clarification. Add raw all-story orphan checks to CLI validate at `commands.rs:2112`. An anchored root or reference-only root is valid. Replies linked through the surviving root thread do not require independent anchors. A known root with no range and no reference in any story is an error. Do not reuse globally strict `story_ranges()` as a raw presence scanner, because unrelated crossing pairs and accepted-view exclusions must not hide source references. Duplicate or unprovable linkage is diagnosed rather than guessed. Candidate failures preserve all live facade state and every package member. Python revisions advance once only after successful committed removal.

## Rejected alternatives

Silent orphan creation violates the issue. Blanket save-time deletion can destroy preexisting producer data. Carrying threads inside ContentFragment adds an unnecessary lifecycle when atomic detach refusal is explicitly allowed.

## Test plan

**Test gate**: regression. `comment_removal_preserves_thread_closure` proves the reported failure before implementation and exact successful or refused behavior after save and reopen. Existing Rust, Python, typing and CLI entrypoints cover the complete issue criteria, with unrelated package members and opaque XML preserved. All 49 hash entries remain unchanged. Record reporter provenance and full acceptance for sprint close.

| Category | Test | Asserts |
|---|---|---|
| regression | `comment_removal_preserves_thread_closure` | Whole-root/reply/grandchild and companion cleanup through body, table, row, cell and story removal, with source retention |
| round-trip | `comment_removal_refuses_partial_ranges_and_detach_atomically` | Start-only, end-only, surviving reference, discarded pop and unsupported imported graph leave exact package bytes |
| CLI | `validate_distinguishes_orphans_point_comments_and_linked_replies` | All-story references and valid replies are accepted, true orphan roots cause failure |
| Python | `comment_removal_paths_preserve_atomicity_and_revisions` | Reported three operations and Google wrappers, successful once-only invalidation and refused no-op |

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
- [x] Capture compiled fail-before evidence for the named thread/partial gates and row, pop, cell and CLI operations.
- [x] Implement only the concrete existing-file API and source ownership contract.
- [x] Pass runtime, typing, CLI, exact preservation and atomic refusal controls.
- [x] Pass scoped risk riders, archive/README checks and zero-finding microscope.
- [x] Update exactly the HLD impact files. Prepare the structured handoff in the feature-local completion phase.
- [x] Record implemented complete issue acceptance for root reconciliation at verified sprint close.

GitHub issue closure and final integrated acceptance remain root obligations after the full sprint gate and sprint review. Worker prepare does not close the issue.

## Open questions

None requiring a new user decision. The user approved the workflow records and the orchestrator selected the issue-permitted semantics above. Root reviewed and approved the contract with no unresolved material question. A new material unsupported source case must be reported with exact source evidence rather than silently narrowing the full issue contract.
