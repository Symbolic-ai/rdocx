# F-279, Pagination field materialization across stories

**Status**: completed
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

Format PAGE using its field owner's section and PAGEREF using the target's
section. A unique placed target permits unlocked PAGEREF materialization even
when the source story is unused. Unplaced PAGE, NUMPAGES, SECTION and SECTIONPAGES
retain their source caches. A note field retains its printed physical page, but
its displayed page and section values follow its unique body reference. Missing
or conflicting body references retain caches with diagnostics.

Implement documented numeric
switches and section numbering formats required by oracle cases. Unsupported
instructions or switches keep their cache without suppressing supported fields
elsewhere. Locked fields retain cache and dirty spelling. Successful writes
become clean.

Fresh Word 16.113.2 build 16.113.26092012 captures require literal header and
footer PAGE and NUMPAGES caches to remain unchanged across simple and complex
forms, first, even and default variants, and singly placed owners. Their dynamic
rendered values still use the immutable placement snapshot. Other reachable
furniture fields use their measured owner. Unused source stories retain their
owner-dependent caches while unique target-owned PAGEREF may update. Preserve atomic
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
| differential | `continuous_furniture_and_restart_caches_match_authenticated_word` | Section ownership, counts, displayed numbering and boundaries match Word. |
| differential | `pagination_field_caches_match_pinned_word_across_stories` | Default, first and even shared stories select Word's saved cache. |
| differential | `continuous_note_caches_follow_authenticated_reference_owners` | Continuations, section-end notes and anchored boxes use measured ownership. |
| regression | `nested_story_field_identity_is_collision_free` | Tables, controls, boxes and nested fields update the right physical owner once. |
| regression | `page_field_formatting_is_scoped_to_its_section` | Non-decimal sections do not suppress other supported caches. |
| regression | `locked_and_unplaced_page_fields_keep_original_cache` | Caches and dirty spelling remain with stable diagnostics. |
| round-trip | `page_fields_in_related_tables_and_controls_keep_physical_owners` | Instructions, opaque XML, formatting and relationships survive. |
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

## Authenticated target-format rider

Fresh Word 16.113.2 mixed-format captures distinguish PAGE's owning field
section from PAGEREF's owning target section. The immutable result adds
`pub fn bookmark_page_section(&self, name: &str) -> Option<WordPageSection>`.
The existing `bookmark_page` accessor remains compatible and returns that
record's displayed page. Missing, ambiguous and unplaced targets return None.
The target section selects the unformatted PAGEREF cache's numeric style.

## Source file claims

The existing low-level field owner requires a claim rider. Its parsed complex
model retained nested cached-result fields only as producer bytes, and authored
cached runs were private. Hidden ordered cache traversal and edit hooks retain
nested field representation and source invalidation without flattening it.
This exported hidden surface receives the public API and parser riders above.

The approved text-box rich-body rider extends existing
`crates/rdocx-oxml/src/drawing.rs` with an optional `CT_Body` projection in
`CT_Shape`. It must replace the paragraph-only execution path when present,
retain legacy `shape.text` behavior when absent and preserve raw DrawingML.
Its public struct literal impact, scoped namespace handling, cell clipping,
source identity and all existing constructors are part of the API, parser and
archive riders. An additional optional source_text_box_owner member retains the
physical selected owner ordinal in the actual OPC part. Native projection binds
namespace-qualified source anchor occurrences before layout, counting earlier
unregistered owners and consuming identical anchors separately. Existing anchor
capture trims text, so comparison applies exactly that parser event projection
to the source span without changing producer bytes. Logical owner_children paths
remain independent of physical owner ordinals. A hidden snapshot owner accessor
joins cache staging to this physical identity, with exact field source validation
inside that owner. The hidden text_box_paragraph_index accessor supplies the
owner-local paragraph preorder, distinguishing identical fields in different
rows without matching their displayed text. Missing or conflicting identity
retains the source cache.

The rich-story rider adds `pub story_bodies: HashMap<WordStory, CT_Body>`
to `LayoutInput`. A present physical-story projection replaces the legacy
paragraph-only layout and source inventory for that header, footer or note.
Absent projections preserve legacy callers. Native projections resolve actual
OPC owners and close inherited namespace scopes without changing package bytes.
Related-story tables reuse row painting, cell wrapping and clipping. Their rows
enter the existing furniture and note flow as indivisible row lines, retaining
paragraph source paths through nested table and control children. Existing
constructor consumers initialize the map. This pre-1.0 struct member earns the
public API and package archive riders. Rich cache staging uses the existing
namespace-qualified physical paragraph scanner and checked field source edits.

The existing input.rs settings rider adds modern_footnote_layout and
footnote_layout_like_word8 boolean members, projected from typed compatibility
settings by document.rs. Legacy constructors default both to false. Mode 15 and
later use modern continuous note flow. In the default and explicit mode 12
captures, an actual placed body footnote before the continuous boundary advances
the following section. WW8 true instead defers advance until the next actual
footnote-bearing block. Orphan definitions, endnotes and note-part reference
markers do not trigger that condition. Actual Word immediate and delayed anchor
pairs gate the branch, with body-only capture procedures explicitly limiting
cache assertions. Section-end notes consume the last merged member's flow band,
and deferred document-end notes resume after it without overlap. These document
policy fields also participate in reusable-engine input equality.

The physical-story rider adds the resolved OPC names to `LayoutInput`. This is
an additive pre-1.0 struct member and existing literal consumers initialize its
map. An absent mapping leaves text-box provenance unavailable rather than
inventing a name. Existing table and note shaping sites bind anchored child
paragraphs before pagination. The authoring and archive riders apply.

- `crates/rdocx-layout/src/input.rs`, physical story part-name map
- `crates/rdocx-layout/src/table.rs`, anchored child source binding and constructor defaults
- `crates/rdocx-layout/src/math.rs`, existing constructor default only
- `crates/rdocx/tests/integration_test.rs`, existing constructor default only
- `crates/rdocx-oxml/src/text.rs`, nested cached-result field ownership and checked edit hooks
- `crates/rdocx-oxml/src/drawing.rs`, optional authoritative rich text-box body projection

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

- [x] Complete F-278 before starting.
- [x] Capture Word continuous-section and shared-story policy probes.
- [x] Retain page, section, bookmark and field placement in one result.
- [x] Add SECTION and SECTIONPAGES substitution.
- [x] Register text-box and recursive field identities.
- [x] Align discovery, placement and cache mutation traversals.
- [x] Respect locked, unsupported, ambiguous and unplaced fields.
- [x] Extend report counters and reconcile Python wrappers.
- [x] Capture and check genuine pinned Word results.
- [x] Pass focused tests, scoped verify and zero-finding microscope.

## Open questions

None. The user approved all six stories' workflow records and the dedicated
bibliography module. Native APIs are additive, existing outline APIs retain
their behavior, and fresh Word evidence is pinned to 16.113.2 build
16.113.26092012. INDEX and TOA use en-US source language with the captured Word
locale recorded explicitly. Technical ownership probes in the test plan must
pass before completion and may not be replaced by guessed expectations.

## Review remediation clarification

Nested cached display segments retain their actual field owner and formatting.
Only instruction operands inherit the outer display placement. A cached child
must reach its own visible layout position, including page and column breaks.
Invisible cached children retain their source caches with diagnostics. The
existing text.rs hidden public cached_display_field_segments hook exposes this
identity to the Word engine. The engine's hidden public document_sections hook
uses its modeled main-story item inventory for facade format and referenced
furniture lookup, including control-owned section-ending paragraphs. Both hooks
are public API riders within the existing source claims and package gate.

The completion policy rider updates the existing expected-owner inventory in
`scripts/test_sprint_workflow.py` when DOCX-046 becomes complete and Owner becomes
none. Every incomplete row still requires one pending or in-progress live owner.
The row describes modeled all-story materialization and records opaque,
unsupported, locked and unplaced preservation alongside Word furniture cache
policy. This is the same truthful capability completion boundary used by F-278.

The existing Python rider updates `crates/rdocx-py/python/rdocx/_rdocx.pyi` and
`crates/rdocx-py/tests/test_core.py` to declare and test both frozen count getters,
five-count totals and backward-compatible constructor defaults. These existing
binding files belong to the public API and isolated Python verification claims.

The existing `crates/rdocx-py/tests/test_python_docx_parity.py` rider retains producer footer caches and missing cache runs in the established producer matrix and issue-158 workflow. General non-pagination field discovery retains its legacy direct-body section scope. The pagination inventory alone includes modeled control section endings. Test labels above name the implemented entrypoints, with the same differential gate and obligations. Related rich-row continuation and section-end note tests supplement the reference-owner Word cases.

The independent pass-2 provenance rider carries structural note_reference_source
separately through the existing format-neutral TextSegment and positioned text
and multilingual glyph structs. Generated note glyphs keep exact text source
None. Existing line shaping, painting, source remapping and exhaustive constructor
sites are claimed, including constructor-only consumers in font, chart, PDF,
DOCX SVG and presentation render code. No new source file, trait or module is
introduced. Public struct literal and archive consumer verification applies.
The existing oxml-layout, rdocx-layout and rdocx-oxml crate README measurement
rows are refreshed from actual reviewed archive evidence with the validator.


The pass-2 physical-owner rider adds hidden metadata-only accessors in existing
`crates/rdocx-oxml/src/revision.rs` and
`crates/rdocx-oxml/src/content_control.rs`. They traverse insertion, deletion,
move-source, move-destination and control projections before accepted filtering.
Original revision and control XML stays authoritative. Binding uses an atomic
layout-only candidate. Every physical anchor occurrence consumes its source
slot, including opaque occurrences outside exposed story grammar. Logical
projected run paths include accepted revision runs. The existing
CT_R::raw_child_has_alternate_content_drawing predicate is exposed as hidden
public metadata so direct and selected MC drawing projections consume actual
run-child slots in order. Raw producers and unselected branches are preserved.

Authoritative rich story bodies and actual OPC names participate in reusable
layout context and furniture cache identity. Default and explicit document-end
notes retain bounded restart completion through the final recorded free band.
Unused note parts stay eligible. Existing middle-tail and final-completion
performance limits remain, with short fitting and measured multi-page note
payloads compared against fresh deterministic layout.

Additional existing constructor and structural-channel source claims are:

- `crates/oxml-layout/src/line.rs`
- `crates/oxml-layout/src/font.rs`
- `crates/oxml-chart/src/lib.rs`
- `crates/oxml-pdf/src/font.rs`
- `crates/oxml-pdf/src/lib.rs`
- `crates/oxml-pdf/src/writer.rs`
- `crates/rdocx/src/svg.rs`
- `crates/rpptx-render/src/text.rs`
- `crates/rpptx/src/lib.rs`
- `crates/rpptx/src/pdf.rs`
- `crates/rdocx-oxml/src/revision.rs`
- `crates/rdocx-oxml/src/content_control.rs`
- `crates/oxml-chart/README.md`
- `crates/oxml-pdf/README.md`
- `crates/rpptx-render/README.md`
- `crates/rpptx/README.md`
- `crates/oxml-layout/README.md`
- `crates/rdocx-layout/README.md`
- `crates/rdocx-oxml/README.md`
- `README.md`
- `scripts/readme_doctests.py`
- `scripts/test_sprint_workflow.py`

The changed native crate gate covers oxml-layout, oxml-chart, oxml-pdf,
rdocx-oxml, rdocx-layout, rdocx, rpptx-render and rpptx. The changed rdocx-py
crate uses the isolated rebuilt-wheel suite above. Constructor-only PPTX and
neutral consumers add no reverse dependency or Word-specific inference.
Public docs and all 22 patched local-source archive consumers remain riders.
Actual measurements and verified publish dry-runs are distinct checks.

Fresh saved-output Word reopen evidence is authenticated for all 18 cases,
957 retained fields and 180 successful source-contract checks. Word 16.113.2
build 16.113.26092012 opens the exact prior saved outputs offline, exports Best
printing PDF, saves and closes without F9. No stored cache, dirty or lock value
changes. Original partial-update procedures and OLD sentinels remain explicit.
The root evidence is recorded in `/private/tmp/S90-F279-reopen/manifest.json`
and `summary.json`, with exact source, initial saved and reopened artifact hashes.
This authenticates the required Word reopen lifecycle without upgrading partial
initial updates into full native cache comparisons.


The same-run physical-order regression declares DrawingML namespaces on the
preserved document root. An additional diagnostic input declared wp, a and wps
only on w:drawing. That is valid OOXML, but the existing CT_Drawing typed writer
omits declarations on its wrapper while retaining the inner anchor bytes.
The renderer's original typed projection can still see the box, while the
reconstructed story XML cannot establish its namespace-qualified physical owner.
The ordered binder therefore leaves its metadata unbound and does not guess.
This pre-existing local-wrapper serialization limitation is retained for the
independent preservation/contract review. The root-scoped regression proves
physical order without claiming to repair that separate namespace case.


Implementation review adds namespace, ancestry and furniture discriminators.
Nested simple-cache wrappers reuse the effective producer Word alias and never
replace an occupied prefix. Aliased owner writes retain the foreign binding and source text aliases,
while source-less generated result elements receive canonical Word scope locally. The hidden
Field.cached_display_owner_is_locked accessor retains every cached ancestor lock.
Relationship activation and inheritance use the modeled main-story section order,
including block controls and excluding table-cell section properties. Existing
MERGEFIELD discovery retains its deliberately separate traversal boundary.
The existing regression entrypoint covers foreign opaque cache bytes and genuine
identities through reopen, three-level simple/complex middle locks with flags,
and unique first furniture inherited after a control-owned ending section.
These are existing-file public-surface and archive consumer riders. A document
root that shadows w remains subject to the existing modified-serialization
rejection. The foreign-cache regression binds w on the aliased outer field and
does not widen that separate root boundary or the local drawing limitation.
