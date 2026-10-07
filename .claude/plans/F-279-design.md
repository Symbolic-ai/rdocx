# F-279, Pagination field materialization across stories

**Status**: approved
**Sprint**: S90
**Size**: L
**Depends on**: F-263, F-271, F-272, F-273, F-274, F-278

## Problem

`Document::update_layout_backed_fields` stages one deterministic layout and
publishes changes atomically at `crates/rdocx/src/field.rs:875`. Its report counts
only PAGE, NUMPAGES and PAGEREF at `crates/rdocx/src/field.rs:684`. Story traversal
at `crates/rdocx/src/field.rs:908` builds header and footer paths only for direct
paragraphs and omits text-box cache updates.

Layout gives source identity only to the existing three page kinds at
`crates/rdocx-layout/src/engine.rs:6954`. Shared `FieldKind` lacks section variants
at `crates/oxml-layout/src/output.rs:198`. `WordLayoutResult` retains sources,
numbering and target names at `crates/rdocx-layout/src/lib.rs:59`, but lacks a
complete section placement and target-page snapshot. Cache formatting refuses
PAGE globally when any section uses non-decimal numbering at
`crates/rdocx/src/field.rs:10469`.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-279, Pagination field materialization across stories".
- `docs/hld/02-scope-and-non-goals.md`, DOCX-046 in the native Word capability matrix.
- `docs/hld/03-architecture.md`, "What stays put", staged updates, physical story identity and pagination boundary.
- `docs/hld/08-rendering-spec.md`, "Word bookmark field pagination" and section pagination and note-placement contracts.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", layout-backed update report and pure evaluation deferral.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and pinned Word note, section and page-field gates.

## Approach

Complete F-278 through a dependency-prefix checkpoint before starting. Reuse its
checked construction and lock state. Keep the existing public update entrypoints:

```rust
impl Document {
    pub fn update_layout_backed_fields(
        &mut self,
    ) -> crate::Result<LayoutBackedFieldUpdateReport>;
    pub fn update_page_fields(&mut self) -> crate::Result<usize>;
}

pub struct LayoutBackedFieldUpdateReport {
    pub page_fields: usize,
    pub num_pages_fields: usize,
    pub page_reference_fields: usize,
    pub section_fields: usize,
    pub section_pages_fields: usize,
    pub diagnostics: Vec<String>,
}

pub struct WordPageSection {
    pub physical_page: usize,
    pub displayed_page: usize,
    pub section_index: usize,
}

pub struct WordFieldPlacement {
    pub source: oxml_layout::FieldSource,
    pub physical_page: usize,
    pub displayed_page: usize,
    pub section_index: usize,
}

impl WordLayoutResult {
    pub fn page_sections(&self) -> &[WordPageSection];
    pub fn field_placements(&self) -> &[WordFieldPlacement];
    pub fn bookmark_page(&self, name: &str) -> Option<usize>;
}
```

`updated_count` includes all five kinds. Add format-neutral `Section` and
`SectionPages` classifications to the existing `FieldKind`. Word records remain
in existing `rdocx-layout/src/lib.rs`, preserving dependency direction.

Physical and displayed pages are one-based and section indices zero-based.
Multiple section records can occupy a physical page for continuous sections.
SECTION returns one plus owning section index. SECTIONPAGES counts distinct
physical pages occupied by that section. Fresh Word evidence settles exact
continuous-section and note ownership policy.

Retain target-page locations after substitution, rather than parsing formatted
PAGEREF display text. `bookmark_page` returns displayed page number. Missing,
ambiguous and unplaced targets return no value and retain caches with diagnostics.
Current target collection at `crates/rdocx-layout/src/engine.rs:2729` stores
physical page number. Reconcile physical and displayed target numbers explicitly
when preserving this map.

One staged deterministic layout is the authority for every cache changed by an
update. Immutable maps derived from that result provide field, section and
bookmark lookup for F-281 and F-283 too, without another pagination pass.

Register typed text-box paragraph sources without collisions with enclosing
body, header, footer or note paragraphs. Extend existing `WordStory` with:

```rust
TextBox {
    part_name: String,
    owner_children: Vec<usize>,
}
```

`WordSourcePath.children` identifies a paragraph within that text box. Discovery,
registration and edits use matching physical traversal through paragraphs,
tables, controls, notes and text boxes. Flatten field identity in preorder,
including nested instructions and cached result fields. Reconcile this with
`FieldSource.index` documentation and every producer and consumer.

Support PAGE, NUMPAGES, SECTION, SECTIONPAGES and PAGEREF in body, headers,
footers, footnotes, endnotes and supported typed text boxes, including nested
tables and modeled controls. Opaque stories remain preserved and produce
retained-cache diagnostics when encountered by inventory.

Format PAGE and PAGEREF using their owning section. Implement documented numeric
switches and section numbering formats required by oracle cases. Unsupported
instructions or switches keep their cache without suppressing supported fields
elsewhere. Locked fields retain cache and dirty spelling. Successful writes
become clean.

Shared header and footer cache selection follows fresh Word update and save
evidence. The existing first-placement rule is provisional. Preserve atomic
publication and related-story relationship closure. Pure evaluation defers all
supported layout fields and never starts layout. STYLEREF remains F-283.

Reconcile existing Python report compatibility when counters are added. Existing
methods remain usable. Extend frozen report count getters where required. No new
Python document method is planned.

No new source file, module, crate, trait, generic or feature flag is required.
Enum variants and exhaustive report literals have documented pre-1.0 source
compatibility impact.

## Rejected alternatives

- Layout per story produces inconsistent totals and targets.
- Main-story ordinal guesses misattribute nested tables and text boxes.
- Parsing rendered page text fails for formatted numbering.
- One section per page loses continuous-section ownership.
- Guessing shared-story cache policy fails the differential contract.
- Reusing old captures under a new Word label fabricates oracle evidence.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `pagination_field_caches_match_pinned_word_across_stories` | All five kinds match fresh Word updates in every named story. |
| differential | `section_fields_match_word_continuous_and_restarted_sections` | Section ownership, counts, displayed numbering and boundaries match Word. |
| differential | `shared_story_field_cache_selection_matches_word_save` | Default, first and even shared stories select Word's saved cache. |
| differential | `note_and_textbox_fields_match_word_placement_owner` | Continuations, section-end notes and anchored boxes use measured ownership. |
| regression | `nested_story_field_identity_is_collision_free` | Tables, controls, boxes and nested fields update the right physical owner once. |
| regression | `page_field_formatting_is_scoped_to_its_section` | Non-decimal sections do not suppress other supported caches. |
| regression | `locked_and_unplaced_page_fields_keep_original_cache` | Caches and dirty spelling remain with stable diagnostics. |
| round-trip | `page_field_cache_update_preserves_producer_xml` | Instructions, opaque XML, formatting and relationships survive. |
| regression | `pagination_update_publishes_one_atomic_candidate` | Failure leaves complete bytes unchanged and success uses one snapshot. |
| regression | `pure_evaluation_defers_every_layout_field` | The five kinds never trigger layout or cache writes. |

**Test gate**: differential. Body, header, footer, note, and text-box field caches
match the pinned Word page and section values.

Capture fresh normalized records from Microsoft Word for Mac 16.113.2 build
16.113.26092012, confirmed installed this sprint. Embed the authenticated pin
and records in existing tests. Older 16.112.3, 16.112.4 and 16.113 records are
not evidence for new S90 cases.

Construct inputs in Rust with explicit page breaks, section boundaries, fixed
geometry and bundled-compatible fonts. Open in Word, repaginate, update every
relevant story range, save, reopen and extract cache values with source identity.
Capture genuine update results without postprocessing values. Export PDF where
placement requires independent evidence. Use pinned Poppler 26.01.0 for PDF
page/text evidence. LibreOffice supports inspection but cannot replace Word.

Record version and build, input/output digests, procedure and normalized results.
Validate extraction against raw saved package XML. Native UI update completes any
story automation cannot update. Never report the gate passed with synthetic
expectations. Add tests to existing entrypoints, use deterministic layout, and
run checks with `RUSTUP_TOOLCHAIN=1.97.1` and isolated `CARGO_TARGET_DIR`, scoped
verify and zero-finding microscope.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout, pagination and text shaping. Read `08-rendering-spec.md`. Deterministic fonts for every baseline, one snapshot and no second pagination pass.
- Any parser or serialiser. Read `04-opc-and-packaging.md` and `06-presentationml-model.md`. Check sequence order, prefix tolerance and exact opaque preservation.
- Public API of a published crate. Read `10-bindings-spec.md`. Document enum and report impact, canonical publish dry-runs with local patches and 10 MiB assertion.
- Crate boundary work. Read `03-architecture.md`. Keep shared section identifiers format neutral and Word provenance in its family. Audit reverse dependencies.
- External oracle. Read `.claude/skills/differential-testing.md`. Authenticate exact Word version and capture fresh genuine results.
- PyO3 bindings if report getters change. Read `10-bindings-spec.md`. Check wrappers, canonical wasm32 check in integrated union, binding exclusions for workspace linking.
- New file. Approve required records. No new source file or module planned.

## Hash harness

Expected unchanged. The seven harness fixtures are generated exclusively by
`crates/rdocx/examples/generate_all_samples.rs`, selected by
`scripts/hash_harness.py:33` and regenerated at `scripts/hash_harness.py:321`.
The generator has no direct `Field::new`, `add_field`, `fldSimple`, `fldChar`,
`instrText`, SECTIONPAGES or SECTION field construction. Its SECTION strings are
comment labels only. It does author TOC through the existing convenience API.
Existing generated samples contain only TOC instructions in feature_showcase,
proposal and report, and none in the other four. No generated package contains
SECTION or SECTIONPAGES. Adding section substitution has no direct exposure.
No baseline update is planned. An unexplained delta blocks completion.

## Source file claims

- `crates/rdocx/src/field.rs`
- `crates/rdocx/src/document.rs`
- `crates/rdocx-layout/src/lib.rs`
- `crates/rdocx-layout/src/engine.rs`
- `crates/rdocx-layout/src/paginator.rs`
- `crates/rdocx-layout/src/notes.rs`, only if source registration requires it
- `crates/oxml-layout/src/output.rs`
- `crates/rdocx/tests/regression_test.rs`
- `crates/rdocx-py/src/document.rs`, existing frozen report getters and native conversion

Shared source and test entrypoint claims require serialization with F-280 through
F-283 unless final designs isolate those owners.

## Implementation checklist

- [ ] Complete F-278 before starting.
- [ ] Capture Word continuous-section and shared-story policy probes.
- [ ] Retain page, section, bookmark and field placement in one result.
- [ ] Add SECTION and SECTIONPAGES substitution.
- [ ] Register text-box and recursive field identities.
- [ ] Align discovery, placement and cache mutation traversals.
- [ ] Respect locked, unsupported, ambiguous and unplaced fields.
- [ ] Extend report counters and reconcile Python wrappers.
- [ ] Capture and check genuine pinned Word results.
- [ ] Pass focused tests, scoped verify and zero-finding microscope.

## Open questions

None. The user approved all six stories' workflow records and the dedicated
bibliography module. Native APIs are additive, existing outline APIs retain
their behavior, and fresh Word evidence is pinned to 16.113.2 build
16.113.26092012. INDEX and TOA use en-US source language with the captured Word
locale recorded explicitly. Technical ownership probes in the test plan must
pass before completion and may not be replaced by guessed expectations.
