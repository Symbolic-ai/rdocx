# F-280, Captions, sequences, and complete cross-references

**Status**: completed
**Sprint**: S90
**Size**: M
**Depends on**: F-248, F-275, F-278

## Problem

Sequence evaluation and partial REF evaluation exist, but the native facade
does not provide caption construction or checked cross-reference insertion.
`crates/rdocx/src/field.rs:9595` evaluates SEQ instructions, while
`crates/rdocx/src/run.rs:505` only constructs a field from instruction and
cached text. Callers must currently assemble labels, sequence runs and
bookmark targets themselves.

REF supports numbering context and main-story relative position at
`crates/rdocx/src/field.rs:9412`. Its position comparison uses paragraph
ordinals, so a target in the same paragraph keeps the stored cache at
`crates/rdocx/src/field.rs:9488`. Its switch whitelist at
`crates/rdocx/src/field.rs:12258` omits delimiter and footnote-reference
switches. Story-local bookmark authoring exists at
`crates/rdocx/src/comments.rs:393`, but caption construction does not compose
that checked ownership with field insertion.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-280, Captions, sequences, and complete cross-references".
- `docs/hld/02-scope-and-non-goals.md`, capability DOCX-047.
- `docs/hld/03-architecture.md`, "What stays put", paragraphs defining the recursive Word field grammar, story-local sequence counters, accepted revision projection and atomic field cache updates.
- `docs/hld/03-architecture.md`, "Facade conventions", checked story ownership and paired story ranges.
- `docs/hld/08-rendering-spec.md`, "Word bookmark field pagination", REF numbering and relative position.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The private from-scratch DOCX conformance corpus".

## Approach

Add concrete option and result types in the existing field module. Re-export
them through the existing crate root.

```rust
pub struct SequenceOptions {
    pub restart: Option<i64>,
    pub repeat: bool,
    pub hidden: bool,
    pub restart_heading: Option<u8>,
    pub format: Option<String>,
}

pub struct CaptionOptions {
    pub label: String,
    pub text: String,
    pub bookmark: String,
    pub sequence: SequenceOptions,
    pub separator: String,
    pub properties: Option<CT_PPr>,
}

pub struct CaptionTarget {
    pub paragraph: ContentLocation,
    pub entire_caption: String,
    pub label_and_number: String,
    pub number: String,
}

pub enum CrossReferenceNumber {
    Text,
    Level,
    Relative,
    FullContext,
}

pub struct CrossReferenceOptions {
    pub number: CrossReferenceNumber,
    pub position: bool,
    pub hyperlink: bool,
    pub omit_non_numeric_text: bool,
    pub delimiter: Option<String>,
    pub copy_referenced_notes: bool,
}

impl Document {
    pub fn insert_sequence(
        &mut self,
        position: &StoryRunPosition,
        identifier: &str,
        options: &SequenceOptions,
    ) -> Result<()>;

    pub fn insert_caption(
        &mut self,
        before: &ContentLocation,
        options: &CaptionOptions,
    ) -> Result<CaptionTarget>;

    pub fn insert_cross_reference(
        &mut self,
        position: &StoryRunPosition,
        bookmark: &str,
        options: &CrossReferenceOptions,
    ) -> Result<()>;
}
```

The option structures use direct fields and Default where a meaningful default
exists. They do not introduce a builder, trait, generic parameter or new
module.

Caption insertion creates one paragraph before the checked story location.
The paragraph contains separate label, sequence and title runs, with the
requested paragraph properties. An empty title is valid. An empty label or
invalid sequence identifier is rejected before publishing changes. Figure,
Table and Equation are ordinary label values. Custom labels use the same
contract.

The bookmark option is a requested base name. Allocate collision-free names
and IDs for three targets: entire caption, label plus number, and number
alone. Return their actual names. Allocation must inspect all physical story
parts and preserved producer markers. Each target covers its intended
accepted-view run range. No target stores a second caption text representation.

Construct sequence and REF instructions through F-278's checked
FieldInstruction and Field constructors, with ordered cached runs. F-280
owns the private checked insertion helper in field.rs. Use the existing
ContentLocation and StoryRunPosition ownership checks, staged package patching,
prepare-and-reopen validation and single commit_staged_mutation publication.

The existing low-level CT_P owner adds a hidden public concrete
insert_accepted_run(boundary, CT_R) operation returning RangeAnchorError.
It reuses the accepted_range_sites zero-range resolver and existing direct or
control insertion machinery. The two current consumers are checked sequence
and cross-reference insertion. Preserve the established rejection of revision
interiors and unsupported control boundaries, and leave the paragraph unchanged
on failure. This existing-file cross-crate helper avoids synthetic marker
round trips and adds no new type, trait or generic. Its pre-1.0 public surface
joins the low-level model tests, documentation and verified archive riders.

Typed REF-f copies require a hidden concrete Field::set_cached_runs(Vec<CT_R>)
operation in the existing text.rs owner. Validate an unpublished cloned candidate
with validate_cached_field_runs and checked serialization, refuse a locally
locked field, and leave the original unchanged on failure. Ancestor locks remain
the facade source-inventory responsibility. Keep scalar cached_result consistent
with ordered typed runs and remove superseded cache children. Retain original
form, source identity, span, instruction, dirty and lock values and control XML.

The source-aware simple and complex writers replace only the outer result
payload through namespace-aware field-depth boundaries. Same-run complex fields
split the original run wrapper around that payload, retain rPr and attributes on
preserved controls and outside text, and emit supplied sibling runs without
nested w:r. Preserve instruction operands and propagate a modified nested cache
through its owning field. A private typed override participates in unchanged,
child projection and display checks only while its scalar projection agrees.
The current consumers are typed footnote, endnote and annotation REF-f copies.
Retain parsed complex cached CT_R runs as private source projections without
marking the field changed. A hidden cached_result_runs read-only slice getter
serves facade target extraction and layout note-reference discovery and shaping.
Empty scalar text must remain distinguishable from typed note or comment
references after reopening. Keep unchanged complex source bytes verbatim and
exclude cached generated fields from physical source interpretation. Prove
no-op byte preservation, typed parse/update/reopen, nested cache boundaries and
actual note discovery and rendering, not just serialized reference presence.
Facade mutation remains staged and validates prepare/reopen before publication.
Cover empty results, simple and complex forms, same-run prefix/suffix, nested
rich caches, locks, alternate prefixes and malformed-cache failure atomicity in
the existing low-level entrypoint. Include this additive hidden mutation in the
low-level documentation, all-target consumers and verified archive riders.

Annotation REF-f copying also needs paragraph-level comment range markers.
Add a hidden concrete Field::set_cached_runs_with_comment_ranges operation in
existing text.rs, taking Vec<CT_R> and the existing Vec<CommentRangeMarker>.
Clone first, reuse checked typed-run replacement, and validate paired unique
range IDs, ordered run-boundary positions and matching typed references before
publishing. Write commentRangeStart and commentRangeEnd as cache siblings
between runs, never inside CT_R::extra_xml. Retain the marker projection in the
private typed override and source-aware simple/complex writers, including
same-run controls, nested caches and no-op serialization. The current concrete
consumer is captured annotation REF-f copy materialization. No new type, file,
trait or generic is introduced.

A hidden read-only Field::cached_result_comment_ranges slice getter exposes
owned cached CommentRangeMarker values to the existing facade target-range
extractor. Keep marker boundary positions in semantic field equality and
source-aware nested cache projections. Prove parsed simple and complex caches
retain their typed Field ownership and preserve no-op source bytes. Reopened
annotation copies must retain actual range endpoints, typed references and
owned comment definitions, while their copied literal content still paints
through the existing field cache path. Mere XML well-formedness or typed run
presence does not establish that ownership and literal-display contract.

The current layout engine treats CommentReference as nonpainting and has no
Word comment-owner input or comment-callout renderer. Preserve that rendering
boundary. F-280 does not add a speculative callout or anchor-overlay surface.
DOCX-042 and F-275 own paired annotation ranges, and F-293 owns broader modern
comment metadata authoring. REF-f closes only the captured copied dependency
graph needed by the materialized field. Native Word anchor-outline pixels
remain qualified oracle lifecycle evidence, not a new library paint target.

The existing no-marker setter remains the ordinary typed-run operation.
The marker operation owns validation and sibling serialization rather than
only forwarding. Prove empty and literal ranges, exact outer instructions and
controls, schema-valid marker placement, locks, nested cache propagation and
complete failure atomicity in the existing low-level test entrypoint. Copied
annotation owners require fresh comment IDs, paragraph IDs and durable graph
identities with relationship closure, validated through prepare and reopen.
Do not borrow source comment IDs or substitute the full comment-thread importer
for the independently measured field-copy contract. Include this hidden API in
the existing low-level documentation, all-target and verified archive riders.

The native physical-owner probe separately measures main-body, selected body
text-box and ordinary default-header references to one bookmarked footnote.
Main-body references allocate four typed note copies. The measured body box and
ordinary header keep bookmarked literal text and allocate no notes. These
partial stages are retained in /private/tmp/S90-F280-ref-owner-controls with
source fingerprint 850f3e823b02dc1c20aed0cb684a90f2be150b1e4f6b7a9d06b45e6e6b2b2758.
The completed native probe also explicitly updates the ordinary footer and
selected header and footer boxes. All five measured new contexts retain empty
MARK and literal-only WHOLE results without typed references or additional
notes. Word moves default furniture between package parts during editing and
collapses it again during export and save. Follow section relationships rather
than treating emptied former parts as lost fields. All 31 rich field contracts
survive the distinct no-F9 reopen. The two one-page Word PDFs retain exact text
and 96dpi pixels. This supports the captured plain target and shapes, not a
universal rich copy policy or Rust rendering parity. The context-note cache
changes between the footer-box update snapshot and offline export/save, before
close and reopen, with no explicit note F9 in that interval. Keep this bounded
automatic lifecycle separate from library leave-alone save behavior.

Use the existing sequence traversal with explicit story ownership. Main-body
and selected anchored text-box fields participate in the captured document
sequence context. Alternate fallback representations must not increment it
again. Related-story repeats read the document context at their reference or
furniture placement. Header, footer, footnote and endnote increments return
the captured `Error! Main Document Only.` without changing that context.
Physical story identity alone is not an independent writable counter.
Implement repeat, explicit restart, heading restart, hidden result and supported
formatting with validated switches. Evaluate an inserted caption through the
same traversal as producer-authored fields. Insertion before an existing
caption changes the later result only when the caller explicitly updates
fields. Save remains a leave-alone operation. Shared header and footer cache
values and per-page rendered repeats are distinct observations.

Producer SEQ instructions can include the documented optional bookmark
operand. Resolve that cross-reference against the checked physical target and
the same sequence snapshot, rather than ignoring the operand or incrementing
again. Caption authoring continues to generate its existing one-operand SEQ.
Fresh controls must discriminate forward and backward references, an identifier
mismatch, missing targets and ambiguous bookmarked ranges before completion.
Until authenticated target semantics are available, explicit cache retention
with a diagnostic is an implementation checkpoint, not complete SEQ support.
Use existing field and range owners, without another public option type or
counter interpreter. The [Microsoft SEQ reference](https://support.microsoft.com/en-us/word/field-codes-seq-sequence-field)
is primary syntax evidence, not a substitute for the pinned Word comparison.

### Shared sequence snapshot

Move sequence interpretation into one concrete, font-independent operation in
the existing rdocx-layout engine. Both facade evaluation and rendering consume
its results. It neither shapes text nor paginates. Reuse SourceRegistry::for_input
and its physical WordSourcePath registration so source IDs stay consistent
with the existing immutable layout result. Do not invent another owner inventory.

```rust
pub struct WordSequenceEvent {
    pub source: oxml_layout::FieldSource,
    pub accepted_run: usize,
    pub identifier: String,
    pub value: i64,
}
pub struct WordSequenceSnapshot {
    // Private source registry projections, field outcomes, ordered events
    // and per-identifier event indexes.
}
pub fn evaluate_sequence_fields(
    input: &LayoutInput,
) -> Result<WordSequenceSnapshot>;
```

The snapshot exposes main_events, source_node, source_id, field_value and
context_value lookups. For selected anchored-box cache updates it also retains
the existing registry field_source_xml and text_box_owner_indices projections,
with hidden source_field_xml(FieldSource) and text_box_owner_index(&WordStory)
lookups matching WordLayoutResult. These bind the same physical owner without
text matching, guessed child indices or a pagination pass. Include the retained
projections in semantic snapshot equality and test selected alternate-content
owners with identical displayed text, nested tables and locked caches.
Nested complex fields may lack an isolated source_replacement even when their
exact enclosing raw field is known. Retain compact namespace-parsed field
semantics from the same SourceRegistry registration traversal: effective
instruction, inherited lock and generated-cache membership. Expose one hidden
read-only WordSequenceSnapshot::source_field_context(FieldSource) getter
returning Option<(&str, bool, bool)> for the concrete facade selected-owner
binder. Include these projections in semantic snapshot equality and public
consumer and verified archive riders. No new field tree clone or source
interpreter is introduced. A hidden Field::effective_instruction_text() -> String
getter in existing text.rs uses the identical raw versus changed-structured
instruction decision as effective_instruction, but formats its borrowed
instruction without cloning nested Field trees. SourceRegistry compact-context
registration and facade selected-child context comparison are its two current
consumers. Include this additive hidden getter in low-level public documentation,
all-target and verified archive checks. Prove raw instruction spelling, changed
structured instructions, nested fields and effective-instruction parity.

The enclosing raw field, physical story, paragraph, preorder ordinal and
matching nested instruction, inherited lock and generated status are jointly
required before binding a child without standalone raw XML. Exact source
identity never comes from displayed text or a global ordinal. Exclude generated
cache children through that same source inventory. Prioritize known inherited
locks and leave locked cache, dirty flags and controls verbatim. Ambiguous or
unbound unlocked children retain a diagnostic instead of advancing a counter
or rewriting a guessed owner. Cover selected nested controls, identical
alternate branches, locked parent fields, zero counter advancement and failure
atomicity in existing entrypoints.

The selected DrawingML projection in existing drawing.rs must preserve
significant text before source binding. parse_alternate_content and the
scoped inline and anchor NsReaders in CT_Drawing::from_xml_with_prefixes
currently trim instruction and literal text while capturing shape contents.
Disable text trimming at these three projection sites, leaving geometry-only
shape-property parsing unchanged. Original raw serializers remain the write
authority. Prove instruction spaces and literal leading, trailing and internal
whitespace in direct and selected alternate-content inline/anchor owners,
unchanged raw producer serialization, locked field preservation and exact
unlocked nested binding. Run applicable drawing/model and existing hash checks,
and classify any intentional render change before accepting it. Do not weaken
the raw-owner comparison to accommodate corrupted projection text.

Each main event stores one successfully applied counter
delta, including hidden increments and selected anchored boxes. Per-identifier
indexes support predecessor lookup at an event boundary. Do not clone the entire
counter map per event or derive chronological order from HashMap iteration.
REF target text can consume the same resolved SEQ segments during one explicit
update, without another increment pass or stale caption-cache lookup.

Hidden events need genuine zero-width structural placement through the existing
neutral FieldKind and substitution machinery. SequenceContext carries a main
event index. A repeat uses its existing FieldSource to identify its resolved
request. Preserve exact text provenance separately and prove hidden-only, table
and control markers create no glyph or extra layout space. Cached replay must
retain or rebind the correct physical source identity. Generated and copied
result caches must not manufacture sequence sources. Preserve ancestor locks.
The public neutral classification changes require exhaustive consumer checks
and verified archive consumers, with documented pre-1.0 enum impact.

Pure evaluation and ordinary update_fields retain furniture repeat caches with
ordered diagnostics when no unique physical context exists. Unique note
references supply their source context. Per-page rendering uses structural
placements and the same event table. An existing explicit layout-backed update
may use an unambiguous supported placement without another pagination pass.
Divergent shared placements retain the stored cache with a diagnostic unless an
operation explicitly selects a context. The measured normal-close value 16 and
reopened value 14 are separate lifecycle evidence, not a universal final-context
cache policy. Library save/reopen does not simulate Word's later recomputation.

Share numeric formatting in the existing layout engine through concrete
format_numeric_field_picture(value, picture) and
format_numeric_field_general(instruction, value) functions returning
Result<String, String>. The facade preserves the established order: numeric
picture, its existing date-time picture branch, then general switches. Shared
SEQ uses the same numeric and general phases, with date-time pictures rejected
by instruction validation. Preserve PAGE-family repeated-letter alphabetic
formatting and existing errors. These additive functions replace divergent
formatters, with facade and sequence rendering as current consumers. Their
public documentation, existing format regressions and archive consumers join
the scoped gate. Date-time parsing stays in its existing owner.

Propagate the immutable snapshot through one hidden public
LayoutInput::sequence_snapshot field of type Option<Arc<WordSequenceSnapshot>>.
Existing constructors initialize it to None. Complete layout entry points
regenerate the snapshot against their current SourceRegistry before shaping or
cache lookup and ignore caller-supplied snapshots. Source-less standalone
paragraph and content measurement retains stored SEQ results and cannot consume
unvalidated snapshot identities. Internal source-qualified shapers consume only
the current derived snapshot. Do not hash the whole model per paragraph.

Retained paragraph, table, furniture and restart work must account for changed
sequence outcomes and structural event identities. Semantic equality or explicit
invalidation, rather than Arc allocation identity, governs reuse. Test a changed
earlier restart with an unchanged later SEQ paragraph, table and header, source
reindexing, stale supplied snapshots and deterministic warm versus fresh output.
The additive pre-1.0 LayoutInput field requires existing struct-literal consumers,
public documentation and verified archive consumers to be updated. No new file,
trait or generic is introduced.

Accepted run insertion retains the existing boundary marker attachment. At an
end boundary the inserted run can remain inside an existing bookmark end. Test
that paired ownership and atomic behavior explicitly. The authenticated
same-paragraph AFTER discriminator has an ordinary literal run after its target
before the REF field. Do not claim an end-boundary insert is outside the target
or change marker attachment to make that test pass.

Share numbered REF selection and validated delimiter formatting through the
existing concrete ResolvedNumbering implementation in
`crates/rdocx-layout/src/style_resolver.rs`. Add the hidden method
`numbered_reference_text(&self, instruction: &FieldInstruction, source: Option<&Self>) -> std::result::Result<String, String>`.
The facade numbered REF evaluator and layout numbered_ref_text are its two
actual consumers. Keep caller-owned position and hyperlink handling separate.
Use the registered numbering context, preserving source delimiters and current
level literal text. Do not replace every full stop or duplicate formatter rules
in the facade. The existing authenticated full-context delimiter control has
`1.1.-Clause 1` before insertion and `2.1.-Clause 1` after insertion. Capture
level, relative, text-omission, empty-context and embedded-ancestor delimiter
controls before generalizing beyond that observation. Unsupported or malformed
operands retain their complete caches with precise diagnostics. Extend existing
API documentation, all-target checks and affected published archive consumers.
No new type, trait, generic or file is introduced.

The additional source-built delimiter matrix is now authenticated under
/private/tmp/S90-F280-ref-delimiter-controls. All 53 instructions and physical
owners, 45 first-materialization outcomes and eight genuine comparison results
are independently audited. Relative/full context inserts the requested hyphen
before the current leaf when a preceding context prefix exists. Level-only,
root and embedded-ancestor controls remain unchanged. Text omission retains the
same context boundary. An empty delimiter inserts a space in the measured
relative/full controls, while level-only remains unchanged. Do not equate an
empty delimiter with an absent switch. A distinct no-F9 reopen preserves all
53 rich caches and exact two-page PDF text and 96dpi pixels. Four package parts
change only in classified producer metadata, so no archive byte-equality is
claimed. Original numbering, styles, target properties and paired bookmark
semantics remain preserved. The prepared target index list was stale after
matrix assembly, and the audit binds the actual physical target positions.
These bounded native results do not replace implementation parity checks.

Map reference choices to REF text, level, relative and full-context switches.
Position, hyperlink and text-omission choices retain their instruction
semantics. Add validated delimiter and referenced-note copying through the
`\f` switch. Its result is typed note-reference content with a corresponding
allocated note, not a number string. Preserve bookmarked literal content and
stage the reference, copied note and its relationship closure in the same
atomic package mutation. Rich runs, bookmark identities, hyperlink and image
relationships require captured copy evidence. The measured note-context
restriction retains literal content without allocating another note.
Repeated explicit materialization must not accumulate orphan copied notes.
The authenticated plain repeat retains four copied references and exactly six
referenced normal owners. Changing only the original source note text refreshes
all four copied payloads while retaining those owner IDs and all 31 field
contracts. The rich inherited-Emphasis repeat preserves media bytes, note
relationships, style definitions and six referenced owners without package
or relationship growth. Copied drawing identities regenerate and remain
unique, so do not claim byte-identical copied drawings. These controls are
retained in /private/tmp/S90-F280-ref-owner-controls.

Reuse a cached typed note owner only when the physical REF-f field exclusively
owns its reference, it is distinct from the source target owner, and the
candidate graph passes ownership validation. Refresh the payload from the
frozen source inventory after original field traversal. Retain diagnostics for
ambiguous or shared ownership, and never delete unowned note or relationship
content. Extend the existing private append operation with a checked replacement
path for this concrete materialization consumer, preserving ordinary fragment
import append behavior. Plain and rich repeat tests must show stable referenced
owner counts, refreshed source content, no orphan or dangling references,
preserved original source ownership and bounded package relationship growth.
All failures preserve complete receiver bytes. Saved ID stability does not
establish Word's internal reuse mechanism or authorize general cleanup.

Repeated annotation materialization has its own measured contract. The genuine
second body update retains three comment owners and all seven field contracts
without added parts or relationships. Changing only original comment 1's literal
refreshes copied comment 2 from that current source while retaining context
comment 3. The copied paragraph and durable identities regenerate coherently
during update and save. Preserve the closed, unique commentsExtended and
commentsIds ownership graph rather than requiring those copied identities to
remain fixed. These controls are retained in the existing note-annotation
probe directory as annotation-repeat-stage-audit and
annotation-source-mutated-stage-audit. The marker-only target remains absent
from earlier native normalization, so its missing-target errors do not establish
an unsupported copy operation.

Reuse or replace a cached annotation definition only when the checked physical
REF-f cache exclusively owns its paired markers and typed reference, distinct
from the original target and context owners. Refresh its payload from the frozen
original source inventory and stage its paragraph and durable graph closure
atomically. Shared, ambiguous or broken graphs retain diagnostics and complete
receiver bytes. Do not delete unrelated comments or borrow note-copy rules.
Tests must cover repeated update and source-only refresh through save and reopen,
bounded owners and relationship counts, preserved original and context payloads,
unique graph identities and failure atomicity. Captured ID stability does not
identify Word's internal allocation mechanism.

Reject mutually exclusive numbering modes and malformed switch operands.

Resolve bookmark position using physical story identity, accepted paragraph
order and accepted run boundaries. This fixes references before and after a
target within the same paragraph. Do not infer above or below across unrelated
stories. A cross-story reference can resolve uniquely owned target text and
number context, while an unavailable relative position retains its cache and
reports a diagnostic.

An explicitly requested REF hyperlink remains a field and produces its
internal target link during cache materialization and rendering. Its creation
does not flatten the field into a standalone hyperlink that would stop
updating. Missing, duplicate or unsupported targets retain saved display
content and diagnostics under the existing evaluation policy.

Use F-248's existing resolved numbering and accepted projection. F-283 owns
the final cross-consumer numbering reconciliation. F-280 must not create a
second counter engine or repurpose pagination as an independent evaluator.

## Rejected alternatives

- Construct caption XML through raw string concatenation. It bypasses F-278's instruction validation and checked story ownership.
- Persist a caption registry beside the package. The authoritative state already consists of sequence fields and bookmark markers.
- Resolve same-paragraph position from paragraph ordinal alone. It cannot distinguish a preceding target from a following target.
- Flatten hyperlink references to plain text. It loses the dynamic REF instruction.
- Add a captions module. Existing field and range owners already provide the necessary boundaries.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `captions_and_references_match_pinned_word_before_and_after_renumbering` | Figure, table, equation and numbered-heading references match fresh Word semantic records before and after insertion and explicit update. |
| integration | `caption_targets_cover_text_label_and_number_in_each_story` | Three unique bookmarks have the correct ranges in body, table cells, headers, footers, notes and text boxes. |
| unit | `sequence_options_validate_and_preserve_switch_semantics` | Increment, repeat, restart, heading restart, hidden result, formatting and malformed combinations behave as specified. |
| regression | `ref_position_distinguishes_same_paragraph_run_boundaries` | Before-target and after-target references return the pinned Word position values. |
| regression | `ref_switches_preserve_number_context_delimiters_and_links` | Numbering modes, delimiter, text omission, position and hyperlink semantics compose without losing the field. |
| differential | `sequence_story_context_matches_pinned_word` | Main and selected text-box sequence traversal avoids fallback duplication, related increments report the captured error, and note or furniture repeats resolve the applicable document context. |
| differential | `ref_note_copy_matches_pinned_word_typed_content` | Body references allocate typed note references and copied notes atomically, rich content and relationship targets match captured source-note semantics, and note-context references do not allocate recursive copies. |
| round-trip | `caption_reference_round_trip_preserves_producer_xml` | Instruction shape, cached run order, styles, markers, unrelated XML and relationships survive save and reopen. |
| regression | `sequence_snapshot_reuses_source_identity_and_linear_events` | Shared source registration, hidden deltas, selected branches and indexed contexts remain consistent without quadratic retained state. |
| regression | `hidden_sequence_markers_do_not_paint_or_shift_layout` | Body, table and control event placement retains source identity without extra glyphs or layout space. |
| regression | `caption_and_reference_failure_is_atomic` | Invalid names, stale locations, ambiguous targets and malformed fields leave the receiver and package unchanged. |

**Test gate**: differential. Figure, table, equation, and numbered-heading
references match Word before and after insertion and renumbering.

Use the existing regression_test.rs and in-module unit tests. Do not add an
integration-test binary.

Pin fresh captures to Microsoft Word 16.113.2 build 16.113.26092012. Existing
Word 16.112.3 evidence at regression_test.rs:9512 and :26828 remains evidence
for its original tests, not for this story. Extend the existing live-capture
pattern at regression_test.rs:26876. Construct sanitized inputs in Rust,
capture the same source before and after insertion and renumbering, update all
physical stories in Word, save and inspect field instructions, caches,
bookmark targets and hyperlink destinations. Record exact semantic records
in the existing test file only after capture. Keep generated DOCX and PDF
files ignored. An unavailable live capture is a failed or incomplete
differential gate, not a reason to label generated expectations as Word data.

## Captured contract clarification

The fresh sequence controls retain all 33 authored fields. Body increments,
related-story errors, note repeats and the selected text-box values are pinned
in `F280-sequence-controls-snapshot-audit.json`. Shared furniture renders 14
on page one and 16 on page two, while its saved cache changes across the
observed save lifecycle. Do not replace per-page expectations with one cache.

The seven-field footnote control pins typed copied references and corresponding
note content in `F280-ref-footnote-authenticated-20261007-01.json`. Marker-only
results can have empty text and a nonempty typed reference. The authenticated inherited-Emphasis rich copy probe pins retained style
references and copied note payload/media, with new drawing identities, removed
copied internal bookmarks and flattened external hyperlink wrappers. Keep the
original source note intact. This is bounded evidence for the captured payload,
not permission to infer arbitrary annotation or relationship copying.

Five caption and REF-f saved-output reopens retain all 134 field contracts and
per-page PDF text. The three REF-f controls retain their rich payloads. In both
caption cases Word removes explicit font properties from two heading REF
caches, which then inherit the producer defaults. That is an observed
formatting change, not strict or effective formatting equality. Classify this
native lifecycle separately from library save/reopen preservation. The authenticated endnote control retains all seven instructions and four
typed copied owners, with note-context literal results and no recursive copies.
Its distinct no-F9 reopen retains rich caches and identical one-page PDF text
and pixels. The authenticated annotation control copies the WHOLE comment
range and owner in body context, but retains literal content in the annotation
context without allocating another owner. Word removes its marker-only target
bookmark during the body update, so those missing-target errors do not establish
unsupported annotation copying. Its reopened rich caches remain exact apart
from recorded editing metadata, while the PDF retains text with a qualified
54-pixel anchor-outline difference. The original source, stages and audits are
retained in /private/tmp/S90-F280-note-annotation-controls. These are bounded
native captures, not implementation parity or a completed differential gate.

These clarifications follow the [Microsoft SEQ reference](https://support.microsoft.com/en-us/word/field-codes-seq-sequence-field)
and [Microsoft REF reference](https://support.microsoft.com/en-us/word/field-codes-ref-field).
They replace the earlier independent-story counter and number-string reading.
They do not reduce the approved caption or cross-reference scope.

## Selected header and footer text-box painting

The full authenticated REF-f owner fixture requires selected header/footer
text-box literal results to paint, along with ordinary furniture and body boxes.
The existing furniture layout retains anchored content, but
`crates/rdocx-layout/src/paginator.rs` render_hf_blocks currently emits only
paragraph lines and change bars. Complete that existing path using its concrete
anchor placement and shape-text routines. Pass content-relative paragraph y,
which is start_y minus margin_top plus story flow offset. The existing vertical
resolver already adds the page margin for paragraph-relative anchors. Do not
apply that margin twice or borrow body wrapping state for furniture.

Cover first, default and even furniture variants, page/margin/paragraph-relative
positions, indentation and overlapping neighboring paragraphs. Behind-document
anchors belong to the existing page layer before text, retaining page-border
and watermark background order. Front anchors follow their story text without
changing unrelated body foreground or front-border policy. Preserve source
provenance, shape geometry and producer XML on pure layout and save/reopen.
No new renderer, helper API, type or file is proposed. Deterministic rendering
and the existing hash harness gate this intentional appearance correction.
If a corpus fixture contains previously omitted furniture boxes, identify its
exact expected newly painted content and review that delta in its own labelled
commit before changing a baseline. No unexplained delta is allowed.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Public API of a published crate. Read native facade stability and structural rules. Changes are additive pre-1.0 APIs. Run cargo publish --dry-run for affected published crates and the existing .crate size assertion.
- Any parser or serialiser. Read OPC and packaging and PresentationML model sequencing guidance. Test prefix-tolerant reading, ordered field serialization and byte-preserved unmodelled subtrees.
- Layout, pagination, line breaking, text shaping. REF display and links can affect layout. Run deterministic font mode checks and compare the relevant rendered reference cases.
- External oracle comparison. Read differential-testing.md. Assert the exact Word version and build before fresh capture, retain provenance and triage every disagreement.
- No new crate, module or production file is proposed. Required design and review records have consolidated user permission.

## Hash harness

Expected unchanged for the existing corpus. New public-only fixtures do not
change existing inputs. Existing output that exercises a corrected REF switch
or same-paragraph position may change intentionally. Such a delta must be
identified by exact fixture and field, reviewed against captured Word
behavior, and committed separately with a labelled expected delta. Do not
record an unexplained baseline change.

## Implementation checklist

- [x] Complete F-278 and confirm F-248 and F-275 remain done.
- [x] Add checked option types and existing-module exports.
- [x] Implement atomic checked story insertion for sequence and REF fields.
- [x] Compose caption paragraphs and three uniquely allocated bookmark targets.
- [x] Extend REF evaluation and rendering without flattening dynamic fields.
- [x] Cover same-paragraph position and physical story ownership.
- [x] Capture and pin fresh Word semantic evidence.
- [x] Run focused tests, risk riders and scoped verification.
- [x] Obtain a zero-defect, zero-smell microscope review.
- [x] Prepare the structured worker handoff.

## Open questions

None. The user approved all six stories' workflow records and the dedicated
bibliography module. Native APIs are additive, existing outline APIs retain
their behavior, and fresh Word evidence is pinned to 16.113.2 build
16.113.26092012. INDEX and TOA use en-US source language with the captured Word
locale recorded explicitly. Technical ownership probes in the test plan must
pass before completion and may not be replaced by guessed expectations.

## Capability ownership verification rider

Update the existing `scripts/test_sprint_workflow.py` expected capability owner
set when DOCX-047 is verified complete and its HLD owner becomes `-`. The
existing F-278 and F-279 exclusions establish the same boundary. Exclude F-280
from that expected incomplete-owner set and retain the matrix classification
and live-owner assertions. Do not change worker sprint ledgers to satisfy this
test. Run the existing workflow tests against the completed HLD contract.

## Exclusive resources

- `crates/rdocx/src/field.rs`
- `crates/rdocx-oxml/src/text.rs`, checked accepted-view run insertion using the existing range-site resolver
- `crates/rdocx-oxml/src/drawing.rs`, preserve significant selected-owner text in existing projection readers
- `crates/rdocx/src/lib.rs`
- `crates/rdocx/tests/regression_test.rs`
- `crates/rdocx/src/document.rs`, if checked story insertion needs an existing-owner helper
- `crates/rdocx/src/comments.rs`, if paired-target publication needs an existing-owner helper
- `crates/rdocx-layout/src/style_resolver.rs`, shared concrete numbered REF formatting for facade and layout consumers
- `crates/rdocx-layout/src/paginator.rs`, selected header/footer anchored text-box painting and existing page-layer ordering
- `crates/rdocx-layout/src/engine.rs`, shared sequence event evaluation and per-page reference display
- `crates/rdocx-layout/src/lib.rs`, concrete sequence snapshot and event records
- `crates/rdocx-layout/src/input.rs`, immutable derived snapshot propagation
- Existing LayoutInput constructors, only for initialization of the additive field
- `crates/oxml-layout/src/output.rs`, neutral structural context and repeat classifications
- Existing neutral and render consumers, only for required exhaustive classification or structural-marker propagation
- `scripts/test_sprint_workflow.py`, expected incomplete capability owner reconciliation
- The named HLD sections in the impact list
- Hash baseline only if an individually reviewed intentional delta is proven

F-280 cannot share a wave with F-278, F-279, F-281, F-282 or F-283 when their
claims overlap these files.

## Accepted projection registration clarification

The font-independent `evaluate_sequence_fields` requires
`RevisionView::Accepted` and rejects Tracked input. Rendering other views retains
producer SEQ caches with ordered diagnostics and emits no sequence event or
repeat marker. Facade field evaluation uses its accepted source projection.
The shared physical inventory registers revision-owned source fields as well
as direct, control and hyperlink fields, preserving unique stable source
identities. Registry, sequence collection and facade evaluation consume that
same inventory. Accepted insertion rules remain separate from evaluation.

TOC source discovery consumes the same accepted sequence snapshot through the
existing evaluator, including accepted revision and inline-control fields. A
private concrete `Arc` may share this snapshot between the ordinary evaluator
and TOC discovery, its two actual consumers. Never restore an independent SEQ
interpreter or join fields by displayed instruction text. Test deleted-before-
visible physical inventory, accepted revision/control/hyperlink TOC prefixes,
inherited locks and exclusion of generated cache fields. Correct any earlier
unit that pinned revision-source omission to the actual registered inventory.

The existing `text.rs` may add hidden concrete
`CT_P::source_runs(&self) -> Vec<&CT_R>` for physical identity registration.
Traverse existing boundary owners and typed direct, control, hyperlink and
revision runs in exact physical order. Include deleted revision runs for
identity, while accepted selection remains caller-owned. Preserve existing
`CT_P::runs` behavior for unrelated callers. The two current consumers are
layout source registration and collection, and facade evaluation and raw
updating. Reuse this borrowed inventory rather than duplicating traversal.
Document the additive hidden getter and cover all-target consumers, verified
archives, physical order, deleted-before-visible registration and unchanged XML.

## README archive measurement rider

Reconcile the four affected archive measurements in existing
`scripts/readme_doctests.py`, `README.md`,
`crates/oxml-layout/README.md`, `crates/rdocx-layout/README.md` and
`crates/rdocx-oxml/README.md` against actual current archives. Root README is
the facade crate README. Record measured compressed bytes, normalized member
bytes and member counts using the existing command and preserve all other
measurement rows and evidence conventions. Rerun README verification after
recording, accounting for changed README bytes in the archives.

The README measurement script uses archive-only
`cargo package --locked --allow-dirty --no-verify` at its existing
`target/package` location. This archive-only measurement route is permitted.
It is not verified packaging evidence. Compilation, tests, docs, lint and
verified publish dry runs retain the healthy target directory. Complete the
four affected published archive verification dry runs with local patches and
the ten MiB assertions separately, without substituting the measurement route.

## Microscope malformed quoting rider

Remediate the independently cited malformed producer SEQ boundary through a
hidden concrete `FieldInstruction::quotes_are_balanced(&self) -> bool` in the
existing `crates/rdocx-oxml/src/text.rs`. Move the existing escaped quote and
backslash algorithm from facade `instruction_quotes_are_balanced` unchanged.
Read the preserved raw instruction, without reparsing, normalization or mutation.
The facade instruction-shape validator and shared sequence snapshot collector
are two existing consumers. Validate before any snapshot event or counter
mutation and before facade or renderer consumption. Retain the producer cache
with the established unclosed-quoting diagnostic on failure.

This is an additive hidden pre-1.0 XML API, with no new file, module, dependency,
trait or wrapper. Document it and include all-target consumers, affected archive
verification, README archive measurements and the ten MiB gate. Reuse the
approved HLD impact list. Test an unclosed quoted SEQ followed by a valid
increment through pure evaluation, update, saved reopen and actual deterministic
painting. The malformed event must not advance the counter. Include escaped
quotes and backslashes and keep the genuinely measured optional bookmark
operand valid. Do not restore the old one-argument SEQ shape restriction.

## Qualified bookmark numbering projection rider

The existing `WordLayoutResult` may expose hidden concrete
`bookmark_numbering(&self, name: &str) -> Option<&ResolvedNumbering>` in the
existing `crates/rdocx-layout/src/lib.rs`. Project existing resolved bookmark
numbering from the single `NumberingState`, with exact registered source IDs
and the same separated physical story inventory. Admit only a uniquely paired
accepted physical bookmark context. Duplicate names, malformed ranges and
unavailable contexts yield no projection. Preserve established flattened main
story entries, including nested tables and controls. Do not reconstruct a
source path from facade owner indexes or introduce another numbering engine.

The actual facade REF consumer and existing renderer use the same resolved
numbering state. This derived result projection replaces the newly attempted
facade location reconstruction, with no new file, module, dependency, trait or
wrapper. It is an additive hidden pre-1.0 layout API. Document the getter,
reconcile the approved HLD impact files and cover all-target consumers,
warnings-denied docs, affected verified archives, actual README archive
measurements and the ten MiB gate. Test numbered related nested-table targets,
selected text boxes with no SEQ, controls, uniquely paired ordinary related
text, duplicate physical name refusal and unchanged main-table numbering.
Cross-story relative position remains unavailable with complete cache retention
and a diagnostic, including combined number and position requests.

## Shared physical relative-position projection rider

The existing `WordSequenceSnapshot` may expose hidden concrete
`bookmark_relative_position(&self, target: &str, source: FieldSource)` returning
`Result<&'static str, String>`. Reuse the same accepted physical collector to
retain exact field run boundaries and uniquely paired bookmark endpoints.
Compare accepted owner-local paragraph order and run boundaries only within
the same physical story owner. Normalize related header/footer relationship
aliases to their actual part and preserve note ID and selected text-box owner
identity. Do not reconstruct source paths from facade indexes, transient
paragraph addresses or displayed instruction text.

The facade and renderer are two existing consumers of this projection. Replace
their main-only position special cases with this one qualified comparison.
Before-target, after-target and same-paragraph cases use exact boundaries.
Containment, duplicate or malformed target ranges, generated cache membership,
missing bindings and cross-owner requests retain the complete saved cache
with the established diagnostic. Validate inherited locks before resolving
position. Add the private position bindings to snapshot semantic equality and
retain source-only registration for REF documents with no SEQ events.

This is an additive hidden pre-1.0 layout API in the existing approved files,
with no new module, file, dependency, trait or interpreter. Include all-target
consumers, warnings-denied docs, affected verified archives, README archive
measurements and the ten MiB gate, using the approved HLD impact list. Capture
bounded native same-owner header, footer, footnote, endnote and selected
text-box controls without deriving expectations from the implementation.
Preserve raw cache and provenance when native updates do not reach an owner.
Do not infer position from an unchanged OLD sentinel. The resulting readable
regressions must prove pure evaluation, update, saved reopen and deterministic
painting, while retaining cross-owner fallback and ordinary discovery policy.

The bounded same-owner native capture pins36 fields across body, header, footer,
normal footnote, normal endnote and selected body text box. Exact instructions,
12 paired selected bookmarks and all literal targets survive nine stages. Each
owner-specific update changes only its six caches. Successful paragraph and run
positions resolve below before the target and above after it. Containment
produces Word's self-reference error, while this implementation deliberately
retains the saved cache with a diagnostic under the conservative policy above.
The prepared direct paragraph drawing is invalid and is not admitted by the
parser. Tests use a cache-only derivative of the genuine schema-valid native
normalized package, with its selected Choice and opaque Fallback preserved
separately. Actual close and no-F9 reopen preserve36 rich caches. Equal page text
and the measured439-pixel native PDF difference do not imply pixel parity.
