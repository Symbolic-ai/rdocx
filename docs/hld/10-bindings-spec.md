# 10, Bindings spec

Owners: `oxml-py-support`, `rdocx-py`, `rpptx-py`, `rdocx-wasm`, `rpptx-wasm`,
`oxml-cli-support`, `rdocx-cli`, `rpptx-cli`.

## The PyO3 lifetime problem

A `#[pyclass]` must be `'static`. The facade is built on borrow handles:
`Paragraph<'a> { inner: &'a mut CT_P }`, plus consuming builders and
`Document::add_paragraph(&mut self) -> Paragraph<'_>` which holds the document
mutably borrowed for the handle's whole life. Python additionally requires that
`p = doc.add_paragraph("x")` stay usable across arbitrary later mutations,
including ones that reallocate the content vector.

References are out, categorically. Four options were weighed:

| Option | Verdict |
|---|---|
| **Index and path handles** re-resolving on every call | **chosen** |
| `Rc<RefCell<_>>` or `Arc<Mutex<_>>` in the core | rejected: rewrites every crate, pollutes the Rust API with borrow noise for users who never touch Python, and `Rc` is not `Send` so `allow_threads` is lost |
| Arena with generational ids | correct long-term, but converts the content vectors across every crate. Deferred |
| A separate owned mirror API | rejected: doubles the API surface, and "attach" reintroduces the identity problem |

### The chosen design

```rust
pub enum PathSeg { Slide(usize), Shape(usize), Body(usize), Row(usize),
                   Cell(usize), Para(usize), Run(usize) }
pub struct ContentPath { pub segs: SmallVec<[PathSeg; 5]>, pub revision: u64 }
pub struct RevisionCounter { current: u64 }

#[pyclass(name = "Document")]
struct PyDocument { inner: rdocx::Document, revision: RevisionCounter }

#[pyclass(name = "Paragraph")]
struct PyParagraph { doc: Py<PyDocument>, path: ContentPath }
```

The Rust API adds only total, index-based paragraph and run accessors needed to
re-resolve these handles. Read-only resolution stays on immutable paragraph
handles so it cannot clear the layout caches. Run setters and structural
mutations retain their required mutable resolution. No interior mutability
leaks into the core.
Aliasing is checked by PyO3's own `RefCell` on the pyclass, so a violation is a
clean `RuntimeError`, never undefined behaviour. Resolution is a handful of
vector index operations, negligible against FFI overhead.

The shared crate carries the Word path variants consumed by the rdocx binding
and the `Slide(usize)` plus repeatable `Shape(usize)` variants consumed by the
rpptx binding.

### The invalidation problem, handled loudly

An index path addresses a **position**, not an object. After
`doc.remove_content(1)`, a handle to paragraph 3 would silently read what used
to be paragraph 4. python-docx does not have this problem because it holds an
lxml element pointer that follows the element.

v0.1 therefore carries a **document revision counter**, bumped after every
successful structural mutation and captured by every handle at construction.
Failed and value-only mutations do not bump it. The shared crate reports a
concrete Rust `StaleElementError` on mismatch. The package binding maps that
domain error to its Python exception with the same revisions and message:

```
rdocx.StaleElementError: paragraph handle was created at document revision 4,
but the document is now at revision 5 (a structural change invalidated it).
Re-fetch it with doc.paragraphs[i].
```

**Loud failure beats silently wrong data.** There are no snapshot accessors that
keep working after invalidation.

v0.2 upgrades to lazily-assigned stable ids backed by `w14:paraId`, which OOXML
already defines for exactly this purpose, so they round-trip to disk and improve
DOCX fidelity as a side effect. Then a handle survives unrelated removals and
matches python-docx semantics, with no API change.

### Two supporting decisions

**Collections are lazy.** `doc.paragraphs` is a pyclass holding only
`Py<PyDocument>` and implementing `__len__`, `__getitem__` with negative and
slice support, and `__iter__`. `Document::paragraphs() -> Vec<ParagraphRef>` is
never called from the binding.

**Consuming builders are bypassed.** A `fn bold(mut self, val: bool) -> Self`
cannot back a Python property setter. The facade exposes 61 non-consuming
`set_*` twins: 24 on `Paragraph`, 19 on `Run`, and 18 across `Table`, `Row`, and
`Cell`. The existing builders delegate to them. The surface is additive, and a
borrowed nested handle can mutate without a rebind:
`doc.paragraph_mut(3).unwrap().add_run("text").set_bold(true)`.

**Threading.** `Document` remains `Send` and `Sync`. Its normal and
deterministic layouts live in separate
`Mutex<Option<Arc<WordLayoutResult>>>` caches. One private normal-font engine
lives behind a separate mutex and survives result invalidation, with a
compile-time regression gate preserving that threading contract.
`to_pdf`, `render_all_pages` and `to_bytes` run inside `py.allow_threads`, so a
Python thread pool genuinely parallelises work across documents. Concurrent
rendering of one document shares the immutable cached result after the first
layout for that font mode. That is a capability python-docx has no equivalent
for.

## Python API shape

**Drop-in compatibility is an explicit non-goal. Source compatibility for the
documented API is an explicit goal.**

python-docx's real-world surface is inseparable from lxml, and a large fraction
of production code reaches through `._p`, `._r`, `doc.element.body`, `qn()` and
`OxmlElement`. Promising drop-in means promising an lxml-shaped shadow API that
can never be delivered, and every gap then reads as a bug.

The compatibility promise is the completed binding surface, not every public
python-docx method. Its executable gate is an explicit seventeen-example
manifest from the python-docx 1.2.0 Working with Documents, Quickstart, and
Working with Text pages. Each entry records a stable v1.2.0 tagged source URL,
heading, exact source statements, declared transformation, and normalized
structural assertion. Sixteen entries use only the `docx` to `rdocx` import
substitution. The Quickstart held-row example additionally re-fetches
`document.tables[0].rows[1]` before its second cell assignment because the
first cell text replacement advances the global revision and stales the held
row. This is the minimal public compatibility adaptation and does not weaken
strict revision validation. A touch of `._p` raises a clear
`NotImplementedError` naming the attribute and its equivalent rather than an
`AttributeError` five frames away.

```python
from rdocx import Document, Inches, Pt, RGBColor, WD_ALIGN_PARAGRAPH

doc = Document("in.docx")
p = doc.add_paragraph("Hello")
p.alignment = WD_ALIGN_PARAGRAPH.CENTER
r = p.add_run(" world")
r.font.bold = True
r.font.size = Pt(18)
doc.add_picture("img.png", width=Inches(2))   # height inferred by oxml-media
doc.save("out.docx")
doc.save_pdf("out.pdf")                        # documented as an rdocx extension
```

- `font` and `paragraph_format` are themselves handles, so `r.font.bold = True`
  writes through the chain. They store only a document reference and content
  path, re-resolve on every operation, and become stale after a structural
  mutation.
- **Tri-state properties return `None` for inherit**, `True` or `False` when
  explicit. rdocx's `Option<bool>` already matches. Never collapse `None` to
  `False`.
- `Length` is a pure-Python subclass of `int` and returns EMU, matching
  `docx.shared.Length`, with `.inches`, `.cm`, `.mm`, `.pt`, `.emu` and
  `.twips`. `Inches`, `Cm`, `Mm`, `Pt` and `Emu` are immutable subclasses, and
  `RGBColor` is an immutable three-channel tuple. Float constructors use
  `int(value * factor)`, preserving the truncation toward zero pinned by the
  Rust `Length`. The types are available at the top level and from
  `rdocx.shared`, while native-base inheritance stays outside the Python 3.9
  limited ABI.
- The bounded core enum inventory is pure-Python `IntEnum`:
  `WD_ALIGN_PARAGRAPH` and `WD_UNDERLINE` in `rdocx.enum.text`, plus
  `WD_TABLE_ALIGNMENT` and `WD_CELL_VERTICAL_ALIGNMENT` in
  `rdocx.enum.table`. All four are also top-level exports. Their checked
  integer literals cover the paragraph, run and table variants exposed by the
  S33 facade, including `WD_ALIGN_PARAGRAPH.CENTER == 1`. Underline codes use a
  total binding-oriented facade value accessor rather than expanding the
  published exhaustive Rust `UnderlineStyle` enum.
- The package layer owns `RdocxError(Exception)` as the base, with
  `PackageError`, `XmlError`, `StaleElementError` and `LayoutError` beneath it.
  OPC, I/O and missing-part failures map to `PackageError`, OXML failures map
  to `XmlError`, layout failures map to `LayoutError`, and the shared stale
  domain error maps to `StaleElementError`. `oxml-py-support` therefore remains
  independent of any Python base class.

The S33 formatting inventory is intentionally bounded to font name, size,
colour, bold, italic, underline and strike, plus paragraph alignment, spacing,
indentation, keep-with-next, keep-together, page-break-before and widow
control. Assigning `None` clears direct tri-state formatting. The S33 table
inventory is lazy table, row, cell and nested paragraph lookup, table style,
alignment and width, plus cell text, width and vertical alignment. These
handles use `Body`, `Row`, `Cell`, `Para` and `Run` path segments and reach the
document only through the public `rdocx` facade.

`rpptx` mirrors python-pptx through an unpublished mixed-layout `rpptx-py`
crate. `Presentation` owns the Rust facade and one revision counter. Lazy
layouts, slides, shapes, placeholders, text frames, paragraphs, runs, columns
and cells store only a presentation reference and `ContentPath`. The bounded
source-compatibility surface is the seven python-pptx 1.0.2 Getting Started
workflows. They change the import namespace and re-fetch through the public
path after each structural write, because strict global revision invalidation
intentionally stales every pre-write handle and collection. Pure-Python
`Length`, `Inches`, `Pt` and the required `MSO_SHAPE` members keep native
inheritance outside the limited ABI.

## Native Word facade stability

The public `rdocx` facade is the common source for native, Python, WASM, and
CLI consumers. Custom lists are created with `Document::add_list_definition`
from up to nine `ListLevel` values. Each value selects a `ListNumberFormat` and
an optional start number. Later slice entries are ignored because Word exposes
exactly nine levels. Paragraph numbering stores an explicit list ID and a
zero-based level from 0 through 8. Its in-place setters return `false` without
mutation for a larger value. `Document::set_list_level` can redefine an
existing level without rebuilding the document. A rejected redefinition is
side-effect free.

When native callers enable the default-off `agile-encryption` feature,
`Document::open_encrypted`, `Document::from_encrypted_bytes`, and the bounded
bytes variant open password-protected OOXML through the shared package layer.
`Document::save_encrypted` and `Document::to_encrypted_bytes` write the shared
fixed Agile profile after staging a cloned document and package. A failed save
does not mutate the live document, and the file API publishes through a
sibling temporary file. These additive native APIs are unavailable without
the feature. Python, WASM, and CLI manifests do not enable the feature, so
their API and dependency graphs remain unchanged.

When native callers enable the default-off `digital-signatures` feature,
`Document::verify_signatures` directly returns the shared package verification
reports. The additive API distinguishes cryptographic verification and
complete declared coverage from certificate-chain trust. It does not expand
Python, WASM, or CLI surfaces and those dependency graphs remain unchanged.

Native callers rebuilding one Word document from another can call
`Document::transfer_reusable_layout_from`. The method moves the source's normal
layout engine only when the complete private retained-work context matches. A
rejected transfer preserves both engines, a successful transfer preserves both
completed result caches, and no unchecked engine accessor becomes public. This
is an additive native Rust method. Python, WASM, and CLI surfaces gain no
transfer method.

Paragraph mutation supports explicit hard breaks and hyperlinks backed by a
document relationship. Table column mutation keeps the table width, grid
column, and every covering cell width consistent. A cell with `gridSpan`
receives the sum of its covered grid columns. Negative widths, invalid spans,
and overflowing totals are rejected without mutation. These are additive
stable APIs. Existing binding surfaces do not gain new methods implicitly, but
their owned `rdocx::Document` remains package-preserving when native code uses
the new operations.

Native Word callers can inspect comments through `Document::comments` and
author threads through `add_comment`, `reply_to`, `resolve_comment`, and
`remove_comment`. `RunPosition` and `RunRange` define top-level paragraph run
boundaries with an inclusive start and exclusive end. `CommentRef` exposes
comment metadata, text, parent identity, and resolved state without permitting
part-local mutation. These additions do not implicitly expand the Python,
WASM, or CLI surfaces. Those consumers continue to own the same
package-preserving `Document`, so native comment edits remain intact when a
binding subsequently saves it.

Native Word callers use `Document::bookmarks` for immutable `BookmarkRef`
summaries and `Document::add_bookmark` for atomic insertion over the existing
top-level half-open `RunRange`. A summary exposes an optional id, name, range,
current text, and marker issue. Insertion validates the Word name and both
boundaries, rejects duplicate or producer-reserved names, and returns the
allocated nonnegative id. The shared recursive `Field` model retains the
complete `REF` and `PAGEREF` instruction, target argument, cached display,
dirty state, source form, and producer XML. These additions are native Rust
APIs only. Python, WASM, and CLI consumers keep their existing surface and
preserve the typed content when they save the owned document.

Native Word callers evaluate fields with `Document::evaluate_fields` and an
explicit `FieldEvaluationContext`. `FieldDateTime` supplies deterministic civil
time. Caller maps supply merge values and included text, including
`source#bookmark` keys for bookmark-scoped includes. Each `FieldEvaluation`
records a snapshot-local document-order index, original instruction, stored
display, and a `FieldOutcome` that is resolved text, pagination deferral, or a
stable stored-display fallback. Evaluation is additive and read-only. It never
reads the ambient clock or filesystem and never changes field caches. Python,
WASM, and CLI surfaces gain no evaluator methods and continue to preserve the
same package content.

Native Word callers opt into cache materialization with
`Document::update_fields`, `Document::save_with_field_updates`, or
`Document::to_bytes_with_field_updates`. The facade stages the full evaluation
before mutation, updates resolved displays, and marks retained displays dirty.
Existing `save` and `to_bytes` methods continue to preserve intentionally stale
caches and producer dirty spellings. These methods are additive native Rust
APIs. Python, WASM, and CLI surfaces gain no field update methods and continue
to preserve updates already made through their owned `Document`.

Native Word callers merge flat records with `Document::mail_merge` or
`Document::mail_merge_sections`. Each record is a
`BTreeMap<String, String>`. Separate mode returns one complete validated
document per record. Section mode returns one document with record bodies in
input order and a next-page boundary after every non-final record. Empty input
is rejected. Missing merge values become empty text only inside these two
methods. A record-varying merge field in a referenced header, footer, footnote,
or endnote rejects section mode because it combines main-body stories only.
Both methods are additive on the pre-1.0 native Rust facade. Python, WASM, and
CLI surfaces gain no merge methods and continue to preserve documents already
merged by native code.

Native Word callers render templates with
`Document::render_template(&serde_json::Value)`. Scalar tags use
`{{ path.to.value }}` syntax and may cross ordinary Word run boundaries.
Dedicated marker paragraphs and rows use `{% for item in path %}` with
`{% endfor %}`, or `{% if path %}` with `{% endif %}`. Blocks nest within one
container. Loops require arrays and introduce lexical variables. Conditions
treat false, null, zero, empty strings, empty arrays, and empty objects as
false. Other JSON values are true. Structural generation is limited to the
main body and its tables, while other stories retain scalar rendering. Missing
paths, malformed markers, invalid scalar leaves, invalid numbering references,
and crossed container boundaries fail without mutation. One row loop may own
several adjacent template rows. Each iteration retains table banding, grid and
merge properties, and preserved row and cell XML. Repeated list items retain
one source numbering identity and level, so their sequence continues across
iterations. The existing method remains additive on the pre-1.0 native facade.
Python, WASM, and CLI surfaces gain no template method and continue to preserve
a document rendered by native code.

Native Word callers can also inspect content controls through
`Document::content_controls` and the tag or alias lookup methods.
`ContentControlRef` exposes immutable metadata and display text. Direct setters
update every matching tag or alias, while `bind_content_controls` applies a
string map with tag precedence and alias fallback. Bound values update their
custom XML datastore and displayed text atomically through the
package-preserving facade. These methods are additive native APIs. They do not
implicitly add Python, WASM, or CLI methods, and the existing binding surfaces
remain unchanged.

Native Word callers inspect direct body order through
`Document::body_items`. Each `BodyItemRef` borrows one paragraph, table,
body-level content control, or preserved unsupported XML child. It does not
flatten control content, and it does not change the recursive semantics of
`paragraphs()` or `tables()`. The API is additive on `rdocx` only. Python,
WASM, and CLI surfaces gain no ordered-body method and continue to preserve a
document opened and saved through their existing owners.

Native Word callers inspect tracked changes through `Document::revisions`.
Each immutable `RevisionRef` exposes the revision id, author, optional
timestamp, and `RevisionKind`. Results recursively cover the main document
body in document order, including tables, cells, and content controls. The
facade reads a typed projection while serialization continues to use the
captured raw WordprocessingML subtree. This is an additive native Rust API.
Python, WASM, and CLI surfaces do not gain revision methods, and their existing
load and save paths preserve the revision XML.

Native Word callers inspect document protection through the borrowed
`Document::document_protection` accessor. `ProtectionMode` distinguishes
read-only, comments-only, forced tracked changes, and forms-only intent.
`DocumentProtection` also reports the recorded enforcement and formatting
flags, provider type, algorithm class and type, algorithm SID, spin count,
hash, and salt. The accessor reports metadata only. It does not verify a
password or enforce access control. This additive Rust API does not add
Python, WASM, or CLI methods. Those surfaces remain unchanged and preserve the
relationship-resolved settings part when they save their owned document.

The low-level revision and field storage is an intentional breaking pre-1.0
Rust boundary. `RunContent` adds `DeletedText` and replaces the narrow
`FieldType` payload with the recursive `Field`, `FieldInstruction`,
`FieldArgument`, and `FieldSwitch` model. `CT_R`, `CT_P`, `HyperlinkSpan`,
`CT_PPr`, `CT_RPr`, `CT_SectPr`, `CT_TblPr`, and `CT_TrPr` add required
preservation or revision fields, including ordered raw-child sidecars.
`CT_TcPr` also adds an ordered raw-child sidecar that retains external
namespace bindings declared only on the property owner or enclosing cell.
Only WordprocessingML children advance its schema insertion boundary, so a
foreign same-local-name child remains in its source slot. Serialization keeps
`w:textDirection` before preserved `w:tcFitText` and `w:vAlign`. This sidecar
assigns absolute schema slots to the unmodelled standard `w:hMerge`, `w:tcMar`,
`w:hideMark`, `w:headers`, `w:cellIns`, `w:cellDel`, `w:cellMerge`, and
`w:tcPrChange` children. This sidecar is part of the intentional pre-1.0 0.8
low-level Rust source break. Existing exhaustive matches and full struct
literals must be updated or moved to the provided constructors. The workspace
and its exact seven-package stable family are published at 0.8.0, not as a 0.7
patch. Earlier immutable registry versions remain available.
The additive `rdocx::Document` facade and
unchanged Python, WASM, and CLI surfaces do not inherit this low-level source
break.

The low-level layout boundary also adds `source: Option<SourceSpan>` to the
exhaustive public `TextSegment` and `GlyphRun` structs. Existing external
struct literals must supply `None` when they do not own an exact source range.
`rdocx-layout` adds `WordStory`, `WordSourcePath`, and `WordLayoutResult`, plus
normal-font and deterministic provenance entry points. Node ids resolve only
through the result-local Word source table, and ranges use Unicode scalar
indices in the recorded revision view. The existing layout functions keep
returning `LayoutResult`. The `rdocx::Document` facade consumes the provenance
entry points through additive native accessors, while Python, WASM, and CLI
surfaces remain unchanged. The exhaustive literal change is published in both
the incubating 0.4.0 family and the stable 0.8.0 family.

Native callers resolve tracked changes through `accept_all`, `reject_all`, the
exact-author pair, the inclusive RFC 3339 date-range pair, and the id pair.
Each method returns the number of modeled revision elements resolved. Shared
ids select every matching placement, author matching is case-sensitive, and
missing dates do not match a date range. Invalid bounds and malformed selected
changes return an error before mutation. These eight methods are additive on
`rdocx::Document` only. Python, WASM, and CLI surfaces remain unchanged and
continue to preserve the resulting document when they save it.

Native callers generate tracked changes with `Document::compare`, supplying an
edited document, author, and RFC 3339 timestamp. The additive
`ComparisonDiagnostic` value reports stable formatting-only locations and
messages without turning those differences into revisions. Comparison rejects
existing modeled revisions and unsupported structural shell differences, and
it commits only after accepting and rejecting staged copies reproduce their
respective modeled baselines. This API is native Rust only. Python, WASM, and
CLI surfaces gain no comparison method and preserve comparison output when
they save their owned document.

Native Word rendering exposes `rdocx::RevisionView` and the concrete
`rdocx::RenderOptions`, whose default selects the accepted view. Additive
option-taking counterparts cover PDF bytes and files, single-page and all-page
raster output, page layout, deterministic rendering, and caller-supplied font
paths. The existing methods keep their accepted default. Python, WASM, and CLI
surfaces do not implicitly expose the selector and retain their existing
rendering behavior.

Native renderers obtain the complete positioned output through
`Document::layout` and `Document::layout_with_options`. Accepted calls return a
shared `Arc<WordLayoutResult>` from the normal-font cache, including pages,
font bytes, revision view, and the result-local Word source map. After a
mutation, the retained normal engine may reuse bounded context-independent
paragraph and shaping work while rebuilding the completed result. Tracked calls
stay uncached and use a distinct revision-view paragraph identity.
`Document::layout_with_fonts` and
`Document::layout_with_fonts_and_options` return owned uncached bundles whose
font mapping contains the exact caller-provided bytes selected for shaping.
They construct a caller-only engine and cannot observe the normal process font
snapshot. Deterministic calls remain isolated on the bundled-font-only path.
The built-in PDF, raster, and page accessors consume these same paths. This is
an additive pre-1.0 native Rust surface and does not add binding methods.

Native Word callers author watermarks with `Document::set_text_watermark` and
`Document::set_image_watermark`. Text uses fixed Word-like defaults of 468 by
117 points, 315 degree rotation, `D9D9D9`, Calibri, and 50 percent opacity.
Image callers provide positive width and height, while rotation stays zero and
opacity stays at 50 percent. Both methods replace one API-owned watermark in
every active default, first, and enabled even header variant atomically. These
methods are additive on the native pre-1.0 facade. Python, WASM, and CLI gain no
watermark methods and continue to preserve watermarks already authored through
their owned `Document`.

The public low-level `VmlWatermark` projection and the added paginator section
and header-selection fields are part of the intentional pre-1.0 Rust source
break for the next stable family. They expose renderer input, not a second
authoring surface. Opened header XML remains the serialization authority, and
callers should use the native `Document` methods for mutation.

The stable Rust family moves to 0.5.0 for the numbering preservation model.
`CT_Lvl`, `CT_AbstractNum`, `CT_Num`, and `CT_Numbering` expose raw XML state so
producer extensions survive typed mutations. Full struct literals written for
0.4 must add the preservation fields, or callers should use the existing
constructors. This is an intentional breaking pre-1.0 boundary. Python, WASM,
and CLI consumers continue through the package-preserving facade and do not
construct these low-level structs.

`CT_TabStop` also exposes `source_occurrence: Option<usize>`. Parsed numbering
tabs use this provenance to retain producer XML on the same occurrence after
an edit, insertion, or removal. New tabs carry `None`, and semantic equality
continues to compare only alignment, position, and leader. The public
`CT_Tabs::from_xml_with_prefixes` parser accepts the in-scope WordprocessingML
prefixes and tracks nested namespace shadows. Paragraph-property namespace
context stays in one internal projection used by numbering, style, body,
table-cell, header, footer, footnote, and endnote readers, so `CT_PPr` does not
expose a partially contextual parser. Established aliased and default
WordprocessingML inputs remain accepted outside numbering.

## Packaging

**maturin, mixed Rust and Python layout**, so type stubs and enum shims have a
home. `python-source = "python"`, `module-name = "rdocx._rdocx"`,
`features = ["pyo3/extension-module"]`. The rpptx package uses the parallel
`rpptx._rpptx` module name.

**abi3-py39.** One wheel per platform rather than one per interpreter version,
so roughly 6 wheels instead of 48. The cost is marginally slower attribute
access and no free-threaded build under abi3. Start abi3-only and revisit only
if profiling shows attribute overhead matters.

Matrix: `manylinux_2_28` x86_64 and aarch64, `musllinux_1_2` x86_64, macOS
x86_64 and arm64, Windows x86_64, plus an sdist.

Two traps specific to this workspace:

- **`fontdb`'s `fontconfig` feature is useless on musl and Windows.** Gate it
  per-target.
- **Bundled fonts are always compiled into wheels.** The optional
  `system-fonts` feature adds host discovery, but a bare manylinux container
  still has the bundled fallback inventory needed for `to_pdf()`. Roughly 4 MB
  per wheel is a fair trade for deterministic fallback text.

Each mixed package ships a hand-written native-extension stub beside its
extension module and a `py.typed` marker at package root. The stubs describe
concrete lazy handle and collection types, integer and slice overloads, typed
iteration, path-like inputs, byte outputs, optional values, bounded enum inputs,
and concrete Length returns. Native handles and collections are factory-only,
so their stubs reject direct construction just as the extension types do. The
pure-Python units, enums, and exception hierarchies remain inline typed rather
than duplicated in package-level stubs. Exact `mypy==2.3.0 --strict` smoke
checks and `stubtest` against freshly installed wheels keep the declarations
honest. Do not auto-generate them from PyO3.

**Distribution names `rdocx` and `rpptx`**, import names identical. The binding
crates are `publish = false`, because a cdylib has no business on crates.io.

## CI

`wheels.yml` on a **`py-v*` tag namespace**, separate from `publish.yml` on
`v*`, so a Rust patch release does not rebuild twelve wheels and a binding-only
fix does not force a crates.io release. Publishing uses PyPI trusted publishing
via OIDC, with no long-lived token in secrets. The workflow builds `rdocx` and
`rpptx` across the six declared targets, produces one source distribution per
package, and uploads each matrix product independently. Every native wheel is
installed into a fresh environment for its compatible pytest, exact
`mypy==2.3.0 --strict`, and `stubtest` gates. Each musllinux wheel is installed
in a fresh Python 3.9 Alpine environment and runs the same package parity suite
as the native cells.

The build jobs have only repository read permission. A separate publish job
depends on all wheel and source-distribution jobs, requires exactly twelve
wheels and two source distributions, and receives `id-token: write` only for a
`py-v*` tag event in the `pypi` environment. Manual dispatch builds and tests
artifacts but cannot publish them. Every external action and the maturin tool
version are pinned to reviewed immutable versions.

**A PR-time job that builds the wheel and runs pytest is mandatory.** The
absence of exactly this job for wasm is why `rdocx-wasm` rotted.

The rdocx parity suite pins and asserts `python-docx==1.2.0` before comparison.
It writes the approved S33 content and direct formatting with each producer,
opens both outputs with both readers, and directly compares normalized public
paragraph, run, table, cell, unit and enum records. It compares no ZIP or XML
bytes. Relative float line spacing remains distinct from absolute `Length`
spacing in those records. An explicit table style is checked after each saved
output is reopened by both readers. The suite commits no binary fixture and
keeps python-docx out of runtime package dependencies.

## WASM

### The rdocx wrapper

```rust
#[wasm_bindgen]
pub struct WasmDocument { inner: rdocx::Document }
```

`fromBytes` delegates to `Document::from_bytes`, and `toDocxBytes` delegates to
`Document::to_bytes`. The facade therefore flushes modeled changes into the
original package. Images, headers, footers, numbering, settings, themes, font
tables, notes, properties, content types, relationships, and opaque parts stay
in the package rather than being reconstructed by the binding.

The constructor, `fromBytes`, `addParagraph`, `addHeading`,
`addBoldParagraph`, `addTable`, `getText`, `paragraphCount`, `toDocxBytes`,
`toPdf`, `toHtml`, `toHtmlFragment`, `toMarkdown`, and `replacePlaceholder`
names remain stable. `toPdf` delegates to the normal `Document::to_pdf` facade
and returns its bytes directly. `Document::open`, `save`, and a second
deterministic PDF alias stay absent because browser callers supply and receive
bytes and the WASM profile already excludes host font discovery.

The `system-fonts` feature is default-on in `rdocx-layout` and `rdocx`, which
preserves native behavior. `rdocx-wasm` disables `rdocx` defaults, while the
bundled font data remains unconditional. The wasm32 graph therefore excludes
host font discovery without inventing a second bundled-font feature.

The R-class regression constructs a document with an image, header, and
numbering, then checks the complete part, relationship, and content-type graph
through `fromBytes` and `toDocxBytes`. The same contract is an inline
`wasm-bindgen-test` for Node. The Node test reflectively calls those generated
JavaScript members and crosses the `Uint8Array` boundary in both directions.
A second inline Node test calls generated `addParagraph` and `toPdf` members,
then requires a complete PDF with a Type 0 font, an embedded TrueType stream,
and the bundled Carlito base font. Pull-request CI target-checks the wrapper
with the locked workspace graph and runs both tests in Node.

`rpptx-wasm` owns one `rpptx::Presentation`, never a mini-model. Its default
profile exposes the constructor, `fromBytes`, `toBytes`, `slideCount`, and
`addSlide`. It includes the bundled default template but no renderer, PDF
backend, rasteriser, or host font discovery. The `render` feature adds only
`toPdf` and selects the facade's deterministic renderer. The optimized default
artifact must remain below 1,000,000 bytes after deterministic gzip.
Pull-request CI target-checks the default wrapper with the locked workspace
graph and runs its package-preserving inline test in Node.

The npm package names are `@tensorbee/rdocx-wasm` and
`@tensorbee/rpptx-wasm`. Both use the bundler target, their Rust package
versions, and release output optimized by exact wasm-opt 125 with `-Oz`,
`--enable-bulk-memory`, and `--enable-nontrapping-float-to-int`. Pull-request
CI creates local tarballs with `npm pack`, installs each tarball into a separate
fresh consumer, and checks the installed WASM, JavaScript glue, public
TypeScript declaration, and module import. This is an installation gate only.
The job has no npm publication, registry authentication, token, OIDC, release,
or tag authority.

## CLIs

`rpptx-cli` extends the seven-command `rdocx-cli` surface with `inspect`,
`text`, `convert`, `diff`, `replace`, `validate`, `render`, `thumbnail`, and
`outline`. It uses clap derive and `serde_json` for `--json`.

`inspect` reports the file, slide and layout counts, slide size, core metadata,
and each slide's identity, hidden state, and shape count. Its JSON form uses the
shared schema-1 envelope. `text` emits slide text in presentation order.
`convert` produces deterministic PDF or PNG output. Multi-slide PNG output uses
one-based filename suffixes and renders one slide at a time. `diff` compares
slide text with longest-common-subsequence semantics and rejects matrices above
one million cells. `replace` delegates to the facade's literal,
formatting-preserving text replacement. `validate` is dispatched separately so
its exit status carries the verdict. `render` uses deterministic fonts and the
shared one-based range grammar.

PNG rendering is limited to eight million pixels per slide for both `convert`
and `render`. A zero-slide PNG conversion fails without creating output.
The exact validation gate corrupts one relationship and requires a nonzero exit,
then requires every verified pinned corpus deck to exit zero without skips.

`thumbnail` renders slide one with deterministic fonts at exactly 320 pixels
wide and preserves the rendered page aspect ratio. Its output defaults through
the shared extension helper. `outline` prints each slide title once, followed
by non-title text paragraphs in recursive shape z-order. Tables use row-major
cell order, paragraph levels add two spaces of indentation, empty text is
omitted, and embedded paragraph breaks become spaces.

Shared range parsing, output-path defaulting, and JSON envelope rules live in
`oxml-cli-support`. Ranges are positive, one-based, comma-separated values and
inclusive ranges. Parsing sorts and deduplicates the result, and rejects more
than 100,000 requested values before expansion. The output helper replaces or
adds only the requested extension. The envelope accepts an object without a
caller-supplied `schema` field and adds the reserved top-level
`{"schema": 1, ...}` contract.

`rdocx-cli` uses the shared envelope for inspect JSON and the shared path helper
for convert defaults. Its flags and zero-based `render --page` compatibility
contract do not change. The `text` command emits paragraphs and table cells in
document order through the facade plain-text representation. Both the selected
page and all-page `render` paths use the bundled-font deterministic facade.
The compiled seven-command surface is covered by one integration binary, with
fixtures constructed in code and no command-only test dependency.
