# 03, Architecture

## Three families, one workspace

```
crates/
  # format-neutral infrastructure
  oxml-core          units, xml helpers, entity decoding, raw-XML capture,
                     core / app / custom properties
  oxml-opc           ZIP and OPC package, relationships, content types
  oxml-media         image sniffing, dimensions and DPI, MIME, media naming
  oxml-drawing       DrawingML: colour, transforms, geometry, fills, lines,
                     effects, theme, text body
  oxml-layout        output types, font manager, bundled fonts, line breaking
  oxml-pdf           PDF writer and tiny-skia rasteriser
  oxml-sml           minimal SpreadsheetML writer, chart workbooks only
  oxml-cli-support   range parsing, JSON envelope, output-path defaulting
  oxml-chart         ChartML model and renderer
  oxml-py-support    content paths, revision checks, stale-domain errors,
                     Length conversion helpers

  # WordprocessingML
  rdocx-opc          deprecated shim over oxml-opc
  rdocx-oxml         WordprocessingML types, re-exports oxml-core
  rdocx-layout       flow engine, paginator, blocks, tables, style resolver
  rdocx-pdf          deprecated shim over oxml-pdf
  rdocx-html         HTML and Markdown emitter
  rdocx              the python-docx-shaped facade
  rdocx-cli  rdocx-wasm  rdocx-py

  # PresentationML
  rpptx-oxml         PresentationML types
  rpptx-layout       inheritance resolver, chart routing and flattener
  rpptx-render       resolved slides to page frames
  rpptx-chart        deprecated shim over oxml-chart
  rpptx              the python-pptx-shaped facade, plus assets/default.pptx
  rpptx-cli  rpptx-wasm  rpptx-py
```

## The dependency rule

The graph is acyclic and layered. **Nothing in `oxml-*` may depend on
`rdocx-*` or `rpptx-*`.** There is no exception, and
`no_shared_crate_depends_on_a_format_crate` in `oxml-drawing` keeps it that
way.

```
oxml-core ──┬─→ oxml-drawing ──→ rpptx-oxml ──→ rpptx-layout ──→ rpptx-render
            │         │                                              │
            │         ←────────────────── rdocx-oxml ──→ rdocx-layout │
            ├─→ oxml-opc                                    │        │
            ├─→ oxml-media                                  ↓        ↓
            └─→ oxml-layout ──→ oxml-pdf ←──────────── rdocx-pdf   rpptx
                                                            ↓        ↓
                                                          rdocx   rpptx-cli
```

**The theme adapter.** `rdocx_oxml::theme::Theme` is a thin adapter over
`oxml_drawing::CT_OfficeStyleSheet` (`impl From<&CT_OfficeStyleSheet> for
Theme`), so that `rdocx-layout`'s existing `LayoutInput.theme` field does not
churn. The impl lives in `rdocx-oxml`, which owns `Theme`, so the edge runs
`rdocx-oxml → oxml-drawing` like every other cross-family edge.

It used to sit in `oxml-drawing` and point the other way, as the one documented
exception. That single edge made the two publication trains mutually dependent,
because `rdocx-layout` already depends on `oxml-layout`. Once both trains
carried breaking changes neither could publish first, so the adapter moved to
the side that owns its target type.

## Why these seams

**`oxml-opc` does not depend on `oxml-core`.** It has its own small local-name
handling. Staying independent means it is publishable first and consumable
alone. `rdocx-wasm` consumes the complete `rdocx` facade rather than using this
lower-level seam as a second document model.

**`oxml-media` has no dependencies at all.** It owns byte sniffing, image header
probing, and intrinsic EMU sizing through its local `NativeSize` value. It
remains a leaf that anything can take cheaply without importing `oxml-core`.
The `rdocx` facade depends on it directly for collision-free Word media names,
sniffed package metadata, and byte-first HTML and layout MIME inputs.

**`oxml-layout` is where the format boundary genuinely falls.** Its
output, font, and line modules hold page frames, positioned elements, glyph
runs, colours, fonts, and owned line parameters, none of which name a document
format.

One construct is an exception and is called out rather than glossed. A text
segment carries an optional `NoteRef`, a footnote or endnote reference, and
notes are a WordprocessingML idea with no PresentationML counterpart. It sits
here because a note reference has to survive line breaking, which is the shared
code, and the alternative is a parallel segment type for one field. The pair
`NoteStream` and `NoteRef` replaced an untyped `footnote_id` that had the same
problem less visibly.
`rdocx-layout` keeps its Word-specific input and converts paragraph alignment,
tabs, leaders, underlines, spacing, wrapping, and twips in `convert.rs`. The
converter also preserves Word's established glyph slicing and automatic line
height at this boundary. That seam is the reason the PDF backend transfers for
free.

**`oxml-pdf` consumes `LayoutResult` and shared image metadata.** It depends on
`oxml-layout` for the rendering contract and on `oxml-media` for byte sniffing
and header probing. It has no format-specific workspace dependency. A slide is
a page with a fixed size, so the same crate serves both formats without knowing
either exists. The `rdocx` facade renders through this crate directly, while
`rdocx-pdf` remains an exact deprecated re-export shim.

**`rpptx-layout` is separate from `rpptx-render`.** The inheritance resolver
produces a `ResolvedSlide` in which every theme reference, colour transform and
inherited property is already collapsed to a concrete value. The renderer
consumes that and nothing else. Freezing this contract is what lets the resolver
and the renderer be built and tested independently.

**`oxml-chart` depends on `oxml-layout` for backend-neutral geometry.** Its
typed ChartML caches lower directly to `PathElement` and `Group` values. The
edge stays inside the format-neutral family, and no PDF or raster backend
becomes a chart dependency. `rpptx-chart` is an exact deprecated re-export shim
over this shared owner.

**`rpptx-layout` depends on `oxml-chart` for native chart projection.** Package
assembly parses scoped ChartML targets, then the resolver freezes a completed
backend-neutral group or a visible fallback in `ResolvedContent`. The
PresentationML resolver depends inward on the shared chart engine.
`rpptx-render` and the format-neutral backends consume only the frozen group
and do not parse ChartML.

**`rdocx-layout` depends on `oxml-chart` for native chart projection.** The
Word facade resolves document-scoped chart and theme relationships into layout
input. The layout engine freezes each inline or anchored chart as a
backend-neutral group before pagination. `oxml-layout` carries that group
through line breaking and page placement without gaining a ChartML or document
family dependency.

The `rdocx` crate uses `rpptx` only as a development dependency for the exact
cross-family chart golden. The production dependency tree has no Word to
PowerPoint edge. The all-target tree admits this test-only edge and retains the
rule that no `oxml-*` crate depends on either facade family.

## What stays put

`rdocx-oxml` remains a real crate holding the WordprocessingML grammar for
text, properties, tables, styles, numbering, borders, headers and footers,
footnotes, comments, settings, placeholder replacement, and `drawing.rs`. The
`wp:` inline and anchor code in the latter is Word-only and has no pptx value,
so it is not migrated.

The settings model owns the separate `w:settings` root and read-only
projections for `w:documentProtection` and valid `w:docVars` entries. It reports
the four supported editing modes, the recorded enforcement and formatting
flags, password-verification metadata, and ordered document-variable names and
values. Prefix aliases are accepted on read. Parsed producer bytes remain the
sole serialization source, so root attributes, schema order, unmodelled
children, and unsupported or malformed protection and variable elements
survive unchanged. Invalid elements are preserved but are not reported through
the typed projections.

The comments model owns typed comment entries and the three body anchor forms.
Comment bodies retain ordered paragraphs, producer attributes, and unmodelled
children. Paragraph and run models retain each anchor at its insertion boundary
without moving neighbouring raw XML. Parsing accepts in-scope aliases for the
WordprocessingML namespace, while serialization uses the fixed `w:` prefix.
The comments-extended model owns paragraph-id parent linkage and resolved state,
with unmodelled attributes and root children retained at their original
boundaries. The `rdocx` facade owns the relationship-resolved pair of comment
parts and coordinates them with the anchors in the main document.

The Word text model also projects bookmark starts and ends at direct-run
boundaries while retaining every marker as ordered raw XML. Simple and complex
fields share one recursive `Field` grammar with a normalized name, text or
nested arguments, switches, cached result, and optional dirty state. Its private
source records the original field form, run partition, and producer XML.
Unchanged fields therefore write their original bytes. Cache and dirty updates
rewrite only the typed values while preserving run formatting and unmodelled
neighbours. Markers are recognized only as direct run children through their
in-scope WordprocessingML namespace bindings. Malformed sequences remain opaque
raw XML, while unsupported valid fields retain their cached display. Dirty
complex hyperlinks are not reported as `Document::links()` until the update
policy defines how to handle them. The `rdocx` facade correlates bookmark ids
and owns mutation across top-level body paragraphs.
`rdocx-layout` resolves bookmark text and maps page targets, while the shared
`oxml-layout` boundary exposes only format-neutral `Target` and `TargetPage`
field kinds.

The `rdocx` facade owns pure field evaluation over that recursive grammar. It
walks every typed paragraph in main text, tables, content controls, distinct
header and footer parts, footnotes, and endnotes. Package-backed inputs come
from unique bookmarks, styles, core and custom properties, and settings
document variables. Date-time, filename, merge, and included-text values come
only from an explicit caller context. Evaluation reports resolved text,
pagination deferral, or a stable cached-display fallback without mutating the
package. Sequence counters remain isolated by story. Raw text boxes and other
untyped XML remain outside this evaluation boundary.

The facade also owns explicit field cache updates across that same typed story
scope. It evaluates the complete field set before changing cloned document and
package-backed parts. Resolved values replace the stored display and clear the
field-local dirty flag. Pagination deferrals and stored-display fallbacks keep
their cache and become dirty so Word may retry them. Only validated staged XML
is committed, then both layout caches are invalidated once. Existing save and
byte methods remain leave alone operations that preserve cache content and
dirty spelling. Update-aware save methods opt into the same atomic operation
before writing. The settings-level `w:updateFields` value remains untouched.

The `rdocx` facade owns structured template evaluation over
`serde_json::Value`. The focused `template` module recognizes scalar tags
across ordinary run boundaries and pairs nested `for` and `if` controls with a
container-aware stack parser. Top-level marker paragraphs clone body entries,
including section-ending paragraphs and their section properties. Marker rows
clone every row in a multi-row template group inside their owning table. The
owning table is retained, and each row and cell is deep-cloned with its merge,
banding, content-control, and ordered raw XML state. Numbered paragraphs in a
loop retain their source `numId` and level, which keeps one continuous list
without allocating definitions. Numbering references are validated before
evaluation. Loop variables form lexical scopes, and dotted lookup searches the
innermost scope before the root value. Structural controls are limited to the
main body and its tables. Headers, footers, text boxes, and chart labels retain
scalar-only replacement through the existing Word placeholder mapper. A
successful render commits the staged document and package together and
invalidates both layout caches once.

The content-control model owns one recursive `CT_Sdt` grammar at block, row,
cell, paragraph, and run placement boundaries. It reports tag, alias, numeric
id, bounded control type, and custom XML binding metadata from `CT_SdtPr`.
Unmodelled attributes, properties, and content children remain in ordered raw
slots. Empty or malformed controls remain opaque. Prefix-tolerant readers and
fixed-prefix writers follow the same boundary rules as the surrounding
WordprocessingML model.

The `rdocx` facade owns content-control value mutation because one operation
can cross the typed document and package parts. Immutable summaries expose the
control metadata and display text. Lookup and mutation select tags before
aliases, so map application updates each control at most once. Display changes
preserve the control shell, direct run formatting, and nested control
boundaries. The facade stages every selected display and custom XML change on
cloned state, validates the resulting XML, then commits once and invalidates
layout once. Any rejected control or binding leaves both document and package
state unchanged.

The revision model belongs to `rdocx-oxml`. Insertions, deletions, moves,
property changes, deleted text, and contextual markers are typed read-only
projections over captured WordprocessingML subtrees. The captured raw subtree
is the sole serialization source until an explicit accept or reject operation
replaces it. Invalid revision metadata remains preserved but is not reported.
Prior run, paragraph, table, and section properties are projected with the
namespace context of the revision element, including nested properties.
The ordered preservation sidecars added to the public low-level Word model and
the `RunContent::DeletedText` variant form the breaking pre-1.0 0.8.0 boundary
for the next published stable family. The higher-level `rdocx::Document`
revision API remains additive.

The `rdocx` facade owns revision resolution because one operation can replace
content wrappers, property owners, paragraph boundaries, and table rows.
Accepting keeps insertions and move destinations, while rejecting keeps
deletions and move sources and converts deleted text to ordinary text.
Property rejection restores exactly one namespace-correct prior property
value. Contextual markers act on their owning run, paragraph mark, numbering
property, or row. Resolution stages the complete main-document XML, resolves
selected descendants before their enclosing subtree, reparses the result, and
commits once only after validation succeeds.

`rdocx-layout` owns the renderer-only revision projection. The
`LayoutInput::revision_view` selector chooses an accepted or tracked view. The
engine merges ordinary runs and typed revision runs at their preserved
boundaries without mutating the package. Accepted layout keeps insertions and
move destinations and omits deletions and move sources. Tracked layout keeps
both sides, applies neutral decorations, and carries changed-paragraph state
through pagination. The `rdocx` facade owns the concrete `RenderOptions` value
that passes this selection into layout. Default accepted renders reuse the
normal and deterministic caches, while tracked renders remain uncached.

`rdocx-layout` keeps the flow model: the engine, the paginator, blocks, tables
and the style resolver. Slides do not paginate, so none of it transfers. The
flow engine resolves Word relationship IDs to content-addressed `MediaId`
values before pagination, and page output carries the resolved bytes and MIME
type rather than a relationship-scoped placeholder. One `MediaRegistry` per
layout compares complete bytes, assigns deterministic alternate IDs when two
compact keys collide, and is shared by the lower-level layout and pagination
entry points.

Footnotes and endnotes are laid out into a `NoteRegistry` before pagination, and
the paginator reserves, splits and draws them. Note placement is part of
pagination rather than a pass that runs after it, because a page's body height
depends on the note area it owes, and a note that does not fit continues on the
following page. The registry pre-shapes each note's marker, so the paginator
places notes without needing a mutable font manager.

Each note is laid out once per distinct section content width rather than once
per document, and is looked up by the width of the section drawing it. A note is
broken to the measure of the section carrying its reference, since that is the
measure it is drawn at, and reserve and render therefore still read the same
lines. A document whose sections share a page size registers one width and lays
each note out once, which is the common case. Endnotes are measured against the
final section, because they are emitted after the last body page and drawn
against that section's geometry wherever their references sit.

The paginator also reflows a paragraph around any floating drawing that wraps,
because whether a drawing overlaps a line is only known once the paragraph has a
position on a page. The inputs to line breaking are therefore kept alive past
layout, but only for a document that actually holds a drawing whose wrap is not
`none`, since those inputs hold the same shaped glyphs the laid-out lines do.

Text also flows around a wrapping drawing anchored to a **later** paragraph,
which Word documents do routinely. A drawing framed by the page or a margin has
a position without its own paragraph being placed, so one pass is enough. A
drawing framed by its own paragraph does not, so a section holding one
paginates **twice**: the first pass records where each such drawing landed and
on which page, and the second offers those rectangles to the text above them.
The first pass is identical to a single-pass run, and a section holding no such
drawing paginates once, which is every sample and every corpus document today.

Two passes, and not a fixed point. The second pass reflows earlier text, which
can move the drawing's own paragraph, so the rectangle it flowed around may be
slightly stale. Iterating is not guaranteed to terminate, since growing a
paragraph can push a drawing to the next page, which shrinks the paragraph,
which pulls the drawing back. Two passes give one answer, always.

The two note streams are placed differently and are keyed apart. A footnote
sits at the foot of the page carrying its reference and takes height from that
page. An endnote costs its page nothing and is emitted after the last body
page, where endnotes flow from the top of their own pages without a separator
rule. A reference therefore carries a `NoteRef`, its stream and its number,
because the streams number independently and a document may hold a footnote and
an endnote sharing a number.

## Versioning

The 15 shared and PowerPoint publication candidates use the explicit common
incubating version in their manifests and workspace pins. The family adds
`oxml-chart` as the format-neutral owner while retaining `rpptx-chart` as a
source-compatible deprecated shim. The released `rdocx-*` crates
continue to use the separate workspace version. Version preparation and
manifest eligibility do not authorize publication. Every release still
requires `/release` at an exact reviewed SHA and separate final approval at
the external mutation boundary. `oxml-cli-support` is the format-neutral owner
of range parsing, JSON envelope, and output-path contracts. It has no
dependency on either document family, while CLI binaries depend inward on it.

The immutable `rpptx-v0.1.2` release contains the earlier 12-package family.
`oxml-cli-support` and `rpptx-cli` remain unpublished at 0.1.2. The original
14-package family is published at the immutable 0.1.3 and 0.2.0 boundaries. No
existing tag or registry version was moved or overwritten.

The `rpptx` facade owns formatting-preserving presentation text replacement.
`Presentation::replace_text` applies literal, non-recursive replacement across
contiguous regular runs in ordinary shapes, nested groups, and table cells.
Fields, breaks, and selected alternate-content fallbacks remain traversal
boundaries so the facade preserves their unmodelled or separately typed XML.

`rpptx-*` crates carry their own `keywords` and `categories`, because the
workspace values say `["docx", "word"]` which would be wrong on a presentation
crate. Once publication is approved, the rpptx family uses its own pre-1.0
version train so breaking releases do not drag the released rdocx family with
them. The families fold into a lockstep train once rpptx stabilises.

## Crate-level conventions

- **quick-xml pull parsing only.** No serde, no derive, no macros, no codegen.
  Every element's parser and serialiser is hand-written. This is a deliberate
  existing choice and the new crates follow it.
- **Spec names.** Types are `CT_*` and `ST_*` after the schema, under a
  crate-level `#![allow(non_camel_case_types)]`.
- **Root parts** get `from_xml(&[u8]) -> Result<Self>` and
  `to_xml(&self) -> Result<Vec<u8>>`. **Nested elements** get
  `from_xml(reader: &mut Reader<&[u8]>)` and
  `to_xml<W: Write>(&self, writer: &mut Writer<W>)`.
- **Prefix-tolerant on read, fixed prefix on write.** `matches_local_name`
  strips any prefix and compares the local part.
- **An unmodelled enumerated value reads as an absent attribute.** A value
  parser rejects a string it does not list, and the property parsers treat that
  rejection as "not specified" rather than propagating it. An absent attribute
  means the element's default, which is usually inheritance from the style
  chain, so the surrounding properties survive and the document opens. The
  parsers stay fallible, so a caller that wants strictness keeps it: the
  tolerance belongs to the reader, not to the type. A value carried this way is
  lost on save, which is the accepted cost of opening the document at all.
- **Unmodelled subtrees are preserved verbatim** via `capture_element` into
  `raw_xml` fields. This matters far more for PresentationML than for
  WordprocessingML, and it is the scope control for an otherwise unbounded
  format: parse only what you render, preserve the rest.
- **`thiserror`, no `anyhow`.** One error enum per crate plus a `Result` alias.
- Edition 2024, MSRV 1.93.

## Facade conventions

Both facades use the same borrow-handle idiom rdocx already has: a mutable
`Foo<'a>` wrapping `&'a mut CT_Foo` and a read-only `FooRef<'a>`, with
consuming builders for formatting so calls chain, `&mut self` methods for adding
content that return a nested handle, and index-based `Option`-returning
accessors that never panic.

The `rdocx` facade also provides direct immutable paragraph lookup. Mutable
and read-only paragraph handles each provide total run count and lookup, while
only the mutable handle provides mutable run lookup. These accessors let the
Python binding re-resolve lazy index paths without allocating paragraph
snapshots, clearing layout caches for reads, or reaching through private OOXML
fields.

`Document::text` traverses body paragraphs and table cells in document order.
The WASM binding uses that additive facade accessor for its existing `getText`
method and otherwise owns one complete `Document`. It never reaches into
`rdocx-oxml` or maintains a second package representation.

The same direct lookup rule covers document tables and paragraphs nested in
table cells. `Document::table` and `Document::table_mut` are total, and cell
handles provide paragraph counts plus immutable and mutable lookup. Run and
paragraph formatting expose direct `Option<bool>` values and clear-capable
setters, preserving the distinction between inherited, explicitly false, and
explicitly true formatting without bypassing the facade.
The binding-only underline variants travel through a bounded integer-code
accessor so the published exhaustive Rust `UnderlineStyle` enum stays stable.

Low-level content-control traversal is recursive and ordered. Body, table,
row, cell, and paragraph accessors expose each wrapped ordinary paragraph,
table, row, cell, and run once while retaining the surrounding `CT_Sdt` for
metadata lookup. The facade consumes this single WordprocessingML ownership
tree and does not maintain a second content-control representation.

Revision traversal follows that ownership tree through the main body, tables,
cells, and content controls. `Document::revisions` reports every valid modeled
revision once in document order as a borrowed `RevisionRef`. The facade does
not copy or reparse the raw subtree, and revisions outside the main document
part remain outside this traversal.

Revision mutation uses explicit all, exact-author, inclusive RFC 3339 instant,
and id selectors. One id operation resolves every modeled element carrying the
id, while undated revisions do not match date ranges. Invalid bounds or a
malformed selected revision leave the typed document, package bytes, and both
layout caches unchanged. A successful operation invalidates layout once.

Word comment mutation uses `RunPosition` and half-open `RunRange` values whose
body indexes select top-level paragraphs and whose run indexes select insertion
boundaries. `Document` validates both endpoints before mutation, allocates
collision-free comment and paragraph ids, updates the comment parts and all
three anchors together, then invalidates layout once. `CommentRef` is a
read-only view over the typed comment and its comments-extended thread entry.
Replies follow paragraph-id parent linkage, resolution applies to the thread
root, and removal deletes the selected comment plus descendant replies without
deleting unrelated runs or producer XML.

Word bookmark mutation reuses the same top-level `RunPosition` and half-open
`RunRange` boundary. `Document::bookmarks` returns immutable correlated
summaries in document order and reports malformed, unmatched, reversed, or
duplicate markers without hiding their preserved XML. `Document::add_bookmark`
validates both endpoints and the Word name, rejects producer-reserved and
duplicate names, allocates the first free nonnegative id, stages both marker
insertions, commits once, and invalidates layout once.

The `rpptx` facade provides the same total lookup boundary for slides, nested
shape trees, placeholders, text frames, paragraphs, regular runs, tables and
cells. Consuming mutable accessors transfer a facade borrow into its nested
handle, which lets the Python binding re-resolve a path without exposing
PresentationML internals or storing a Rust borrow in a pyclass.

The facade also owns package-to-render-input assembly. Its deterministic render
entry points resolve the current package once and return either the shared
render input and layout or a complete PDF. The corpus example and
`rpptx-wasm` call that boundary, so neither binding nor development tooling
maintains a second PresentationML package interpretation path.

Every consuming formatting builder on `Paragraph`, `Run`, `Table`, `Row`, and
`Cell` has a non-consuming `set_*` twin because a `mut self -> Self` builder
cannot back a Python property setter. The 61 consuming builders delegate to
their setter twins, so Rust callers retain chaining while borrowed handles and
Python properties use in-place mutation.
