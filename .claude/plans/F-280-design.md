# F-280, Captions, sequences, and complete cross-references

**Status**: approved
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

Map reference choices to REF text, level, relative and full-context switches.
Position, hyperlink and text-omission choices retain their instruction
semantics. Add validated delimiter and referenced-note copying through the
`\f` switch. Its result is typed note-reference content with a corresponding
allocated note, not a number string. Preserve bookmarked literal content and
stage the reference, copied note and its relationship closure in the same
atomic package mutation. Rich runs, bookmark identities, hyperlink and image
relationships require captured copy evidence. The measured note-context
restriction retains literal content without allocating another note.
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
native lifecycle separately from library save/reopen preservation. Endnote
and annotation REF-f probes are prepared but remain uncaptured and must not
be presented as passing oracle evidence.

These clarifications follow the [Microsoft SEQ reference](https://support.microsoft.com/en-us/word/field-codes-seq-sequence-field)
and [Microsoft REF reference](https://support.microsoft.com/en-us/word/field-codes-ref-field).
They replace the earlier independent-story counter and number-string reading.
They do not reduce the approved caption or cross-reference scope.

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

- [ ] Complete F-278 and confirm F-248 and F-275 remain done.
- [ ] Add checked option types and existing-module exports.
- [ ] Implement atomic checked story insertion for sequence and REF fields.
- [ ] Compose caption paragraphs and three uniquely allocated bookmark targets.
- [ ] Extend REF evaluation and rendering without flattening dynamic fields.
- [ ] Cover same-paragraph position and physical story ownership.
- [ ] Capture and pin fresh Word semantic evidence.
- [ ] Run focused tests, risk riders and scoped verification.
- [ ] Obtain a zero-defect, zero-smell microscope review.
- [ ] Prepare the structured worker handoff.

## Open questions

None. The user approved all six stories' workflow records and the dedicated
bibliography module. Native APIs are additive, existing outline APIs retain
their behavior, and fresh Word evidence is pinned to 16.113.2 build
16.113.26092012. INDEX and TOA use en-US source language with the captured Word
locale recorded explicitly. Technical ownership probes in the test plan must
pass before completion and may not be replaced by guessed expectations.

## Exclusive resources

- `crates/rdocx/src/field.rs`
- `crates/rdocx/src/lib.rs`
- `crates/rdocx/tests/regression_test.rs`
- `crates/rdocx/src/document.rs`, if checked story insertion needs an existing-owner helper
- `crates/rdocx/src/comments.rs`, if paired-target publication needs an existing-owner helper
- `crates/rdocx-layout/src/engine.rs`, if per-page reference display needs the existing layout owner
- The named HLD sections in the impact list
- Hash baseline only if an individually reviewed intentional delta is proven

F-280 cannot share a wave with F-278, F-279, F-281, F-282 or F-283 when their
claims overlap these files.
