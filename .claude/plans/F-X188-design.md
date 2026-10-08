# F-X188, Preserve comment ownership when replacing or removing whole stories

**Status**: completed
**Sprint**: S90
**Size**: L
**Depends on**: F-X184

## Problem

[Issue 288](https://github.com/tensorbee/rdocx/issues/288), reported by Hadrien
Mary (`hadim`) on 2026-10-08, reproduces whole-story comment orphans against
main plus the open contributions. Completed F-X184 already reconciles note
and section-story removal. Text header/footer setters at
`crates/rdocx/src/document.rs:19610` through19660 and the shared installation
path at20193 still replace markers without reconciling their owned definitions.
`crates/rdocx/src/building_block.rs:588` similarly removes a glossary entry
without comment reconciliation. Authored-only section-part pruning at
`document.rs:22939` requires explicit imported last-reference coverage.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X188, Preserve comment ownership when replacing or removing whole stories (L)", complete Issue288 acceptance.
- `docs/hld/03-architecture.md`, "Facade conventions", shared ownership reconciliation and atomic publication.
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Relationship types", physical story ownership, glossary bundles, shared section references and retained producer graphs.
- `docs/hld/10-bindings-spec.md`, "The chosen design", "The invalidation problem, handled loudly" and "Python API shape", checked document ownership, error propagation and revision publication.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "Binding tests" and "The hash harness", exact-Base regression and retained source evidence.

## Intake and sequencing

The full issue and empty discussion were read against canonical93118299.
The read-only acceptance map is
`/private/tmp/S90-Issues288-289-acceptance-map.md`, SHA256
`b9b6b85de21af177c38c16bb7519717b0dfbdea500a93da93d9310006d418782`.
Source reasoning is not runtime proof. The user explicitly approved this
story's design, review, progress and handoff files. No contribution beyond
PR287's already assessed ownership work is assumed accepted.

Subsequent intake includes [PR290](https://github.com/tensorbee/rdocx/pull/290)
by Hadrien Mary (`hadim`), observed immutable head
`e57f18b963c901f2e81af21b707bb07fbde2d8fc`, stacked on updated PR287 head
`d823ba2ae205b6e9b9a71a75c4519031664d19d6`. The complete incremental
contribution and metadata were read. Its three new commits begin at
`5b0c37ac69fb5faa39427cc0b95b444f4649db2e`. The retained incremental patch
has SHA256 `69ced5022f302ab58d8ca2de7df4af01b2de67c99a7addfa40460d58e7372e7d`.
Reuse its destructive caller census, source-built test scenarios and specific
contributor provenance. Its successful header installation after malformed
companion cleanup fails conflicts with this approved atomic refusal contract.
Do not adopt that fallback or its separate absence-of-markers inventory.
Upstream test and archive measurements are not current acceptance evidence.

Subsequent [Issue292](https://github.com/tensorbee/rdocx/issues/292), reported
by `hadim` on 2026-10-08 at20:02 UTC, is included in this same ownership
story. The full issue and source routes were read. A commented fragment
imported into a glossary entry currently imports definitions into the main
comments part. Reverse extraction can likewise pair glossary-local markers
with main definitions of the same numeric id. The reporter permits either
omitting comments from building blocks or implementing glossary-local review
parts. This plan selects documented omission from transferred content, with
qualified owner isolation and successful valid output. Mere refusal of every
commented creation is not sufficient acceptance. The user corrected the
exclusion to Issue281, which remains open for F-184. Issue291 is included
as a separate field snapshot correction after this story. Issue264 remains
excluded. This story's ownership implementation scope is unchanged.

The updated contribution is assessed only through isolated Issue292 commit
`33ac78278aacbcd981f2e59c737f0c16d97732ea` by `hadim`, retained patch SHA256
`b452fd61a19eee973b1cd737374760f7470e617711768ae28841b0af7f3edb05`.
Updated PR290 head `f7841769bdf4c68e886f9e52657f1ba159cfcd12` and PR287 head
`39945370bbd18fc45ee0a1626e5bc8e9f73560da` carry separately assessed Issue291
ancestry. No entire stack is imported into this story. Omission preserves
carrier shells and refuses malformed qualified marker ids or opaque marker
payload. Real producer reverse-note controls require private selected closure
omission before capture. Glossary-local note relationships refuse ambiguous
main numeric projection. Local companions without definitions refuse, while
absence of all local review edges retains legacy main ownership proof.

F-X184 is completed. Run this story in exclusive wave15 after F-X187 because
document.rs, Python bindings and existing regression entrypoints overlap.
Keep full-catalogue F-282 paused at its authenticated checkpoint through this
wave. Resume it afterward, then F-283 after its formal prerequisites complete.
Issue289 belongs to the existing F-X185 checked snapshot contract. Issue264
and F-X178 remain excluded. Final integrated full verification and review
remain mandatory before publication.

## Approach

Reuse F-X184's qualified raw source inventory, complete thread and companion
proof, clone staging and prepare/reopen publication. Do not introduce a second
comment ownership authority or global save-time repair. Census every public
text, raw, image and section replacement entrance that shares header/footer
installation, including Default, First and Even through existing HdrFtrType
APIs. No separate set_even_header API is invented.

Add native `try_set_header(&mut self, text: &str) -> Result<()>`,
`try_set_footer(&mut self, text: &str) -> Result<()>`,
`try_set_first_page_header(&mut self, text: &str) -> Result<()>` and
`try_set_first_page_footer(&mut self, text: &str) -> Result<()>` so Python's
existing setters propagate checked errors through their owning document.
Keep existing infallible native signatures as documented compatibility
wrappers using the existing expect convention. Valid complete removals clean
up their threads. Callers requiring recoverable refusal use the fallible
entrances. Never publish a failed candidate or silently convert an error into
successful output. Test fallible refusal and once-only Python revision changes.

Reconcile at the complete operation boundary after all source changes and
before one publication, including all raw/image consumers. Remove only safely
owned roots, descendants and selectively linked commentsExtended, commentsIds
and commentsExtensible rows. Preserve unrelated comment threads, package
members, relationships, content types and opaque XML. A partial cut, malformed
graph or unprovable companion linkage returns a diagnostic naming the affected
comment where identity is known, with exact original bytes retained.

Glossary removal uses the same reconciliation before its existing reopen,
retains the valid empty glossary bundle after last-entry removal, and keeps
other entries and their dependencies intact. Comment-bearing fragment
detachment retains F-X184's refusal policy. Do not adopt PR287's incompatible
automatic re-anchoring or carried-thread semantics.

The destructive glossary census also includes `replace_building_block` and
`update_building_block_from_fragment`. Reconcile old-body ownership after the
complete staged replacement, including dependency import, and before final
publication. `update_building_block` delegates to replacement while requiring
an unchanged body, so its metadata-only control must retain the original
thread and companions. Prove typed and retained raw glossary coherence so a
later flush cannot restore removed markers. These are existing entrances to
the same complete Issue288 contract, with no new public API or file.

For Issue292, `create_building_block_from_fragment` and
`update_building_block_from_fragment` omit namespace-qualified comment markers
and their definition import from the private transferred content before
dependency capture and import. Include selected note and textbox dependencies
so no companion transfer reintroduces main-owned orphan comments. Plain dependency-free creation retains its existing atomic refusal of
qualified comment-bearing bodies. Direct replacement omits qualified markers
from newly supplied bodies. Preserve metadata-only updates. Reverse
`building_block_fragment` and `insert_building_block` omit selected glossary
markers before constructing a main-owned fragment, never pairing numeric ids
with unrelated main definitions. Existing general fragment behavior and
destructive commented-fragment detach refusal remain unchanged.

Reuse qualified marker spans and existing dependency closure. Preserve mixed
reference-run properties, wrappers, foreign lookalikes, comments, PIs and
unmodeled XML. No arbitrary run or container deletion is authorized. Source
documents and retained glossary-local comment parts and relationships remain
unchanged by transfer. All changes publish through one staged transaction.

Resolve glossary comment ownership from its physical relationship owner.
Exactly one valid internal comments relationship identifies local markers,
which must not enter the main comment graph even when ids match. No local
comments relationship permits legacy main-owned processing only through the
existing strict ownership proof. Duplicate, external, missing-target,
wrong-root or malformed local ownership refuses atomically. Existing local
entry cleanup must prove its complete thread and companion closure through
the existing strict graph machinery with the physical owner explicit, or
refuse. Never delete an equal-id main thread or claim omitted local cleanup.

A normalized physical marker-bearing note source cannot belong to both main
and glossary review graphs. Each graph can otherwise accept the same numeric
anchor with its own definition, so qualified source overlap refuses before
cleanup or validation. Shared note parts without qualified comment markers
remain supported. This applies the existing physical-owner proof, not a
second id inventory or a blanket shared-note restriction.

Shared header/footer ownership is resolved by normalized physical target,
including two relationship ids reaching one part and opaque references.
Removing one section reference preserves a still-used thread and its companion
bytes. Removing the final effective reference must not leave a listed orphan
merely because imported part pruning retains its source. Establish exact
source behavior with a saved/reopened before control, then reconcile safely
owned marker and definition closure while preserving retained producer parts
and opaque references. Do not broaden authored-only package pruning or delete
producer parts just to make inventory counts decrease. Any concrete conflict
between complete cleanup and preservation must be reported before narrowing
the issue contract.

## Rejected alternatives

Global orphan deletion can erase unrelated imported review state. Deleting
producer-owned header parts changes existing package ownership rules.
Replacing X184 with PR287's auto-reanchor path abandons the approved atomic
partial-cut and fragment policy. A helper's presence does not prove an untested
operation's acceptance.

## Test plan

**Test gate**: regression.
`whole_story_removal_and_replacement_preserve_comment_closure` fails against
the exact claimed Base for uncovered replacement and glossary routes, then
passes source-built cleanup, shared-reference, companion, namespace and atomic
refusal controls after save and reopen. Existing Rust, Python and CLI
entrypoints cover the complete issue criteria. All49 hash entries remain
unchanged. Record reporter provenance and full acceptance for sprint close.

| Category | Test | Asserts |
|---|---|---|
| regression | `whole_story_removal_and_replacement_preserve_comment_closure` | Whole roots, replies and grandchildren removed with text/raw/image header/footer content and glossary entries, exact unrelated companions retained |
| round-trip | `shared_story_comment_ownership_survives_until_last_reference` | Default/First/Even Header/Footer, imported reopen, inherited/shared targets and different relationship ids, first removal preserves and final removal reconciles |
| integration | `whole_note_removal_reconciles_comment_companions` | Actual remove_footnote and remove_endnote operations, aliased relocated note parts, root/reply/companion cleanup and unrelated owner preservation |
| Python | `whole_story_comment_refusals_preserve_bytes_and_revision` | Existing setters use fallible owned route, success bumps once, malformed/partial/preparation/reopen refusal keeps bytes and handles unchanged |
| CLI | `whole_story_outputs_validate_without_comment_orphans` | Reopened outputs pass strict ownership validation, malformed controls fail with original graph diagnostics |

The existing glossary controls also cover direct replacement, replacement
from a dependency-backed fragment, and metadata-only update. Demonstrate
actual before failures for uncovered destructive replacements, complete
root/reply/companion cleanup, unrelated entry retention and atomic malformed,
partial-cut and prepare/reopen refusal. Retain PR290's malformed companion
fixture as a refusal control with original content and revision unchanged.

Issue292 controls reproduce actual commented creation, fragment update,
extraction and same/different-document insertion with the chosen omission
policy, save/reopen and strict CLI validation. Imported Word-style glossary
local comments must coexist with equal-id unrelated main comments without
wrong-owner pairing. Check exact retained source and local package bytes,
selected note/textbox dependencies, direct supplied bodies, metadata-only
updates, namespace aliases and shadows, mixed reference runs, foreign
lookalikes, opaque XML and complete failure atomicity. Source reasoning is
not an executed before failure. Extend existing entrypoints only.

Extend only existing unit modules and integration entrypoints. No new test
binary, production file, module, crate or dependency. Prove actual compiled
before failures for the named gate and reported uncovered operations, never
missing new APIs or deselected tests. Test alias/shadowed namespaces, preserved
wrappers, foreign attributes, comments and PIs, unselected thread metadata,
shared producer targets and atomic prepare/reopen failure. Runtime and typing
acceptance are recorded separately. No native Word capture is needed for these
package ownership and transaction contracts.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Parser or serializer: read HLD04 and HLD06. Respect schema child order and namespace-qualified identity. Exact source and package comparisons prove unmodeled subtree retention and refused/no-op atomicity.
- Published public API: read HLD10 and CLAUDE structural rules. Additive pre-1.0 fallible setters preserve existing signatures. Remeasure changed published archives and affected README inventories, assert the10MiB ceiling and run actual locally patched publication dry runs without upload.
- PyO3: read HLD10. Compile actual current bindings and run rebuilt Python runtime, exact pinned strict typing and stub agreement. Check both WASM bindings. Linked native workspace tests exclude binding crates as required.
- Scoped verification includes affected all-target checks, full affected native suites, all-feature denied-warning Clippy, fmt, denied-warning docs, hash49, README, workflow, prose and adapter checks, plus every rider above. Final full workspace verification and sprint review remain due.
- Workflow files are explicitly approved. No new production file/module/dependency/trait/generic is planned. Shared source, Cargo and hash execution remain exclusive to this wave. No baseline movement is authorized.

## Hash harness

Expected unchanged: all49 deterministic entries and existing PDF/PNG resources.
Source-built editing controls prove the intentional package behavior separately.
Unexpected deltas block acceptance rather than earning a replacement baseline.

## Implementation checklist

- [x] Read complete issue, verify prerequisite completion and approve design with explicit workflow-file permission.
- [x] Claim wave15 only after exclusive prior waves release source and Cargo.
- [x] Capture genuine exact-Base failures for all uncovered destructive routes.
- [x] Implement shared reconciliation and fallible Python-bound text setters with complete preservation and shared-reference semantics.
- [x] Pass exact native, Python, typing, CLI and atomic refusal controls.
- [x] Update exactly the HLD impact files, pass scoped riders and zero-finding microscope.
- [x] Prepare and validate the structured worker handoff for integration.

- [x] Integrate the reviewed feature and record complete scoped acceptance in the canonical delivery ledgers.

Canonical integration1674306a consumes the validated worker handoff and
matches the reviewed CodeHead exactly. Delivery checkpoint264f5ef2 records
complete Issues288/292 scoped acceptance. Final full integrated sprint
verification, sprint review and external disposition remain pending. The
retained worker branch has the feature CodeHead followed by a handoff-only
commit.

## Open questions

None requiring a new user decision. The user approved the new records and
requested inclusion before publication. Root selects complete owned cleanup,
recoverable fallible native/Python refusals and retained legacy wrapper
conventions. A measured unsupported source or preservation conflict must be
reported before changing these criteria.


## Installer output validation clarification

Full affected native tests identified live identifier changes when the new
installer boundaries published the reparsed output. Existing numbering handles
and raw image relationship identities must remain stable until save. The ten
new boundaries therefore reconcile the complete staged candidate, validate
its cloned complete output through actual preparation and reopen, then commit
the original staged candidate once. This follows existing output-validation
precedent and preserves deterministic subsequent save output. Other glossary
and mutation reopen behavior is unchanged. All four existing regression
assertions and real final-reopen failure atomicity remain required.


## Invalid header text refusal clarification

Complete installer output proof also rejects XML-invalid header text before
publication. The fallible setter returns the original OPC part and code-point
diagnostic. Its legacy wrapper follows the established panic convention and
keeps exact pre-operation bytes. The existing regression's header subsection
now proves both paths. Unrelated paragraph and footnote deferred save-refusal
controls remain unchanged. This intentional early refusal changes no valid
document output and requires no hash baseline delta.


## Formal D1 selected closure clarification

Forward glossary omission visits only the prepared selected main body and its
recursive referenced footnote and endnote owners, including textbox content.
It does not scan unrelated note owners in a retained related part. The same
bounded note traversal serves forward actual-main ownership and reverse
glossary transfer. Physical main identity permits main note relationships,
while genuine glossary-local note ambiguity still refuses before numeric
lookup. Derive the forward closure after package preparation so canonical
note ids remain aligned. Same-part selected/unselected opaque and malformed
carriers exercise create and update success, selected-carrier atomic refusal,
source-byte retention and actual selected note presence. Current-before
evidence is distinct from the original claimed Base.


## Selected body clarification after formal D2
Forward glossary transfer obtains namespace-complete selected content from the
prepared physical body through the existing package-authoritative extraction.
Initial note selection, marker omission and glossary dependency capture exclude
retained main-root producer payload. Generic transfer retains its existing
dependency inputs and ownership admission. Clean content replaces only its physical interval. Final
section properties are omitted separately only when the fragment includes
them, retaining their original section owner for existing downstream handling.
Excluded final section bytes and all outside-body bytes remain private source
payload. Existing generic fragment identity admission is unchanged, including
its refusal of incomplete comment triples within retained body descendants.
A current-before producer background control executes creation and update
failure and selected-body atomic refusal. Its after control requires successful
plain transfer with source bytes unchanged. This is later-current evidence,
not an additional exact claimed Base run.
