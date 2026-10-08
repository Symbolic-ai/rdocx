# F-282, Citations and bibliography authoring

**Status**: approved
**Sprint**: S90
**Size**: L
**Depends on**: F-278

## Problem

The facade has no CITATION or BIBLIOGRAPHY evaluator dispatch at `crates/rdocx/src/field.rs:9352`. Unknown instructions retain their cached display at `crates/rdocx/src/field.rs:9377`. Citation and bibliography control discriminators at `crates/rdocx-oxml/src/content_control.rs:34` do not provide source authoring or native result rebuilding. Existing custom XML graph helpers at `crates/rdocx/src/content_control.rs:589` provide relationship-owned datastore resolution, but do not identify bibliography Sources collections.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability matrix row DOCX-049.
- `docs/hld/03-architecture.md`, "What stays put", recursive field grammar, typed stories and staged cache updates.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", custom XML ownership, and "Package integrity".
- `docs/hld/12-testing-strategy.md`, "The Word corpus" and pinned source-built Word field evidence.
- `docs/hld/14-development-backlog.md`, "F-282, Citations and bibliography authoring".

Primary source model references are [Microsoft's Source documentation](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.bibliography.source?view=openxml-3.0.1), [Sources documentation](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.bibliography.sources?view=openxml-3.0.1), [AuthorList documentation](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.bibliography.authorlist?view=openxml-3.0.1), [Person documentation](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.bibliography.person?view=openxml-3.0.1), and [ECMA-376 Part 1, Bibliography part](https://download.microsoft.com/download/e/1/4/e14fb96f-83b8-4a2a-84db-7fa8acbe061a/Office%20Open%20XML%20Part%201%20-%20Fundamentals.pdf). The implementation reads the normative schema particle before assigning serialization order, rather than assuming that every bibliography child is an xsd:sequence.

## Approach

F-278 completion is a start barrier. Reuse its recursive FieldInstruction grammar, simple and complex Field constructors, ordered Vec<CT_R> caches and staged attachment transaction. Bibliography results are ordered story blocks inside a complex field owner. Preserve paragraph and table results through existing BodyContent::Paragraph and BodyContent::Table variants, with their actual begin, separate and end boundaries. Do not flatten block results into newline text or inline runs.

The user explicitly selected the full Word catalogue and approved bibliography.rs and the workflow records. Implement the complete source schema, all twelve bibliography styles installed in the pinned Word build, and every bibliography locale Word accepts. Do not reduce this to four source kinds, two styles or en-US. Installed resources provide this metadata inventory, not formatting implementation or parity evidence:

| Public style | Installed resource | OfficeStyleKey | Version | XslVersion |
|---|---|---|---|---|
| ApaSixthEdition | APASixthEditionOfficeOnline.xsl | APA | 2006.5.07 | 6 |
| Chicago | CHICAGO.XSL | Chicago | 2012.10.26 | 16 |
| Gb7714 | GB.XSL | GB7714 | 2006.5.07 | 2005 |
| GostName | GostName.XSL | GOST - Name Sort | 2006.5.07 | 2003 |
| GostTitle | GostTitle.XSL | GOST - Title Sort | 2006.5.07 | 2003 |
| HarvardAnglia | HarvardAnglia2008OfficeOnline.xsl | Harvard - Anglia | 2010.2.02 | 2008 |
| Ieee | IEEE2006OfficeOnline.xsl | IEEE | 2010.2.02 | 2006 |
| Iso690AuthorDate | ISO690.XSL | ISO 690 - First Element and Date | 2006.5.07 | 1987 |
| Iso690Numeric | ISO690Nmerical.XSL | ISO 690 - Numerical Reference | 2006.5.07 | 1987 |
| MlaSeventhEdition | MLASeventhEditionOfficeOnline.xsl | MLA | 2006.5.07 | 7 |
| Sist02 | SIST02.XSL | SIST02 | 2006.5.07 | 2003 |
| Turabian | TURABIAN.XSL | Turabian | 2006.5.07 | 6 |

The intentionally misspelled installed filename ISO690Nmerical.XSL is retained in style identity. These are pinned Word style implementations, not a claim that Chicago or Turabian matches the latest externally published edition. No APA Seventh Edition resource was established. Never copy proprietary XSL formatting logic, execute it as the production formatter, or redistribute those files.

Standard Sources root metadata has no locale slot. BibliographyStyleInfo.locale
is None for standard style metadata, while unrecognized producer attributes
remain raw and unchanged. set_bibliography_options atomically sets the standard
document style and applies locale, locale_filter and tags to existing supported
BIBLIOGRAPHY instructions. None and empty tags remove the corresponding owned
formatting, filter and selection switches. Explicit zero remains field-owned
and distinct. Preserve other instruction switches and producer XML. This setter
does not rewrite CITATION locales or evaluate stored caches. A malformed or
ambiguous owner required by the global rewrite aborts the whole operation.
With no bibliography owner, validate all options and set only standard style
metadata. Do not invent a persistent root locale default. insert_bibliography
uses private style-metadata attachment and its new field options, leaving other
existing bibliography instructions unchanged. Tests cover multiple owners,
removed selections, exact unrelated switches and caches, malformed atomicity
and insertion isolation.

Owned bibliography option replacement uses a concrete token-span helper in
existing rdocx-oxml/src/text.rs with the existing tokenizer. Its real consumer
is the global bibliography setter. Replace only the owned l, f and m switch
and operand spans, preserving all unowned opcode and switch case, whitespace,
escaping and order exactly. Reject malformed or ambiguous token boundaries
before any publication. No second tokenizer, trait, generic or new file.
Tests discriminate bare and quoted operands, escaping, repetitions, unowned
raw spelling, unknown switch adjacency and malformed atomicity. Existing field
opcode policies remain unchanged. Include rdocx-oxml in the changed-crate scoped
gate and carry parser and additive API riders through the named HLD03/12 impact.

The shared grammar recognizes CITATION operands l, m, p, v, f and s and
flags n, y and t. BIBLIOGRAPHY l, m and f are operands, retaining the captured
imported optional f spelling. The opcode-specific change preserves other field
policies and exact raw instructions. Tests discriminate quoted and bare values,
flag adjacency, repeated selectors, optional and malformed operands and captured
last-locale association. No unknown switch acquires guessed semantics.

Source kind coverage is the seventeen schema values from [Microsoft's DataSourceValues reference](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.bibliography.datasourcevalues?view=openxml-3.0.1): ArticleInAPeriodical, Book, BookSection, JournalArticle, ConferenceProceedings, Report, SoundRecording, Performance, Art, DocumentFromInternetSite, InternetSite, Film, Interview, Patent, ElectronicSource, Case and Misc. Native authoring and formatting cover every kind, including source-specific property and contributor-role choices.

Locale scope is every LCID accepted by the pinned Word bibliography engine, including document-default selection, source LCID, citation locale switches and bibliography locale switches with their observed precedence. Word.sdef's WdLanguageID enumeration has 204 entries and 202 distinct candidate low-word language values, including two sentinel values. Its SHA256 is 7cb51b924cab566320cc3019e92c1516302ae11220a81ae6279f1930c26320e7. Capture preflight maps candidates to actual numeric LCIDs and probes acceptance, rather than assuming AppleEvent enum codes equal LCIDs. The installed style-name metadata lists 24 localized label LCIDs, which must not be mistaken for the complete result-locale set. Regional variants and Word's own locale alias or fallback behavior remain supported with the same resulting text. Record any Word-rejected candidate as a rejected oracle input, not a supported-locale cache fallback. Discover any additional accepted engine LCIDs through the pinned Word language collection and document them in the same manifest. No supported locale may be left unimplemented or silently mapped to en-US.

Public types and exact signatures, implemented in the approved `crates/rdocx/src/bibliography.rs` module:

```rust
pub enum BibliographySourceKind {
    ArticleInAPeriodical, Book, BookSection, JournalArticle,
    ConferenceProceedings, Report, SoundRecording, Performance, Art,
    DocumentFromInternetSite, InternetSite, Film, Interview, Patent,
    ElectronicSource, Case, Misc,
}
pub struct BibliographyPerson {
    pub first: Vec<String>,
    pub middle: Vec<String>,
    pub last: Vec<String>,
}
pub enum BibliographyAuthor {
    People(Vec<BibliographyPerson>),
    Corporate(String),
}
pub enum BibliographyContributorRole {
    Artist, Author, BookAuthor, Compiler, Composer, Conductor, Counsel,
    Director, Editor, Interviewee, Interviewer, Inventor, Performer,
    ProducerName, Translator, Writer,
}
pub struct BibliographyContributor {
    pub role: BibliographyContributorRole,
    pub value: BibliographyAuthor,
}
pub enum BibliographySourceField {
    AbbreviatedCaseNumber, AlbumTitle, BookTitle, Broadcaster, BroadcastTitle,
    CaseNumber, ChapterNumber, City, Comments, ConferenceName, CountryRegion,
    Court, Day, DayAccessed, Department, Distributor, Edition, Institution,
    InternetSiteTitle, Issue, JournalName, Medium, Month, MonthAccessed,
    NumberVolumes, Pages, PatentNumber, PeriodicalTitle, ProductionCompany,
    PublicationTitle, Publisher, RecordingNumber, ReferenceOrder, Reporter,
    ShortTitle, StandardNumber, StateProvince, Station, Theater, ThesisType,
    Title, PatentType, Url, Version, Volume, Year, YearAccessed,
}
pub struct BibliographyProperty {
    pub field: BibliographySourceField,
    pub value: String,
}
pub struct BibliographySource {
    pub tag: String,
    pub guid: String,
    pub kind: BibliographySourceKind,
    pub locale: Option<u32>,
    pub contributors: Vec<BibliographyContributor>,
    pub properties: Vec<BibliographyProperty>,
}
pub enum BibliographyStyle {
    ApaSixthEdition, Chicago, Gb7714, GostName, GostTitle, HarvardAnglia,
    Ieee, Iso690AuthorDate, Iso690Numeric, MlaSeventhEdition, Sist02, Turabian,
}
pub struct CitationSourceOptions {
    pub tag: String,
    pub pages: Option<String>,
    pub volume: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub suppress_author: bool,
    pub suppress_year: bool,
    pub suppress_title: bool,
}
pub struct CitationOptions {
    pub sources: Vec<CitationSourceOptions>,
    pub locale: Option<u32>,
}
pub struct BibliographyOptions {
    pub style: BibliographyStyle,
    pub locale: Option<u32>,
    pub locale_filter: Option<u32>,
    pub tags: Vec<String>,
}
pub struct BibliographyStyleInfo {
    pub style_key: Option<String>,
    pub style_path: Option<String>,
    pub locale: Option<u32>,
    pub supported_style: Option<BibliographyStyle>,
}
pub struct BibliographySourceInfo {
    pub tag: String,
    pub guid: Option<String>,
    pub source_type: Option<String>,
    pub supported: Option<BibliographySource>,
}
pub struct BibliographyUpdateReport {
    pub updated_citations: usize,
    pub rebuilt_bibliographies: usize,
    pub diagnostics: Vec<String>,
}
impl Document {
    pub fn bibliography_sources(&self) -> Result<Vec<BibliographySourceInfo>>;
    pub fn add_bibliography_source(&mut self, source: &BibliographySource) -> Result<()>;
    pub fn replace_bibliography_source(&mut self, source: &BibliographySource) -> Result<()>;
    pub fn remove_bibliography_source(&mut self, tag: &str) -> Result<()>;
    pub fn bibliography_style(&self) -> Result<Option<BibliographyStyleInfo>>;
    pub fn set_bibliography_options(&mut self, options: &BibliographyOptions) -> Result<()>;
    pub fn insert_citation(&mut self, position: &StoryRunPosition, options: &CitationOptions) -> Result<()>;
    pub fn insert_bibliography(&mut self, position: &ContentLocation, options: &BibliographyOptions) -> Result<()>;
    pub fn update_bibliography(&mut self) -> Result<BibliographyUpdateReport>;
    pub fn update_bibliography_with_default_locale(&mut self, application_locale: u32) -> Result<BibliographyUpdateReport>;
}
```

Keep `update_bibliography` usable for owners whose locale resolves from the
package. Add `update_bibliography_with_default_locale` for the external
application locale required by source LCID 0 or other measured runtime-default
cases. Validate a supplied locale as a supported nonzero LCID. Resolve that
context only at the captured default-selection step, preserving explicit source
and field overrides and leaving LCID 0 and omitted XML unchanged. With missing
required context, retain the complete owner cache and report that application
locale is required. This is missing context, not an unsupported locale. The
explicit-context operation must implement every accepted locale. Existing
citation insertion with absent or zero locale and imported zero-locale source
updates are concrete consumers. No context wrapper, trait or generic is needed.
Test equivalent explicit/default resolution, override precedence, zero and
invalid context rejection, preserved XML and atomic cache retention.

Use existing ContentLocation and StoryRunPosition. No new trait, generic, builder wrapper or runtime XSLT engine is proposed. The concrete collation dependency rider below defines the selected facade dependency contract. Private source parsing and result formatting live together in bibliography.rs so a reader can identify the executing logic locally. All sixteen schema contributor roles are modeled. Source author data follows the Sources/Source tree, contributor-role Author structure, NameList/Person records and Corporate choices permitted by the schema. Repeated name/property members retain their sequence. The explicit property enum avoids ambiguous publication mappings and covers every standard source text property. Tag, Guid, SourceType, LCID and Author are modeled separately. Validate every property's schema cardinality, simple-type bound and role choice before authoring. Unknown producer extensions remain preserved outside these modeled members.

The normative transitional schema in the [official ECMA-376 Part 4 archive](https://ecma-international.org/wp-content/uploads/ECMA-376-4_5th_edition_december_2016.zip)
clarifies the actual particles. Source members and outer Author contributor
roles are repeated choices, so legal repetition and interleaving must survive.
Person has ordered Last*, First*, Middle* members. NameList requires at least
one Person. Fourteen contributor roles require exactly one NameList. Author
and Performer alone admit an optional NameList or Corporate choice. Imported
empty optional roles stay in their original raw shape, without an invented
person. Typed authoring validates the concrete role/value choice.

The same normative shared types define source ST_String and LCID ST_Lang as
unrestricted XML strings. Do not describe the SDK metadata's String255 check
as a normative schema limit, truncate longer imported values, or impose a
blanket 255-character authoring rejection without separately pinned Word
requirements. API tag, GUID and supported kind validation remains distinct
from XSD cardinality. Repeated identity members are inspectable but an ambiguous
owned identity fails checked mutation rather than selecting a guessed first
or last value. The typed view does not replace the original ordered source
spans, especially multiple outer Author elements interleaved with properties.
The extracted normative bibliography/common schema hashes and exact lines are
retained in /private/tmp/S90-F282-block-cache-integration-map.md.

Select the bibliography Sources root by expanded namespace name through the actual main document's internal customXml relationships. Canonical authoring uses the OOXML bibliography namespace. Preserve producer namespace spelling and declarations on imported owners. Any legacy namespace support must be explicitly established by source and capture evidence, not local-name matching.

A bibliography part has application/xml content type and an internal customXml source relationship. Resolve its existing item-properties relationship when present and preserve its ds:itemID, schema references and extensions. Creation allocates collision-free item and properties names, relationship IDs, and store identity. Add all necessary content-type and relationship edges transactionally. Reject multiple candidate bibliography collections, malformed targets, unexpected edge types, external datastore edges or conflicting identities before publication. Do not fetch external targets or assume customXml/item1.xml.

Authored tags and GUIDs are validated and unique in the collection. Caller GUIDs are explicit so source-built oracle identifiers can be compared exactly. Replace identifies by tag, retains existing GUID and rejects an identity change. Preserve unsupported source records and all unowned XML. Parsing is namespace-qualified and prefix-tolerant. Writing uses fixed prefixes for new XML with schema-valid child particles.

An owned existing source mutation replaces only selected modeled property spans. Retain original unknown root attributes, namespace declarations, style settings, locale data, nonstandard source types, producer contributor extensions, unknown child subtrees and unrelated custom XML bytes. Metadata inside a replaced standard simple-text property is not silently discarded, such input is rejected as an ambiguous owned property. A no-op operation preserves package bytes.

Implement independent concrete Rust formatting for all twelve styles. Use exhaustive BibliographyStyle and BibliographySourceKind matches that select concrete source-member assembly rules. Locale records supply observed names, dates, labels, separators, punctuation and collation policy. Assemble directly into existing CT_R, CT_P and CT_Tbl result structures with run, paragraph, table, grid and cell properties, avoiding a second rendering tree or a forwarding wrapper. Reuse the existing ordered story-block representation and atomic mutation transaction for fields spanning sibling paragraphs and tables. Name-list and citation disambiguation state is computed once per source collection. Numeric styles use the observed global citation/source order, while name/title styles use their recorded collation and tie-break rules. Do not sort all locales by Unicode scalar values or approximate full sorting with ASCII lowercase.

Share actual repeated operations such as name selection, date presentation, citation grouping and source ordering through concrete functions, not a speculative formatting trait. Freeze each style's full source-specific grammar from primary public style references and independently captured Word updates. Include all contributor roles, personal and corporate authors, no-author and multiple-author cases, missing properties, long-name abbreviation, author/year disambiguation, grouped citations, page locators, prefix and suffix, suppression switches, numeric citation assignment, localized dates/labels/punctuation, locale-specific collation, bibliography sort, character/run formatting, hanging-indent layout and source mutation. Preserve exact rich result structure, not just its concatenated text. The user asked for Word's catalogue, so do not substitute a newer edition of a style for the pinned Word edition. Validate public options through F-278's exact instruction grammar. Do not infer switch spellings or style identifiers from public enum names. Any collation dependency need discovered during implementation is a plan revision with dependency-direction and packaging riders, not an excuse to approximate locale semantics.

Every standard source kind, installed catalogue style and Word-accepted locale is supported for authoring and native materialization. An imported nonstandard source discriminator, custom user-provided style, invalid LCID or genuinely unsupported producer extension remains inspectable and unchanged with a stable diagnostic. This fallback boundary must not include any catalogue member. Match Word's observed aliases and fallback choices where Word itself applies them, without an agent-invented en-US fallback. BibliographyStyleInfo preserves imported style identity and reports supported_style=None only for a noncatalogue style. The outer Option is None only when style metadata is absent.

Ordered CitationSourceOptions retain per-source pages, volume, prefix, suffix and suppression switches. Captured Word updates visibly apply these switches to the initial tag or the latest m-tag. CitationOptions.locale is the field-wide formatting selector. The captured repeated-l Patent controls use the final locale for both cited sources, so locale must not be modeled as a per-source override. Imported repeated switches keep their original ordered instruction spelling while evaluation follows the captured last-l behavior and explicit source-locale precedence. BibliographyOptions retains repeated m-source selection and both l-formatting and f-filter locale semantics. Follow [Microsoft's CITATION implementation notes](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/bd78b823-ebd8-432b-89a3-aac7f3de99b4) and [BIBLIOGRAPHY implementation notes](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/1aed887d-2615-4119-b901-ce3ec798cccf). Capture the pinned build's observed behavior when locale switches differ, including Word's documented f/l precedence, and assert it explicitly. Empty tags means unfiltered bibliography selection, while an empty CitationOptions source list is invalid. Citation volume, pages, prefix, suffix and suppression remain separately representable for each source. Other standard field formatting switches remain available through F-278's typed/raw instruction surface and are preserved by this updater.

Citation order is document-wide and deterministic, with physical-story ownership used for patches. Discover all sources and result owners before changing a candidate. Compute citations and structured bibliography entries, patch caches and parts, serialize and reparse, then commit once and invalidate layout caches once. Stale positions, duplicate identity, ambiguous ownership and dangling relationships leave complete bytes unchanged. Normal save remains leave alone. Referenced source deletion is rejected across supported physical stories and safely classified preserved CITATION instructions. Ambiguous preserved references make deletion fail closed.

The locale preflight also starts from the 223 documented Office bibliography
LCIDs in [MS-OE376, Part 4 Section 7.6.2.39](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/6c085406-a698-4e12-9d4d-c3b0ee3dbc4a).
This published catalogue contains 23 values absent from the installed dictionary.
Source LCID 0 denotes runtime selection. Probe the union of this catalogue,
installed candidates and any additional engine-discovered values. Published
membership is candidate evidence, not observed acceptance by the pinned build.


Microsoft's bibliography-specific Office LCID table establishes223 documented
valid ordinary language IDs. Its final row excludes other values, while Word
source LCID0 separately requests runtime application context. The installed
Word language dictionary adds only0 and1024 to that ordinary table union,
with1024 remaining a proofing sentinel. Implement every documented valid member
with its independently observed native formatting, including English-equivalent
results, rather than treating such members as unsupported. Matching complete
rich output establishes supported behavior without asserting an unobservable
internal alias or fallback bit. Successful F9 for undefined1133 or999999 does
not admit either value to the valid catalogue. Preserve their imported caches
and source metadata under the existing invalid-input boundary.

The published table is a documented floor, not proof that the2026 pinned engine
has no additional valid extension. Inspect bibliography-specific language UI
for any actually exposed extension and capture it before claiming full coverage.
Proofing labels alone are not that registry. Keep application-default context,
source overrides, field selectors and filtering distinct in native controls.
All223-by12 dense rows, additional grammar and mutation controls, actual reopen
and native implementation parity remain required. The authenticated research
report is /private/tmp/S90-F282-locale-acceptance-boundary-readiness.md, SHA256
d89ad48259fac10db21034302e4ac836acabf7fe61fabf1513cf994b29223ef2.

## Captured block and switch contract clarification

The pinned IEEE matched ordering captures contain fourteen inline CITATION
owners and fifteen BIBLIOGRAPHY owners per document. Word converts the source
simple bibliography fields to complex fields. Each bibliography result spans
a paragraph containing begin and separate, a sibling table, a trailing
paragraph and a final paragraph containing end. Selected results have one
row and two cells. The full result has fourteen rows and two cells. Numeric
labels occupy the left column and rich reference text the right column.
Preserve actual grid widths, table and cell properties, including the measured
width difference between one-digit and two-digit labels. Existing CT_Tbl and
BodyContent supply the concrete representation, with no new rendering tree,
module or trait.

Source-only reversal preserves all twenty-nine IEEE display results and their
rich structure apart from explicitly recorded editing and divId metadata.
Citation-only reversal changes all twenty-nine results, reverses global
reference numbers and full bibliography rows, and keeps selected references'
global numeric labels. APA6's matching triple preserves display text for both
deltas while citation-only reversal changes producer RefOrder values. These
bounded records discriminate source order from citation encounter order.
They do not prove a universal collation algorithm or identify indistinguishable
Smith Title A ties.

APA6 switch pairs discriminate pages, volume, prefix, suffix and author/year
suppression. Equal title, locale and empty-page results in that pair do not
prove those branches. The separate Patent locale controls discriminate the
final repeated field locale and explicit source overrides. Bibliography
locale-filter controls select source language independently of the contrasting
formatting locale. Source LCID 0 removal, numeric-to-name LCID normalization,
and existing RefOrder rewrites are genuine Word source changes and stay
separate from cache and source-preservation assertions. Preserve imported
locale spelling, omitted and zero XML under the approved facade contract.
Do not infer application-default context from proofing language or equal
display text. Test reference-order metadata against the captured document
encounter order and preserve all unrelated source members during owned writes.

Embed the authenticated block, association, ordering and locale records in
the existing regression entrypoint. The IEEE baseline saved DOCX fingerprint
is be57dd054ef87289aafdea76783d6f275bae2cf3aa260af7d64a760cee23a1dc.
The source-only and citation-only variants are
4522d4e1095b25c37f4727ef405495a66ecadd79a3b97913c73100d9649e6c84 and
9d908b27a8d899ed640f85f007e7845457c3aa76696229adb010910e590db9ec.
All captures use Word 16.113.2 build 16.113.26092012 and Poppler 26.01.0.
They are actual update/save captures, with no reopened stability or native
formatter parity claim yet. Full catalogue scope remains unchanged.

Additional authenticated batches retain separate source and lifecycle evidence.
The ten missing dense style baselines cover 180 fields, and the counts, Chicago
ordering, authored-switch and locale controls add 403 fields. Their 20 no-F9
reopens preserve every DOCX archive byte for byte and all per-page PDF text.
Seven PDFs are pixel identical at 96dpi, while thirteen retain recorded
horizontal positioning differences below 0.214 points, with identical font
payloads and no vertical displacement. Do not claim aggregate pixel parity.
These records are retained in /private/tmp/S90-F282-priority-reopen.

The isolated repeated APA page, volume, prefix and suffix controls use the
first supplied value, as does repeated IEEE page. Keep that distinct from the
previous discriminating final-locale controls. Nondiscriminating combined and
title-suppression cases do not establish universal association rules. The
locale control 1133 produces Russian output, while 999999 differs from English
in a full bibliography despite an equal Patent citation. Neither equality nor
these control labels prove acceptance or a universal fallback policy.

Ten contributor-role captures cover another 790 fields and retain all source
XML, including repeated and empty name members. MLA and Turabian full lists
apply repeated-author substitutions that change selected versus full entry
text in the captured collection. Sparse profiles also expose style-specific
invalid-source displays and omissions from full lists. Preserve these native
collection effects and ambiguous identical-text ties explicitly, rather than
assuming one standalone formatter result suffices in every collection.
These captures add bounded contracts and do not complete the full catalogue,
locale, adversarial, reopen or native implementation parity requirements.

Further authenticated date and contributor-count controls are separate
bounded records. Ten date vectors cover 310 fields, 150 sources per phase
and 53 actual PDF pages. Nine additional contributor-count vectors cover
459 fields, 225 sources per phase and 72 PDF pages. Their combined indices
are F282-ten-dates-authenticated-audit-index, SHA256
5e19f3e65271b6907988ed20c99912024d4ccbd049d96b1cc943d95291c8b613,
and F282-nine-counts-authenticated-audit-index, SHA256
245d22e5ba55d0e005cf6979a8cb4571915150e31f0f716aaab3b5d6f1e1f0ac,
under the existing Word capture directory. All 171 index and manifest hash
bindings were independently rechecked.

Date controls distinguish absent from explicitly empty values, partial dates,
access dates, numeric and textual months and emitted invalid date literals.
Contributor-count controls preserve complete ordered author vectors and
style-specific truncation and collection substitutions. Keep source omissions
and explicit empty members distinct. Preserve full versus selected bibliography
contracts rather than inferring collection-independent thresholds from these
vectors. Actual saved style selection generated some caches before explicit
F9, and later F9 changes real cached page-break markers in certain styles.
Retain those raw differences and measure this lifecycle separately from
ordinary library cache preservation. These batches have no additional no-F9
reopen or Rust formatter parity evidence yet.

The repeated-source-member controls separately measure duplicate scalar order
and interleaved outer Author containers. Preserve their namespace-qualified
ordered source XML, including repeated RefOrder members before Word's observed
normalization. IEEE omits the duplicate publication year in its captured Book
vector, while both ISO styles emit the first supplied year. A property absent
from displayed output does not establish a deduplication policy. Retain exact
selected and full results, source order, ambiguous identical-text matches and
style-specific collection substitutions. Remaining styles and source kinds
need their own evidence before these bounded rules become implementation
contracts. Full catalogue scope and the differential gate remain unchanged.

## Concrete collation dependency rider

The full bibliography catalogue uses the concrete ICU4X collator candidate in
the already approved bibliography.rs. Add normal dependency edges only from
rdocx, with exact constraints icu_collator=2.3.1 and icu_locale_core=2.3.0
for the actual comparison and dynamic locale parser consumers. Disable default
features, enable collator compiled_data and locale-core alloc. Five additional
normal facade edges intentionally constrain publication, icu_collator_data,
icu_normalizer, icu_normalizer_data, icu_locale_fallback and
icu_locale_fallback_data all exactly2.3.0 with default features disabled.
These constrain the named collation, normalization and fallback engines and
three bundled datasets across publication. They do not promise a frozen whole
consumer graph, feature union or compiled artifact. Incompatible consumer exact
constraints may produce a resolver error. No unused workspace-only entries or
verification-only patches stand in for published constraints.

The bundled datasets identify CLDR48.2.1 and ICU export release-78.1rc. Before
fresh dependency compilation, require ICU4X_DATA_DIR absent and inspect Cargo
configuration and flags for a manually selected custom-data cfg. Use a unique
scoped temporary target to establish provenance without cargo clean or changing
repository configuration. Record actual resolved versions, feature union and
normalized packaged facade constraints. Check Rust1.93 separately from1.97.1,
the facade WASM targets and no-default paths, cargo deny, verified22-package
publication dry run and each10MiB archive bound. Measure comparable native,
Python and WASM linked artifact deltas. Additional registry dependencies remain
separate packages, rather than vendored copies in every workspace archive.

ICU defaults, successful constructor fallback and LCID-to-language parsing do
not prove Word acceptance or ordering. Concrete comparison options and style
key assembly must match independent native controls for case, accents, scripts,
punctuation, compatibility, numeric strings, tailored locales and independent
source/citation tie reversals. Retain original spelling and grouping identity.
Do not persist ICU sort keys as source identity. If this candidate cannot match
a measured catalogue case, investigate the concrete divergence before claiming
materialization. No standard catalogue owner may fall back to an unsupported
cache path. This rider does not alter F-281's completed ASCII boundary.

The primary manifest archives and the read-only proposal are authenticated in
/private/tmp/S90-F282-collation-dependency-decision-readiness.md, SHA256
c8cd6345eb7d8726c24627537ec21873ca3b8aaa436e3e67bdd5e36e3febf901.
[Cargo resolution rules](https://doc.rust-lang.org/cargo/reference/resolver.html)
define the exact-constraint intersection. Actual build, license, packaging and
Word equivalence riders remain required results, not research passes.

## Rejected alternatives

- Conventional part filenames miss noncanonical producers.
- A second field parser disagrees with nesting, escaping and cached ownership.
- Proprietary XSL reuse violates the independent native implementation boundary.
- One run containing bibliography newlines loses paragraph formatting and ownership.
- Silent en-US fallback would violate the user's full-catalogue decision.

## Test plan

**Test gate**: differential. A source-built citation set and bibliography match
the pinned Word identifiers, ordering, display text, and round-trip package.

| Category | Test | Asserts |
|---|---|---|
| differential | full_bibliography_catalogue_matches_pinned_word | All twelve styles, all seventeen source kinds and every accepted locale match exact tags/GUIDs, text, rich runs, ordering, instructions and reopened sources |
| differential | bibliography_mutation_matches_pinned_word | Addition, replacement and unreferenced removal reproduce fresh Word results |
| round-trip | bibliography_preserves_producer_metadata | Unknown root/source children, contributor extensions, namespace declarations, locale/style metadata and unrelated parts retain exact bytes |
| integration | citations_round_trip_across_supported_stories | Body, table, control, header, footer, note and text-box placements retain owner identity and ordered caches |
| regression | bibliography_invalid_mutations_are_atomic | Duplicate identities, ambiguous collection, wrong graph edge, stale location and referenced deletion preserve complete bytes |
| regression | noncatalogue_bibliography_keeps_saved_cache | Nonstandard kind, custom style, invalid locale and unknown producer switch retain display with diagnostics, no standard catalogue member takes this path |
| differential | bibliography_locale_precedence_matches_word | All accepted LCIDs, omitted locale, source/citation/bibliography overrides, regional aliases, collation and Word-native fallback are exact |
| differential | bibliography_block_and_switch_ownership_matches_word | IEEE table results retain complex block boundaries and global labels, source-only and citation-only ordering differ as captured, per-source switches and field-wide final locale remain distinct |
| regression | bibliography_catalogue_coverage_is_complete | Installed style keys, schema kind set and captured accepted-locale manifest are fully represented, no skip or unimplemented row |
| unit | bibliography_schema_particles_and_namespace | Authored child order, prefix aliasing, namespace shadowing and foreign lookalikes |

Add source-built records to the existing regression entrypoint. No new integration binary or binary fixture. Fresh capture pins Microsoft Word for Mac 16.113.2 build 16.113.26092012. Preflight snapshots twelve style metadata records and obtains actual bibliography LCID support through independent Word probes. The 202 candidate language values include sentinels, so do not fabricate an accepted-locale count before probing.

Use a resumable source-built style-by-locale matrix with all seventeen source kinds in each document. This can approach 2,400 distinct style/locale documents and 40,000 source-result combinations. The existing regression entrypoint's ignored capture preparation generates bounded temporary input batches and reads saved output records. Actual Word open, field update, save and reopen actions use cua_repl UI controls. Do not use osascript, AppleScript, hidden automation or commands that invoke Word outside the available computer-use surface. Bind each normalized output to input fingerprint, Word build, actual style key/version, LCID and the observed UI action log. Resume only completed SHA-bound batches, and keep user communication active during the long capture. Avoid a second tracked harness or invented expectations. Expected records may be losslessly grouped by observed locale equivalence, but every catalogue row still maps to and checks an independently observed result. Hash-only assertions cannot replace readable display and structure comparisons.

Capture additional adversarial cases for every style's source/contributor branches and locale-specific grammar: all standard property members, missing values, multiple authors, mixed scripts, dates and month labels, ambiguous sort keys, multiple-source citation order, suppression and page switches, identical author/year disambiguation, mutations and rich paragraph/run formatting. Locale alias equivalence is an observed result, never assumed from shared language prefix. Word may intentionally ignore some source properties for a style, and the differential records must assert that observed omission while round-trip tests prove metadata retained.

Store temporary DOCX/PDF and capture checkpoints outside tracked fixtures. Encode sanitized readable oracle records in existing test code, with provenance and coverage manifest. Never compute expectations using the Rust formatter or copy Word XSL logic. Opening a file alone is not parity evidence. If any style, source-kind or accepted-locale case remains uncaptured or unimplemented, the full catalogue gate fails. Report the exact blocked rows, do not claim completion or silently carry a smaller domain. This is substantial implementation and evidence work, and its original L sizing does not justify weakening the user's selected scope.

Use cargo +1.97.1 with the sprint isolated target directory. Run focused rdocx checks and tests, /verify --scoped and applicable riders. Full gate runs once on final integration.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Parser/serializer: read packaging and PresentationML conventions, prove schema particles, namespace ownership and byte-preserved opaque subtrees.
- Published API: read bindings spec and structural rules, state additive pre-1.0 impact, run canonical packaging dry run and 10 MiB archive gate.
- External oracle: read differential-testing, assert exact Word version and retain capture provenance.
- Dependency/data: execute the concrete collation rider, exact resolved and packaged constraints, fresh bundled-data provenance, MSRV, WASM, no-default, supply-chain and archive gates.
- New file/module: bibliography.rs and its module were explicitly approved by the user. No new trait, generic, crate or feature flag.

## Hash harness

Expected unchanged. Existing samples do not invoke citation/bibliography authoring or updates. No baseline movement is allocated.

## Exclusive file claims

- `crates/rdocx-oxml/src/text.rs`, existing-tokenizer owned switch span replacement

- `crates/rdocx/Cargo.toml`, seven concrete collation and publication constraint edges
- `Cargo.lock`, reviewed resolved graph
- `scripts/readme_doctests.py` and affected README footprint rows, measured archive metadata only

- `crates/rdocx/src/bibliography.rs`, creation approved
- `crates/rdocx/src/lib.rs`
- `crates/rdocx/src/field.rs`, only shared structured-result/patch integration
- `crates/rdocx/src/content_control.rs`, only existing datastore helper changes
- `crates/rdocx/tests/regression_test.rs`
- HLD files listed above, delivery totals remain integrator-owned

Shared field.rs and regression_test.rs claims require serial implementation with conflicting stories even though the only formal prerequisite is F-278.

## Implementation checklist

- [ ] Complete F-278 and reconcile final builder and staged transaction contracts.
- [ ] Implement the approved full source/style/locale scope in approved bibliography.rs.
- [ ] Probe and record actual Word bibliography locale support and catalogue coverage.
- [ ] Capture independent Word evidence and freeze readable expected grammar.
- [ ] Implement source inspection and transactional relationship closure.
- [ ] Implement authoring, result rebuild, exact preservation and fallback diagnostics.
- [ ] Add differential, round-trip and atomicity coverage.
- [ ] Pass scoped verify and zero-finding microscope, then prepare handoff.

## Open questions

None for user scope or file approval. The user selected the full Word catalogue and approved bibliography.rs and workflow records. Actual accepted LCID inventory, locale precedence, result grammar and schema cardinalities are technical capture/implementation evidence obligations. They do not authorize narrowing the catalogue if the work or oracle capture is difficult.
