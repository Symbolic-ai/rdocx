# F-281, Indexes and tables of figures and authorities

**Status**: completed
**Sprint**: S90
**Size**: L
**Depends on**: F-278 through F-280

## Problem

The facade can rebuild an existing TOC, but it does not author index or
authority source markers and has no generated INDEX or TOA rebuild boundary.
`crates/rdocx/src/field.rs:1023` scans dynamic TOC spans specifically, and
`crates/rdocx/src/field.rs:649` exposes TOC and TC outcomes without corresponding
index or authority result structures.

The existing TOC source model carries a sequence identifier at
`crates/rdocx/src/field.rs:665`, but this selects a prefix for a page value.
It is not a complete table-of-figures caption source selector. The current
facade insertion at `crates/rdocx/src/document.rs:24420` is heading-oriented.
INDEX, caption-selected TOC and TOA need source ownership, ordered entries,
range targets and cache formatting that retain their dynamic fields.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-281, Indexes and tables of figures and authorities".
- `docs/hld/02-scope-and-non-goals.md`, capability DOCX-048.
- `docs/hld/03-architecture.md`, "What stays put", recursive field grammar, TOC owned-span rebuilding, source discovery and atomic package publication.
- `docs/hld/03-architecture.md`, "Facade conventions", checked story ownership and accepted content traversal.
- `docs/hld/08-rendering-spec.md`, "Word bookmark field pagination", generated entry targets, provisional PAGEREF fields and deterministic cache materialization.
- `docs/hld/04-opc-and-packaging.md`, XML preservation and relationship ownership.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The private from-scratch DOCX conformance corpus".

## Approach

Add concrete source-marker and table option types in the existing field module.
Use existing TabLeader and paragraph/run property types.

Extend the existing shared instruction grammar in
`crates/rdocx-oxml/src/text.rs` for measured generated-table operands.
INDEX accepts z, f, h, e, l, g, k and s operands, XE accepts f, r and t,
TA accepts l, s, c and r, TOA accepts c, e, l and g, and caption-selected
TOC accepts c and a. INDEX r and TOA h are flags. Preserve each other opcode's
operand and flag policy, exact original instructions and unknown extensions.
Bare and quoted operands must resolve through this existing grammar rather
than a second field interpreter. Focused parser tests must distinguish bare
numeric categories and LCIDs, caption labels, quoted separators and flag versus
operand collisions. Include rdocx-oxml in scoped test, lint, documentation and
publish evidence. This is necessary source discovery for the approved APIs.

```rust
pub struct IndexEntry {
    pub levels: Vec<String>,
    pub identifier: Option<String>,
    pub page_range_bookmark: Option<String>,
    pub cross_reference: Option<String>,
    pub bold_page_numbers: bool,
    pub italic_page_numbers: bool,
}

pub struct AuthorityEntry {
    pub long_citation: String,
    pub short_citation: String,
    pub category: u8,
    pub page_range_bookmark: Option<String>,
    pub bold_page_numbers: bool,
    pub italic_page_numbers: bool,
}

pub struct IndexOptions {
    pub identifier: Option<String>,
    pub heading_separator: Option<String>,
    pub entry_page_separator: String,
    pub page_separator: String,
    pub range_separator: String,
    pub run_in: bool,
    pub leader: TabLeader,
    pub hyperlink: bool,
}

pub struct TableOfFiguresOptions {
    pub label: String,
    pub include_label_and_number: bool,
    pub hyperlink: bool,
    pub leader: TabLeader,
    pub entry_page_separator: String,
}

pub struct TableOfAuthoritiesOptions {
    pub category: Option<u8>,
    pub include_category_headings: bool,
    pub use_passim: bool,
    pub entry_page_separator: String,
    pub page_separator: String,
    pub range_separator: String,
    pub leader: TabLeader,
}

pub struct GeneratedTablesReport {
    pub index_entries: usize,
    pub figure_entries: usize,
    pub authority_entries: usize,
    pub bookmark_count: usize,
    pub diagnostics: Vec<String>,
}

impl Document {
    pub fn insert_index_entry(
        &mut self,
        position: &StoryRunPosition,
        entry: &IndexEntry,
    ) -> Result<()>;

    pub fn insert_authority_entry(
        &mut self,
        position: &StoryRunPosition,
        entry: &AuthorityEntry,
    ) -> Result<()>;

    pub fn insert_index(
        &mut self,
        before: &ContentLocation,
        options: &IndexOptions,
    ) -> Result<ContentLocation>;

    pub fn insert_table_of_figures(
        &mut self,
        before: &ContentLocation,
        options: &TableOfFiguresOptions,
    ) -> Result<ContentLocation>;

    pub fn insert_table_of_authorities(
        &mut self,
        before: &ContentLocation,
        options: &TableOfAuthoritiesOptions,
    ) -> Result<ContentLocation>;

    pub fn rebuild_generated_tables(
        &mut self,
    ) -> Result<GeneratedTablesReport>;
}
```

Source insertion writes XE or TA fields through F-278's typed recursive
instruction constructor and F-280's checked story insertion helper. Keep
marker fields nonprinting while retaining their cached and raw producer
content. Encode hierarchy separators and quoted operands through the grammar.
Reject empty hierarchy components, invalid categories, conflicting
cross-reference and page-range requests and invalid XML before publication.

Table-of-figures authoring uses Word's caption-selected TOC instruction rather
than an invented TOF opcode. Implement its caption selection independently
from the existing TOC sequence page-prefix switch. Discover F-280 captions
from sequence fields and accepted paragraph content, including equivalent
producer-authored captions. Do not require a private caption registry.

Author dynamic INDEX and TOA fields with owned result boundaries. Existing
producer instructions and supported switches use the same rebuild boundary.
Extend the existing owned-span scanner and replacement machinery locally.
Each result owns only its separator-to-end span, excluding old entries from
source discovery. A malformed span, ambiguous owner or failed required target
aborts the complete operation without partial publication.

Build index hierarchy and authority groups from accepted-view source markers
in physical story order. Normalize duplicates and page lists according to
fresh Word captures. Sort with an explicitly declared locale policy rather
than Rust string ordering masquerading as Word collation. Preserve
case-sensitive display text and source formatting separately from sort keys.

Support repeated pages, range bookmark endpoints, cross-reference entries,
nested index levels, category grouping and captured passim behavior.

INDEX and authority ordering use the bounded captured en-US ASCII key policy.
Case-sensitive display entries remain distinct, while ordering folds ASCII
letters and places a distinct lowercase entry before its uppercase counterpart.
Non-ASCII sort keys retain the complete original generated owner and cache
with a stable diagnostic, including when another supported table rebuilds.
Caption text is not subject to this collation boundary. Tests cover both an
isolated refusal and mixed supported and retained owners. This boundary does
not claim general Unicode or other-locale collation.

Authority authoring follows the authenticated native producer. A numbered
category authors one TOA field. `category: None` authors separate numbered
fields for the populated categories in native category order and returns the
first inserted location. Stage that complete insertion atomically. Do not
encode All as category zero or omit its category operand. Native All captured
categories 1, 2 and 5 as three fields, each with heading and passim switches.
The category-heading option maps to `\h`. Short citations join occurrences
to their long entry within a category. They are grouping keys, rather than a
display-selection switch. The native dialog exposes no short-display or
hyperlink option. Do not invent either switch or interpret TOA `\h` as a
hyperlink. Preserve unsupported producer switches with diagnostics under the
existing result-retention policy.

Exact source, native dialog procedure, instructions and two-page results are
recorded in the authenticated category-control and complete native All records
under `/private/tmp/S90-Word-captures`. The original omitted-category and
category-zero controls both return `Error! Category number not found.`. The
complete All capture preserves all six TA markers and emits three numbered
TOA fields. Source topology and instruction audits passed independently.

Generated entries retain appropriate INDEX, Table of Figures and Table of
Authorities styles, requested leaders, paragraph properties and source run
formatting. Preserve producer style definitions and unrelated content.
Hyperlinked entries point to checked internal bookmark targets. Unsupported
producer formatting or instructions retain their owned cached result and
report a stable diagnostic rather than replacing it with an incomplete table.

Reuse an existing valid source bookmark or allocate one through the
document-wide identifier owner. Range entries correlate both endpoints and
use the first and last displayed page values from the same target map.
Repeated header/footer ownership is handled under F-279's explicit physical
field and target placement contract. Do not invent a page number for an
unplaced source.

Rebuild all requested tables in one staged candidate. Discover and generate
provisional result structures first, then obtain one immutable deterministic
WordLayoutResult from F-279 after the provisional structures exist. Resolve
every page and range endpoint from that snapshot. Patch caches and reopen
before atomically publishing. This avoids measuring the document before new
tables change pagination. Preserve the existing documented single-snapshot
pagination policy. Do not create a second paginator or repeatedly estimate
page targets until values appear stable.

Keep Document::rebuild_toc as its existing compatibility operation. Its current
behavior and report remain intact. Internal shared ownership and formatting
helpers must have concrete callers in both the TOC and new generated-table
paths. No forwarding-only abstraction, new trait or generic is justified.

## Rejected alternatives

- Treat every generated table as heading TOC selection. XE hierarchy, authority grouping and caption sequence sources have different semantics.
- Use a TOF field opcode. Word tables of figures use TOC instructions with caption selection.
- Estimate source pages before inserting table caches. Generated results change pagination.
- Generate plain paragraphs outside a dynamic field. Word could not refresh them after source mutation.
- Sort every source lexically with Rust Ord. It would silently impose a different locale contract.
- Create a generated-tables module or a new integration-test binary. Existing field ownership and test entrypoints suffice.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `generated_tables_match_pinned_word_after_source_mutation` | INDEX, caption-selected TOC and TOA entries, ordering, duplicates, categories and page targets match fresh Word updates before and after source changes. |
| differential | `index_hierarchy_ranges_and_cross_references_match_word` | Hierarchies, same-page duplicates, page ranges, cross-reference text and page formatting match the chosen Word locale. |
| differential | `authority_categories_short_citations_and_passim_match_word` | Authority grouping, citation selection, repeat-page handling and passim follow captured Word behavior. |
| integration | `table_of_figures_uses_caption_sequence_sources` | Figure, Table, Equation and custom-label selection reads F-280 and producer captions rather than heading-only TOC sources. |
| round-trip | `generated_tables_preserve_instruction_and_result_formatting` | Dynamic fields, ordered caches, leaders, hyperlinks, styles and unrelated XML survive save and reopen. |
| regression | `generated_table_targets_use_one_post_insertion_snapshot` | All page and range values come from the same deterministic candidate layout after provisional result insertion. |
| regression | `generated_table_source_scan_excludes_old_result_ranges` | Repeated rebuild does not index previously generated entries or accumulate duplicate bookmarks. |
| regression | `generated_table_rebuild_rejects_ambiguous_ranges_atomically` | Missing or duplicate markers, malformed spans and required unplaced targets leave the original package unchanged. |

**Test gate**: differential. Mutating sources produces the same ordered entries
and page targets as the pinned Word update.

Extend existing regression_test.rs with source-built synthetic documents and
live capture tests. Pin Microsoft Word 16.113.2 build 16.113.26092012. Record
Word locale, document language, field update action and exact saved semantic
records. Capture both initial and mutated documents, update all requested
fields in every relevant story, save, then extract ordered field caches,
entry text, page/range values, category headings and hyperlink anchors.
Store sanitized semantic records in the existing test source after capture.
Keep binary artifacts ignored.

Use bundled/document-supplied fonts for Rust layout. If raster evidence is
included, pin Poppler 26.01.0 and 150 DPI, require no more than one pixel of
paired page-size difference and at least 0.95 SSIM for each compared page.
The semantic comparison remains exact. LibreOffice 26.2.5.2 is supplementary
repair/render evidence and does not substitute for the Word update gate.

Fresh initial and mutated INDEX, authority and figure-table captures are
retained under /private/tmp/S90-F281-oracle and /private/tmp/S90-Word-captures,
with their original source, action and saved-output provenance. Corrected
complex XE and TA inputs retain source topology. The original full simple-marker
inputs changed topology on native import and are excluded from this gate.
This does not imply every simple marker is invalid. Native authoring uses the
measured complex form, while reading preserves existing producer forms.

Actual no-F9 reopens for both corrected INDEX cases and initial authorities
retain exact rich XML and per-page PDF text and raster pixels. Native All and
category controls also have authenticated reopen audits. Mutated authorities
retain byte-identical saved DOCX and eight-page PDF text and raster pixels.
Both figure-table reopens retain all 27 and 28 instructions, rich result content,
source bookmark ranges, relationships and media, with identical five-page PDF
text and raster pixels. Recorded producer rsid and textId metadata, proofing
state, zoom, timestamps and statistics change. The complete 15-case audit queue
is retained in /private/tmp/S90-F280-F281-F283-reopen/manifest.json. Native capture
records are bounded evidence, rather than a completed Rust differential comparison.
Missing capture, unavailable locale or a failed comparison blocks completion.
Do not manufacture Word expectations from the implementation output.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Public API of a published crate. Read native facade stability and structural rules. State additive pre-1.0 API impact. Run affected cargo publish --dry-run and the existing .crate size assertion.
- Any parser or serialiser. Read packaging and schema sequence guidance. Prove prefix tolerance, ordered complex-field output and byte-preserved unrelated XML.
- Layout and pagination. Use deterministic font mode. Validate post-insertion targets against one retained immutable snapshot and declare any intentional output delta.
- External oracle comparison. Read differential-testing.md. Assert exact Word version/build and locale, record fresh provenance and triage each disagreement.
- No new crate, module or production file is proposed. Required workflow records have consolidated permission.

Feature completion updates the existing incomplete-capability owner assertion in scripts/test_sprint_workflow.py to exclude completed F-281. The package rider remeasures the affected rdocx and rdocx-oxml archives, updates their existing footprint rows in README.md and crates/rdocx-oxml/README.md, and updates only the corresponding archive tuples and dates in scripts/readme_doctests.py. Existing performance observations, thresholds and all other package measurements remain unchanged.

## Hash harness

Expected unchanged for existing corpus entries. Newly supported INDEX, TOA or
caption-selected TOC may intentionally change fixtures that already contain
these instructions. Identify each exact affected fixture and changed field,
capture supporting Word behavior, and commit any intentional behavior delta
separately. Do not re-record an unexplained baseline.

## Implementation checklist

- [x] Complete F-278, F-279 and F-280 through dependency-prefix checkpoints.
- [x] Add marker and generated-table option types in existing files.
- [x] Author checked XE, TA, INDEX, caption-selected TOC and TOA fields.
- [x] Extend owned-span discovery and accepted source traversal.
- [x] Implement hierarchy, caption selection, authority groups, range targets and formatting.
- [x] Reuse one post-insertion deterministic snapshot for all table page values.
- [x] Preserve producer content and reject unsafe publication atomically.
- [x] Capture fresh pinned Word ordering and page-target records.
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

## Exclusive resources

- `scripts/test_sprint_workflow.py`, completed capability owner assertion
- `scripts/readme_doctests.py`, matching measured archive tuples and dates
- `README.md`, affected archive footprint rows
- `crates/rdocx-oxml/README.md`, affected archive footprint row

- `crates/rdocx/src/field.rs`
- `crates/rdocx-oxml/src/text.rs`, shared generated-table operand and flag grammar
- `crates/rdocx/src/lib.rs`
- `crates/rdocx/tests/regression_test.rs`
- `crates/rdocx/src/document.rs`, if shared checked insertion or retained target access needs an existing-owner helper
- The named HLD sections in the impact list
- Hash baseline only if an individually reviewed intentional delta is proven

F-281 cannot share a wave with another story claiming these files.
