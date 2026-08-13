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
  rpptx-chart        ChartML model and renderer
  rpptx              the python-pptx-shaped facade, plus assets/default.pptx
  rpptx-cli  rpptx-wasm  rpptx-py
```

## The dependency rule

The graph is acyclic and layered. **Nothing in `oxml-*` may depend on
`rdocx-*` or `rpptx-*`,** with exactly one documented exception below.

```
oxml-core ──┬─→ oxml-drawing ──→ rpptx-oxml ──→ rpptx-layout ──→ rpptx-render
            │         │                                              │
            │         └────────────────→ rdocx-oxml ──→ rdocx-layout │
            ├─→ oxml-opc                                    │        │
            ├─→ oxml-media                                  ↓        ↓
            └─→ oxml-layout ──→ oxml-pdf ←──────────── rdocx-pdf   rpptx
                                                            ↓        ↓
                                                          rdocx   rpptx-cli
```

**The one exception.** `rdocx_oxml::theme::Theme` becomes a thin adapter over
`oxml_drawing::CT_OfficeStyleSheet` (`impl From<&CT_OfficeStyleSheet> for
Theme`), so that `rdocx-layout`'s existing `LayoutInput.theme` field does not
churn. The edge runs `oxml-drawing → rdocx-oxml`, never the reverse.

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
output, font, and line modules are 100 percent docx-free: page frames,
positioned elements, glyph runs, colours, fonts, and owned line parameters.
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

**`rpptx-chart` depends on `oxml-layout` for backend-neutral geometry.** Its
typed ChartML caches lower directly to `PathElement` and `Group` values. The
edge points from the format-specific chart crate to format-neutral layout, and
no PDF or raster backend becomes a chart dependency.

**`rpptx-layout` depends on `rpptx-chart` for native chart projection.** Package
assembly parses scoped ChartML targets, then the resolver freezes a completed
backend-neutral group or a visible fallback in `ResolvedContent`. This edge
stays within the PresentationML family. `rpptx-render` and the format-neutral
backends consume only the frozen group and do not parse ChartML.

## What stays put

`rdocx-oxml` remains a real crate holding roughly 8,700 lines of
WordprocessingML: text, properties, tables, styles, numbering, borders, headers
and footers, footnotes, placeholder replacement, and `drawing.rs`. The
`wp:` inline and anchor code in the latter is Word-only and has no pptx value,
so it is not migrated.

`rdocx-layout` keeps the flow model: the engine, the paginator, blocks, tables
and the style resolver. Slides do not paginate, so none of it transfers. The
flow engine resolves Word relationship IDs to content-addressed `MediaId`
values before pagination, and page output carries the resolved bytes and MIME
type rather than a relationship-scoped placeholder. One `MediaRegistry` per
layout compares complete bytes, assigns deterministic alternate IDs when two
compact keys collide, and is shared by the lower-level layout and pagination
entry points.

## Versioning

The 14 implemented shared and PowerPoint publication candidates carry an
explicit common incubating version of 0.1.2 in their manifests and workspace
pins. The released `rdocx-*` crates continue to use the separate workspace
version. Version preparation and manifest eligibility do not authorize
publication. Registry publication for this family is authorized only when
`/release rpptx-vX.Y.Z` reaches its exact reviewed SHA and receives the
separate final approval at the external mutation boundary. `oxml-cli-support`
is the format-neutral owner of range parsing, JSON envelope, and output-path
contracts. It has no dependency on either document family, while CLI binaries
depend inward on it.

The immutable `rpptx-v0.1.2` release contains the earlier 12-package family.
`oxml-cli-support` and `rpptx-cli` remain unpublished at 0.1.2. F-X006 owns a
future fresh-version release of the complete 14-package family. No existing
tag or registry version is moved or overwritten.

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
