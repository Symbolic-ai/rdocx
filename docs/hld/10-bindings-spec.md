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
deterministic layouts live in separate `Mutex<Option<Arc<LayoutResult>>>`
caches, with a compile-time regression gate preserving that contract.
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
- **Build wheels with `bundled-fonts` on.** Otherwise `to_pdf()` produces blank
  or mangled text on a bare manylinux container with no system fonts, which
  would be the single most common support question. Roughly 4 MB per wheel is a
  fair trade.

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
via OIDC, with no long-lived token in secrets.

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

### The existing crate is a fork, not a binding

`rdocx-wasm` holds only `CT_Document` and `CT_Styles`. `from_bytes` stores
`package_bytes` and immediately marks it `#[allow(dead_code)]`. `to_docx_bytes`
discards it and rebuilds a package through `oxml_opc::OpcPackage` with the Word
main part, content-type overrides, styles part, and styles relationship
configured at this consumer boundary.

Round-tripping any real document through it **silently destroys** every image
and its relationships, headers and footers, numbering, settings, the theme, the
font table, footnotes and endnotes, core and app properties, every content-type
override, and every relationship except the styles one it re-adds. It has no
tests, no CI job, and `publish = false`, so nothing has ever caught it.

### The fix

```rust
#[wasm_bindgen]
pub struct WasmDocument { inner: rdocx::Document }
```

Everything round-trips immediately, because `to_bytes` flushes into the
**original** package. Three blockers and their answers:

- `Document::open` uses `std::fs`, so expose only `fromBytes` and `toBytes`.
  `save()` is meaningless in a browser anyway.
- `FontManager::new()` loads system fonts and `fontconfig` will not build for
  `wasm32-unknown-unknown`. Add a `system-fonts` feature, default on, off for
  wasm, with `bundled-fonts` on instead. **Then `to_pdf()` works in the
  browser**, which is a genuinely compelling capability that is absent today.
- Watch `getrandom` creep. The workspace already trims `zip` features to avoid
  it.

Keep the existing JS method names so current users do not break. The semantics
only become correct.

**The actual fix is the CI job**: `cargo check --target wasm32-unknown-unknown`
plus `wasm-pack test --node`. The code drifted because nothing was watching.

`rpptx-wasm` wraps the real facade from day one, never a mini-model, in two
profiles: a default without rendering at roughly 600 KB gzipped, and a `render`
build with the rasteriser and bundled fonts at several MB.

## CLIs

`rpptx-cli` mirrors `rdocx-cli`: `inspect`, `text`, `convert`, `diff`,
`replace`, `validate`, `render`, using clap derive and `serde_json` for
`--json`, including the pattern of dispatching `validate` separately so its exit
code carries the verdict.

Two presentation-specific additions: **`thumbnail`**, slide one at a fixed size,
which is what every CMS wants, and **`outline`**, the title and bullet tree,
which is ideal for LLM ingestion and is a genuine differentiator.

`validate` is the highest-value command and pays for itself in the test suite by
running across the corpus in CI.

Shared plumbing, range parsing, output-path defaulting and the JSON envelope,
lives in `oxml-cli-support` rather than being copy-pasted. **Version the JSON
envelope from the first release**: `{"schema": 1, ...}`.
