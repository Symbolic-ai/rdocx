# F-283, Complete numbering-aware navigation fields

**Status**: approved
**Sprint**: S91
**Size**: L
**Depends on**: F-248, F-279 through F-282

## Scheduling

The user explicitly carried this unfinished story from S90 to S91 so completed
S90 work can release. No implementation is delivered by that scheduling change.
Reuse the existing approved design and saved evidence after its prerequisites
complete, with fresh evidence for the final source.

## Problem

Numbered STYLEREF is explicitly rejected at `crates/rdocx/src/field.rs:9798`, and numbering-demand detection recognizes REF only at `crates/rdocx/src/field.rs:9226`. Existing headings excludes table cells at `crates/rdocx/src/document.rs:26319`, and document_outline builds only that tree at `crates/rdocx/src/document.rs:26332`. Renderer outline titles use projected text without visible marker at `crates/rdocx-layout/src/engine.rs:2160`. Heading detection at `crates/rdocx-layout/src/engine.rs:6179` does not consume the complete effective outline-level cascade.

Existing authoritative numbering already contains marker, level, full context and literal-free values at `crates/rdocx-layout/src/style_resolver.rs:37`. Source-qualified numbering lookup exists at `crates/rdocx-layout/src/lib.rs:79`. This story composes these contracts rather than introducing another counter.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability matrix DOCX-050.
- `docs/hld/03-architecture.md`, "What stays put", immutable field evaluation, sequence state, native TOC discovery and result-local Word source maps.
- `docs/hld/08-rendering-spec.md`, "Word bookmark field pagination", "Word revision views", and "Word section geometry and page numbering".
- `docs/hld/12-testing-strategy.md`, numbered REF, TOC and Word source-path regression contracts.
- `docs/hld/14-development-backlog.md`, "F-283, Complete numbering-aware navigation fields" and F-248.

## Approach

Every prerequisite is a completion barrier. Reconcile final F-279 through F-282 contracts before claim. Reuse F-280's SequenceOptions, CaptionOptions, CaptionTarget and CrossReferenceOptions. Reuse F-279's immutable bookmark-target, page and section records and F-281's generated-result ownership.

Use one immutable WordLayoutResult for numbering and placements within each stage. A structural result rebuild may require a later snapshot because generated paragraphs affect pagination. Never treat an old snapshot as final page truth. Match sources by WordSourcePath or result-local SourceNodeId, not another story's flattened ordinal. Do not increment a separate navigation counter.

The root has settled additive compatibility. Existing headings and document_outline signatures and text semantics remain unchanged. Add:

```rust
pub struct NumberedOutlineNode {
    pub level: u32,
    pub text: String,
    pub number: Option<String>,
    pub display_text: String,
    pub source: rdocx_layout::WordSourcePath,
    pub children: Vec<NumberedOutlineNode>,
}
impl Document {
    pub fn numbered_document_outline(&self) -> Result<Vec<NumberedOutlineNode>>;
}
```

The additive result includes accepted body, table-cell and valid nested-control headings in physical document order using effective outline levels and resolved numbering. It retains source identity and separates title text from displayed marker. A suppressed marker returns number=None. New API construction propagates layout errors through Result rather than concealing them in an infallible legacy accessor.

Extend numbering-demand classification and STYLEREF evaluation for supported n, r, w and t combinations. Resolve style identity through the actual cascade. Nearest-source and page-local last-source selection retain distinct semantics. Page-dependent selection uses F-279 placement records, not whole-story reverse search. Unplaced or unsupported sources retain cache and ordered diagnostics. Pure evaluation does not invent layout-derived page scope.

REF reuses ResolvedNumbering::relative_to, level, full-context and literal-free values. Same-paragraph position and cross-story bookmark identity follow F-280. PAGEREF uses the same validated target and final placement as REF and captions. Repeated header/footer placements share physical source identity while retaining placement-specific page/STYLEREF values.

Heading eligibility uses effective outline levels, accepted revision projection and actual source paths. Direct body-text outline suppression overrides inherited heading style. numId=0 suppresses the marker without erasing an eligible heading. Deleted and moved-from paragraphs do not advance accepted counters. Respect accepted paragraph-mark joins. Hidden/suppressed numbering and heading suppression remain separate cases.

Share effective numbering and outline facts while preserving each consumer's
selection rules. The additive outline suppresses effective body-text outline
level 9. The captured TOC with both o and u excludes that source, while an
explicit custom-style selection and o without u include it. Page omission
for a selected level is separate from source exclusion. A numId=0 heading
has no outline marker, but numbered REF modes return the captured zero.
Do not replace these distinct outcomes with one missing-marker fallback.

Shared furniture STYLEREF selects by physical placement. In the captured
first page without a qualifying heading, both first and last modes search
forward to the first qualifying source. Later pages use their own first and
last sources. The initial saved shared-part caches reflect the last explicit update
placement and do not stand for every page's rendered value. Native reopen,
offline PDF export, save and close without F9 retained every baseline header
cache but changed all eight mutated header STYLEREF caches. The mutated
stored last-heading value became 2.4 while the PDF still rendered 2.1 on
page one, 2.4 on page two and 7.2 on page three. No intermediate snapshot
identifies which operation changed the caches. Do not infer a universal
placement policy from that lifecycle or require native Word to retain all
stored cache bytes. Ordinary library save/reopen retains its own published
caches, while Word oracle comparisons assert the captured lifecycle and
per-placement render separately. The equal H2
n, r and w results in this capture do not prove wider switch equivalence.
Retain discriminating mode tests over the shared authoritative facts.

Table-cell and nested-control headings consume the same authoritative numbering that visible layout uses. Propagate heading metadata into table pagination where needed. Renderer outlines use the resolved visible marker, suffix policy and accepted heading title. Preserve tab versus space suffix semantics in field results, while outline display uses a documented readable separator. No new number parser or conversion is added.

Generated TOC, TOF, TOA and bibliography ranges are excluded from new source discovery. Copied fields inside result caches must not advance caption sequences or manufacture source headings. The entire cache/navigation transaction stages, validates, reparses and commits once. Stale paths, ambiguous targets or invalid dependencies cannot partly publish. Ordinary save remains leave alone.

## Rejected alternatives

- Separate TOC counters disagree with table cells, restarts and accepted revisions.
- HeadingN spelling alone misses direct/inherited outline levels and suppression.
- Cross-story flattened ordinals select unrelated paragraph markers.
- Existing outline API changes would break consumers when additive results suffice.
- One full-context STYLEREF result for every switch loses Word semantics.

## Test plan

**Test gate**: regression. One multilevel numbered document stays consistent
across visible markers, navigation structures, references, and saved caches.

| Category | Test | Asserts |
|---|---|---|
| regression | multilevel_navigation_remains_consistent_after_mutation | Public source-built document agrees across visible markers, TOC, STYLEREF, REF, PAGEREF, captions, additive outline and reopened caches before/after insertion, deletion and restart |
| regression | table_and_control_headings_share_navigation_numbering | Body, table cells and nested controls use shared accepted order and source-qualified numbering |
| regression | suppressed_paragraphs_do_not_invent_navigation_markers | numId=0, direct outline body text, hidden markers, deleted content and paragraph-mark joins remain distinct |
| regression | styleref_number_modes_share_authoritative_counter | Level, relative, full and literal-free modes reuse authoritative values |
| regression | generated_results_are_not_navigation_sources | Old TOC, TOF, TOA and bibliography results create no new heading/sequence source |
| round-trip | numbered_navigation_caches_reopen_without_change | Library save/reopen retains ownership, bookmarks, switches and caches. Native Word lifecycle cache changes are separately classified against per-placement rendering |
| integration | navigation_updates_are_atomic | Stale paths, ambiguous targets and invalid dependencies preserve full bytes |
| differential | navigation_switches_match_fresh_word_record | Narrow STYLEREF page-selection and numbering/suppression cases match independently captured Word updates |
| differential | navigation_selectors_keep_distinct_suppression_rules | TOC o/u, o-only and custom-style selection distinguish outline suppression, numId=0 and page omission while numbered REF retains the measured zero result |

Extend existing tests, with no new binary fixture or integration binary. Use public dependency APIs to build the primary regression document. Assert marker values through source paths, not duplicated text matching. Classify native Word normalization rather than asserting strict package equality. The captured navigation reopen remaps three bookmark IDs and removes a row property exception in both cases. The mutated case also adds grammar markers and splits six REF cached runs without changing their text. Assert equivalent ranges, effective formatting and generated TOC content independently from these changes.

Pin fresh rider evidence to Microsoft Word for Mac 16.113.2 build 16.113.26092012, en-US and source fingerprint. Capture actual update, save and reopen before/after mutation, field values, style and numbering records, targets and entry order. Existing F-248 observations are supporting evidence only. Missing fresh switch evidence remains a failed rider.

Use deterministic fonts. Run cargo +1.97.1 in the isolated sprint target, scoped rdocx and rdocx-layout checks/tests, scoped verify and risk riders. Full sprint gate runs once after integration.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout/pagination: read rendering spec, deterministic fonts for every baseline and explicit expected delta before recording.
- Parser/serializer: read packaging and PresentationML conventions, prove ordered ownership, namespace qualification and preserved opaque content.
- Published API: read bindings spec/structural rules, additive pre-1.0 surface, canonical package dry run and 10 MiB assertion.
- External oracle: read differential-testing, assert exact Word build and record independent evidence.
- Crate boundary: existing facade-to-layout dependency direction only, no layout-to-facade edge.
- New file/module/trait/generic/flag: none.

## Hash harness

Expected unchanged. The seven samples are fixed at `scripts/hash_harness.py:33` and generated by generate_all_samples at `scripts/hash_harness.py:316`. Their numbered-looking heading prefixes are authored literal text, for example `crates/rdocx/examples/generate_all_samples.rs:177`. Numbered list items at `crates/rdocx/examples/generate_all_samples.rs:340` are separate non-heading paragraphs. The custom style at `crates/rdocx/examples/generate_all_samples.rs:616` inherits Normal and sets shading/run formatting, not direct outline or numbering. The generator does not call set_numbering or set direct outline levels on headings or table-cell paragraphs. Legacy headings/document_outline calls at `crates/rdocx/examples/generate_all_samples.rs:644` and `crates/rdocx/examples/generate_all_samples.rs:652` retain existing behavior. Therefore this additive API and newly numbered/table/direct-outline cases should not move the existing 49-entry baseline. No baseline claim is allocated. If integrated checks expose an unexpected change, stop and diagnose rather than record it.

## Exclusive file claims

- `crates/rdocx/src/field.rs`
- `crates/rdocx/src/document.rs`
- `crates/rdocx/src/lib.rs`
- `crates/rdocx-layout/src/lib.rs`
- `crates/rdocx-layout/src/engine.rs`
- `crates/rdocx-layout/src/style_resolver.rs`, only if a shared representation gap is demonstrated
- `crates/rdocx-layout/src/table.rs`, if heading metadata propagation requires it
- `crates/rdocx/tests/regression_test.rs`
- HLD files listed above

All prerequisites and conflicting writers finish before this story's wave.

## Implementation checklist

- [ ] Complete all prerequisites and reconcile final source/snapshot/reference contracts.
- [ ] Extend numbering-demand and STYLEREF evaluation.
- [ ] Implement additive source-qualified outline results and shared heading eligibility.
- [ ] Propagate table/control heading metadata through pagination.
- [ ] Cover suppression, revision joins and generated-result exclusion.
- [ ] Build the one-document public mutation regression and fresh Word rider capture.
- [ ] Verify atomic failure, repeated reopen and unchanged hash harness.
- [ ] Pass scoped verify and zero-finding microscope, then prepare handoff.

## Open questions

None. The user approved all six stories' workflow records and the dedicated
bibliography module. Native APIs are additive, existing outline APIs retain
their behavior, and fresh Word evidence is pinned to 16.113.2 build
16.113.26092012. INDEX and TOA use en-US source language with the captured Word
locale recorded explicitly. Technical ownership probes in the test plan must
pass before completion and may not be replaced by guessed expectations.
