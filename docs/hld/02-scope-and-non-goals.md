# 02, Scope and non-goals

This document decides whether something is in v1. `03-architecture.md` decides
which crate owns it.

## The shape of v1

One release, containing:

1. The `oxml-*` infrastructure extracted from rdocx, with rdocx migrated onto it
   and released as 0.3.0.
2. `rpptx` at feature parity with `python-pptx`, including charts.
3. PDF and PNG rendering of slides.
4. Rust crates, CLIs, WASM modules and Python wheels for both rdocx and rpptx.

There are no partial-feature interim releases. This was a deliberate choice and
its cost is recorded in `00-vision.md`.

## In scope for rpptx v1

### Presentation and slides

| Capability | Notes |
|---|---|
| `Presentation::new / open / from_bytes / save / to_bytes` | `new()` uses a bundled template |
| Slide collection, iteration, indexing, lookup by id | |
| `add_slide(layout)` | Synthesises placeholders, does not deep-copy |
| `remove_slide`, `move_slide`, `duplicate_slide` | Beyond python-pptx |
| Slide size get and set | |
| Slide masters and layouts, layout lookup by name | Read |
| Core, app and custom properties | Shared with rdocx via `oxml-core` |
| Notes slides | Read and write |
| Notes-master and handout-master header and footer settings | Native Rust read and write |
| Ordered presentation sections and slide membership | Native Rust read and write |
| Modern comments, authors and threaded replies | Native Rust read and write. Legacy comments remain opaque |
| Slide background, follow-master-background | |
| Hidden slides | Skipped when rendering, preserved on save |
| Modern package classes | Native Rust reads, preserves, inspects, and output-selects PPTX, PPTM, POTX, POTM, PPSX, and PPSM. Binary `.ppt` remains out of scope |
| OLE, ActiveX, and VBA executable payloads | Native Rust relationship-owned inventory, byte-exact extraction and replacement, and ownership-aware removal. Payloads remain opaque and are never executed. OLE renders from its stored preview image |
| OpenDocument Presentation interchange | Native Rust bounded read and deterministic write for slides, ordinary rectangles and text boxes, tables, embedded images, and speaker notes. Other safe content is reported through stable diagnostics |
| HTML slide content import | Native Rust bounded import of HTML5 documents and fragments into editable slides, explicitly positioned shapes, formatted text, tables, caller-supplied images, and links. Browser layout, scripting, fetching, and unsupported CSS remain diagnostic |
| PDF page content import | Native Rust bounded import into either one preserved full-slide graphic per page or an editable subset of text, raster images, paths, and URI links. Unsupported operators and font substitutions remain diagnostic |

### Shapes

| Capability | Notes |
|---|---|
| `add_textbox`, `add_picture`, `add_table`, `add_shape`, `add_connector`, `add_group_shape` | |
| Shape id, name, type, rotation | |
| Position and size, with placeholder inheritance | `Option`-returning plus an `effective_bounds` accessor |
| Fill, line, shadow | Fill and line full, shadow read-only |
| Adjustment values, `a:avLst` | |
| Click actions and hyperlinks | |
| Placeholders by index and by type | |
| Picture crop and intrinsic size | Via `oxml-media` |
| Image deduplication by content hash on insert | |
| SmartArt inspection, bounded node-text editing, and native rendering for six pinned layouts | Data, layout, style, colour, cached drawing, and relationship ownership are typed. The exact pinned list, hierarchy, cycle, relationship, matrix, and pyramid resources lower through the shared DrawingML engines. Unsupported algorithms and unmodelled XML remain preserved |

### Text

| Capability | Notes |
|---|---|
| Text frame, paragraphs, runs, line breaks | |
| Alignment, level, line spacing, space before and after | |
| Font: bold, italic, underline, strike, size, name, colour, caps, language | |
| Bullets: character, auto-number, none, size percent, colour | python-pptx has no bullet API. This is beyond parity |
| Margins, vertical anchor, word wrap, auto-size | |
| Nine-level list style inheritance | |

### Tables

Rows, columns, cells, cell text and text frames, cell fill and margins,
`merge` and `split`, merge-origin and span queries, and the banding flags.

### Charts

`add_chart` with bar, line, pie, scatter, area, doughnut and radar plots.
Series, categories, axes, gridlines, legend, data labels and number formats.
Each chart writes its own part, its relationship, and an embedded workbook.

### Rendering

Preset and custom geometry, solid, gradient, pattern and picture fills, lines
with dash, cap, join and arrowheads, rotation, flips and nested groups, the full
inheritance chain, shape text with anchoring, insets, wrap, bullets and stored
autofit, tables, connectors, hyperlinks, slide-number fields and backgrounds.

### Distribution

`rpptx` and `rdocx` as crates, `rpptx-cli` and `rdocx-cli`, `rpptx-wasm` and a
rewritten `rdocx-wasm`, and `rdocx-py` and `rpptx-py` wheels on PyPI.

## Explicitly not in v1

Each of these is **preserved verbatim on round-trip**. Nothing in this list
causes data loss, only reduced fidelity when rendering.

| Area | v1 behaviour |
|---|---|
| Animations, transitions, `p:timing` | Preserved, irrelevant to static rendering |
| Unsupported SmartArt algorithms and unmodelled `dgm:` content | Preserved. Rendering uses the drawing fallback part, else its cached picture, else its bounding box. The six exact pinned native layouts are handled before this fallback |
| OLE objects, ActiveX | Preserved, rendered as the stored preview image |
| Video and audio | Preserved, rendered as the poster frame |
| 3-D, `a:scene3d` and `a:sp3d` | Preserved, rendered flat |
| Blur on shadows, glow, reflection, soft edges | Shadow renders as a hard offset silhouette. The rest are dropped |
| WordArt text warp, `a:prstTxWarp` | Rendered as plain unwarped text |
| EMF and WMF images | Outline placeholder. Writing an EMF interpreter is out of scope |
| `eaVert` upright stacked CJK | Falls back to rotated vertical text |
| `mongolianVert` upright stacking | Falls back to rotated vertical-270 text |
| `wordArtVert` and `wordArtVertRtl` glyph stacking | Fall back to rotated vertical and vertical-270 text respectively |
| Gradient stop alpha | Stop colour composited, alpha dropped |
| Justified text inside shapes | Treated as left-aligned |
| Table cell text autofit | Not attempted |
| Legacy comments, ink, `p:contentPart` | Preserved |

Every one of these records a diagnostic, surfaced by `rpptx inspect --json` and
by the render API, so a user can tell approximation from fidelity.

## Non-goals, permanently

**`oxml-sml` is not a spreadsheet library.** It writes one worksheet with the
cells a chart needs. It is not a foundation for an `rxlsx` and should not grow
into one without a separate decision.

**Drop-in `python-docx` and `python-pptx` compatibility is not promised.** Those
libraries' real-world surface is inseparable from lxml, and a large fraction of
production code reaches through `._p`, `._r` and `qn()`. Source compatibility
is bounded to the completed public binding surface. The rdocx gate pins the
seventeen executable python-docx 1.2.0 documentation examples that fit the S33
API to stable tagged sources. Sixteen change only their import namespace. The
Quickstart held-row example re-fetches the row through the public document path
before its second cell assignment because the first structural text replacement
intentionally stales every pre-write handle under strict global revision.
Touching a private lxml-shaped attribute raises a clear error naming the
equivalent, rather than failing five frames away.

**Not a PowerPoint clone.** The renderer targets business decks built from
stock or corporate templates. Decks that lean on 3-D, heavy effects or WordArt
will render legibly but not faithfully, and will say so.

## Beyond v1

v1 shipped. This section records what changed after it, and it is the only place
a v1 non-goal may be superseded. A non-goal not named here still stands.

The shape of the roadmap is in `14-development-backlog.md`, M14 through M24. The
principle behind it: v1 proved the model and the renderer can live in one
codebase, which is the thing no other library in Python or Rust has. Everything
after v1 leans on that rather than away from it.

Modern Transitional OfficeMath is part of the post-v1 Word authoring surface.
Native Rust callers can inspect, mutate, and author inline and display
equations through the normalized Word model. Legacy Equation Editor, OLE, and
pre-OOXML equation payloads remain opaque under the permanent legacy-format
boundary.

The native Rust Word facade inventories relationship-owned OLE objects,
ActiveX controls, and VBA projects without decoding or executing their
payloads. Callers can extract and replace exact bytes or remove one validated
owner while shared targets and unrelated producer content survive. Package and
VBA signature evidence is either retained as explicitly invalidated evidence or
removed through an explicit mutation policy. Python, WASM, CLI, binary `.doc`,
payload decoding, and execution remain outside this surface.

Native Rust callers can also import and export the supported normalized
equation subset as Presentation MathML or LaTeX. Lossy format and OfficeMath
properties remain visible through ordered diagnostics. Python, WASM, CLI,
legacy equation formats, and a second conversion model remain outside this
surface.

M23 makes from-scratch business-document generation a tested native Rust
surface. Its completion gate starts from `Document::new()`, uses only public
`rdocx` APIs, and reproduces the required structure and rendering of five
private reference documents without a base template or raw XML injection. The
private corpus is evidence, not distributable product data. F-240 may reshape
the provisional story plan when its property-level audit finds an uncovered
gap.

M24 extends that result to a complete modern DOCX authoring matrix. It covers
modeled paragraph, run, table, section, story, field, numbering, form,
collaboration, drawing, package-extension, accessibility, conformance,
determinism, resource-limit, and binding surfaces. Complete means that every
in-scope modeled capability is publicly authorable or is explicitly classified
as preservation-only or a permanent non-goal. It does not promise execution of
VBA, ActiveX, embedded applications, proprietary cloud services, or unknown
future producer extensions.

## Modern DOCX capability matrix

This is the closed authoring contract for M23 and M24. Each row is one public
capability or property family. `Y` means the operation is complete today. `P`
means only part of the family is modeled or public. `PV` means input is retained
without a modeled authoring surface. `N` means unsupported, `NA` means the
operation does not apply, and `B` means the binding deliberately exposes only a
narrower boundary. Determinism is `NA` for encrypted bytes because fresh
cryptographic randomness is required. Story placement uses `body`, `related`,
`all`, `package`, or `NA`. Layout and render use `Y`, `P`, `N`, or `NA` with the
same meanings.

An owner is required for every `partial` or `unsupported` row. A
`preserve-only` or `permanent-non-goal` row is closed by its evidence instead.
Public OXML types do not count as facade authoring. Save and reopen means that
modeled state returns through the public `Document` surface, not merely that raw
bytes remain in the ZIP package.

| Capability ID | Family | Capability or property | Create | Read | Mutate | Remove | Save-reopen | Story | Layout | Render | Determinism | Native | Python | WASM | CLI | Classification | Evidence | Owner |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| DOCX-001 | package | DOCX open, save, and byte serialization | Y | Y | Y | NA | Y | package | Y | Y | Y | Y | Y | Y | Y | complete | implementation:`crates/rdocx/src/document.rs:1955` | - |
| DOCX-002 | package | bounded and encrypted package input and output | Y | Y | Y | NA | Y | package | Y | Y | NA | Y | B | B | B | complete | implementation:`crates/rdocx/src/document.rs:1962` | - |
| DOCX-003 | package | blank Word-compatible package profiles | Y | Y | Y | NA | Y | package | Y | Y | Y | Y | B | B | B | complete | implementation:`crates/rdocx/src/document.rs:2057` | - |
| DOCX-004 | package | DOCM, DOTX, and DOTM identity and output selection | Y | Y | Y | NA | Y | package | Y | Y | Y | Y | B | B | B | complete | implementation:`crates/rdocx/src/document.rs:2064` | - |
| DOCX-005 | properties | core document properties | P | Y | P | P | Y | package | NA | NA | P | Y | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:4355` | F-244 |
| DOCX-006 | properties | application and custom properties | N | Y | N | N | PV | package | NA | NA | N | P | B | B | B | unsupported | boundary:F-244 | F-244 |
| DOCX-007 | properties | document variables, compatibility facts, and defaults | N | P | N | N | PV | package | P | P | N | P | B | B | B | unsupported | boundary:F-244 | F-244 |
| DOCX-008 | conformance | public and private authoring conformance gate | Y | Y | NA | NA | Y | all | Y | Y | Y | Y | B | B | Y | complete | implementation:`scripts/docx_authoring_conformance.py:388` | - |
| DOCX-009 | theme-font | themes and theme selection | N | Y | N | N | PV | package | Y | Y | P | P | B | B | B | partial | boundary:F-245 | F-245 |
| DOCX-010 | theme-font | font table and licensed embedded fonts | N | P | N | N | PV | package | P | P | N | P | B | B | B | partial | boundary:F-245 | F-245 |
| DOCX-011 | styles | paragraph, character, and table style graphs | P | Y | P | N | Y | package | Y | Y | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:4082` | F-246 |
| DOCX-012 | numbering | complete numbering levels and instances | P | P | P | N | P | package | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:3945` | F-247 |
| DOCX-013 | numbering | style-linked counters, restarts, TOC, and REF | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-248 | F-248 |
| DOCX-014 | package | deterministic identifiers across owned parts | Y | Y | Y | NA | Y | package | NA | NA | Y | Y | B | B | B | complete | implementation:`crates/rdocx/src/document.rs` | - |
| DOCX-015 | sections | ordered section lookup, insertion, mutation, and removal | P | P | P | N | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:4233` | F-250 |
| DOCX-016 | sections | page geometry and ordinary section properties | P | Y | P | N | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:4247` | F-251 |
| DOCX-017 | stories | per-section default, first, and even headers and footers | P | P | P | N | P | related | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:3369` | F-252 |
| DOCX-018 | stories | common content location and traversal | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-253 | F-253 |
| DOCX-019 | stories | arbitrary insert, move, clone, and remove | P | Y | P | P | P | body | P | P | P | P | B | B | P | partial | implementation:`crates/rdocx/src/document.rs:2940` | F-254 |
| DOCX-020 | package | part-scoped images, links, and relationships | P | P | P | N | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:3019` | F-255 |
| DOCX-021 | stories | transactional cross-document fragment import | P | Y | P | NA | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:4409` | F-256 |
| DOCX-022 | tables | table properties, grids, widths, borders, and layout mode | P | Y | P | P | P | body | P | P | P | P | P | B | B | partial | implementation:`crates/rdocx/src/table.rs:46` | F-257 |
| DOCX-023 | tables | row and cell height, merge, margins, borders, and flow | P | Y | P | P | P | body | P | P | P | P | P | B | B | partial | implementation:`crates/rdocx/src/table.rs:266` | F-258 |
| DOCX-024 | tables | container measurement and equal-height layout | N | NA | NA | NA | NA | all | N | N | N | N | N | N | N | unsupported | boundary:F-259 | F-259 |
| DOCX-025 | run | ordered text, tabs, breaks, fields, and drawings | P | Y | P | P | P | body | P | P | P | P | P | B | B | partial | implementation:`crates/rdocx/src/run.rs:272` | F-260 |
| DOCX-026 | stories | rich HTML fragments in arbitrary containers | P | NA | NA | NA | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/html.rs:34` | F-261 |
| DOCX-027 | drawing | M23 pictures, text boxes, and watermarks | P | P | P | N | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:3436` | F-262 |
| DOCX-028 | fields | M23 pagination fields and private corpus gate | N | P | N | N | PV | all | N | N | N | P | B | B | B | unsupported | boundary:F-263 | F-263 |
| DOCX-029 | paragraph | ordinary text, alignment, spacing, indentation, and pagination | Y | Y | Y | Y | Y | body | Y | Y | Y | Y | Y | B | B | complete | implementation:`crates/rdocx/src/paragraph.rs:279` | - |
| DOCX-030 | paragraph | borders, shading, tabs, frames, direction, and mark properties | P | Y | P | P | P | body | P | P | P | P | B | B | B | partial | boundary:F-264 | F-264 |
| DOCX-031 | run | fonts, emphasis, color, language, and ordinary inline content | Y | Y | Y | Y | Y | body | Y | Y | Y | Y | Y | B | B | complete | implementation:`crates/rdocx/src/run.rs:367` | - |
| DOCX-032 | run | theme fonts, complex script, effects, symbols, and special content | P | Y | P | P | P | body | P | P | P | P | B | B | B | partial | boundary:F-265 | F-265 |
| DOCX-033 | paragraph | bidirectional, East Asian, vertical, ruby, and phonetic text | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-266 | F-266 |
| DOCX-034 | tables | table styles and conditional formatting | P | P | P | N | P | all | P | P | P | P | B | B | B | partial | boundary:F-267 | F-267 |
| DOCX-035 | tables | floating, bidirectional, autofit, and advanced table layout | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-268 | F-268 |
| DOCX-036 | sections | borders, columns, line numbers, book fold, and note policy | P | P | P | N | P | body | P | P | P | P | B | B | B | partial | boundary:F-269 | F-269 |
| DOCX-037 | properties | complete settings and web settings authoring | P | P | P | P | P | package | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:4318` | F-270 |
| DOCX-038 | stories | uniform rich header and footer editing | P | P | P | P | P | related | P | P | P | P | B | B | B | partial | boundary:F-271 | F-271 |
| DOCX-039 | stories | rich footnotes | P | P | P | N | P | related | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:2738` | F-272 |
| DOCX-040 | stories | rich endnotes | N | P | N | N | PV | related | P | P | N | P | B | B | B | unsupported | boundary:F-273 | F-273 |
| DOCX-041 | stories | note separators, markers, numbering, and restart policy | N | P | N | N | PV | related | P | P | N | P | B | B | B | unsupported | boundary:F-274 | F-274 |
| DOCX-042 | stories | bookmarks, paired ranges, and annotations | P | Y | P | P | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/comments.rs:44` | F-275 |
| DOCX-043 | stories | complete fragment dependency and conflict policy | P | P | P | NA | P | all | P | P | P | P | B | B | B | partial | boundary:F-276 | F-276 |
| DOCX-044 | stories | glossary and building-block creation and insertion | N | Y | P | N | P | related | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/building_block.rs:14` | F-277 |
| DOCX-045 | fields | simple and complex field builder | P | Y | P | P | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/run.rs:187` | F-278 |
| DOCX-046 | fields | page and section field materialization across stories | N | P | N | N | PV | all | N | N | N | P | B | B | B | unsupported | boundary:F-279 | F-279 |
| DOCX-047 | fields | captions, sequences, and complete cross-references | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-280 | F-280 |
| DOCX-048 | fields | indexes, tables of figures, and authorities | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-281 | F-281 |
| DOCX-049 | fields | citations and bibliography | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-282 | F-282 |
| DOCX-050 | fields | numbering-aware navigation fields | P | P | P | P | P | all | P | P | P | P | B | B | B | partial | boundary:F-283 | F-283 |
| DOCX-051 | stories | stable container-wide template grammar | P | NA | P | NA | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:4837` | F-284 |
| DOCX-052 | forms | content control creation and lifecycle | N | Y | P | P | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/content_control.rs:27` | F-285 |
| DOCX-053 | forms | rich, repeating, and typed content controls | N | P | P | N | P | body | P | P | P | P | B | B | B | partial | boundary:F-286 | F-286 |
| DOCX-054 | forms | custom XML stores and two-way data binding | N | P | N | N | PV | package | P | P | N | P | B | B | B | unsupported | boundary:F-287 | F-287 |
| DOCX-055 | forms | legacy text, checkbox, and drop-down field creation | N | Y | P | N | P | all | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/field.rs:30` | F-288 |
| DOCX-056 | forms | high-level modern form composition | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-289 | F-289 |
| DOCX-057 | forms | mail-merge package and data-source authoring | P | Y | P | P | P | package | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/field.rs:617` | F-290 |
| DOCX-058 | collaboration | tracked insertion and deletion | N | Y | P | P | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/revision.rs:19` | F-291 |
| DOCX-059 | collaboration | property revisions and move ranges | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-292 | F-292 |
| DOCX-060 | collaboration | comments, replies, people, and modern metadata | P | Y | P | P | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/comments.rs:77` | F-293 |
| DOCX-061 | collaboration | permission ranges and protection composition | N | P | P | P | P | all | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:2812` | F-294 |
| DOCX-062 | collaboration | comparison as complete tracked revisions | P | Y | P | NA | P | all | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/comparison.rs:70` | F-295 |
| DOCX-063 | collaboration | deterministic identity and time policy | N | NA | N | NA | N | package | NA | NA | N | N | N | N | N | unsupported | boundary:F-296 | F-296 |
| DOCX-064 | drawing | picture anchors, wrapping, crop, transforms, and effects | P | P | P | P | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:3146` | F-297 |
| DOCX-065 | drawing | shapes, text boxes, groups, and connectors | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-298 | F-298 |
| DOCX-066 | drawing | AlternateContent, VML, and SVG compatibility writing | P | P | P | N | P | all | P | P | P | P | B | B | B | partial | boundary:F-299 | F-299 |
| DOCX-067 | drawing | charts at every valid Word insertion point | P | P | P | N | P | body | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:3053` | F-300 |
| DOCX-068 | drawing | SmartArt and diagram authoring | N | P | N | N | PV | all | P | P | N | P | B | B | B | unsupported | boundary:F-301 | F-301 |
| DOCX-069 | drawing | embedded objects, icons, and alternative-format parts | N | Y | P | P | Y | all | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/embedded.rs:67` | F-302 |
| DOCX-070 | drawing | complete drawing and embedded-content layout | N | P | N | NA | NA | all | P | P | N | P | B | B | B | unsupported | boundary:F-303 | F-303 |
| DOCX-071 | extensions | typed safe custom-part and relationship facade | N | P | N | N | PV | package | NA | NA | N | P | B | B | B | unsupported | boundary:F-304 | F-304 |
| DOCX-072 | extensions | templates, web extensions, task panes, and web settings | N | P | N | N | PV | package | NA | NA | N | P | B | B | B | unsupported | boundary:F-305 | F-305 |
| DOCX-073 | extensions | attach opaque VBA, ActiveX, OLE, and custom UI payloads | N | Y | P | Y | Y | package | NA | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/embedded.rs:44` | F-306 |
| DOCX-074 | accessibility | complete authoring and structural audit | P | P | P | P | P | all | P | P | P | P | B | B | B | partial | implementation:`crates/rdocx/src/document.rs:6895` | F-307 |
| DOCX-075 | conformance | stable modeled, preserved, unsupported, and lossy diagnostics | P | P | NA | NA | NA | all | P | P | P | P | B | B | P | partial | implementation:`crates/rdocx/src/document.rs:112` | F-308 |
| DOCX-076 | conformance | Strict and Transitional validation and repair-free output | P | P | NA | NA | P | package | P | P | P | P | B | B | P | partial | boundary:F-309 | F-309 |
| DOCX-077 | operations | byte determinism, resource limits, cancellation, and stability | P | P | P | P | P | all | Y | Y | P | P | P | P | P | partial | boundary:F-310 | F-310 |
| DOCX-078 | operations | native, Python, WASM, and CLI capability classification | P | Y | NA | NA | NA | all | NA | NA | P | P | P | P | P | partial | boundary:F-310 | F-310 |
| DOCX-079 | run | Transitional OfficeMath authoring and conversion | Y | Y | Y | Y | Y | all | Y | Y | Y | Y | B | B | B | complete | implementation:`crates/rdocx/src/math.rs:39` | - |
| DOCX-080 | package | unknown unmodeled safe producer XML | N | PV | N | N | PV | all | P | P | Y | PV | B | B | B | preserve-only | boundary:`docs/hld/03-architecture.md` preservation contract | - |
| DOCX-081 | extensions | unknown future producer extensions | N | PV | N | N | PV | package | NA | NA | Y | PV | B | B | B | preserve-only | boundary:`docs/hld/04-opc-and-packaging.md` loss-free retention | - |
| DOCX-082 | package | binary DOC, Word 2003 XML, and pre-OOXML payloads | N | N | N | N | N | NA | N | N | NA | N | N | N | N | permanent-non-goal | non-goal:legacy format engine | - |
| DOCX-083 | extensions | VBA, ActiveX, OLE, add-in, and embedded application execution | N | N | N | N | N | NA | N | N | NA | N | N | N | N | permanent-non-goal | non-goal:executable payload execution | - |
| DOCX-084 | operations | proprietary cloud services and hosted collaboration | N | N | N | N | N | NA | N | N | NA | N | N | N | N | permanent-non-goal | non-goal:hosted service execution | - |
| DOCX-085 | drawing | exact 3-D, heavy effects, and proprietary WordArt rendering | N | PV | N | N | PV | all | P | N | Y | PV | B | B | B | permanent-non-goal | non-goal:Word rendering clone | - |

The public facade and modeled-property audit covers the `rdocx` exports, the
WordprocessingML paragraph, run, table, section, settings, styles, numbering,
fields, comments, revisions, content-control, drawing, and OfficeMath models,
the package relationship owners, layout and render entry points, and the native,
Python, WASM, and CLI surfaces. The audit found no capability that requires a
second document model or an additional story beyond F-243 through F-310.

Bounded MHTML import and export is part of the post-v1 native Word interchange
surface. It carries the supported document structure, contained PNG and JPEG
resources, safe links, and ordered loss diagnostics through the existing Word
model. Conversion never fetches a network or filesystem resource. Binary
`.doc`, executable web content, unrestricted MIME processing, and new Python,
WASM, or CLI entry points remain outside this scope.

Modern Word package identity and Flat OPC interchange are also native Rust
surfaces. `Document` reads, preserves, inspects, and output-selects DOCX, DOCM,
DOTX, and DOTM from the exact main-part content type. Flat OPC import is bounded
and strict, and export preserves relationship-owned executable and opaque
payloads without executing them. Binary `.doc` and new Python, WASM, or CLI
entry points remain outside this scope.

Modern OOXML legacy form fields and glossary entries are part of the post-v1
native Word surface. Native Rust callers can inventory supported form fields
across internal Word stories, update their typed values, and replace existing
AutoText and building-block entries. Binary `.doc` input, field execution,
implicit entry expansion, new glossary authoring, and additional binding
surfaces remain outside this scope.

### Superseded

| v1 position | Superseded by | Why it changed |
|---|---|---|
| Charts are a PowerPoint capability | **M15** | `oxml-chart` now owns the format-neutral engine. `rpptx-chart` remains a deprecated compatibility shim |
| Animations, transitions, and `p:timing` are preserved but never executed | **M21** | The static renderer and corpus now provide the geometry, timing-independent frame state, and output backends needed to add bounded timeline execution without making it a prerequisite for ordinary slide rendering |
| Video and audio are preservation-only poster content | **M21** | The native package model edits embedded or linked media, and the additive media-aware timeline path returns poster or labelled fallback output with synchronized playback state while static rendering remains poster-only |

These entries are decisions, not corrections. The v1 positions were right when
they were written.

Bounded timeline execution is additive to ordinary presentation rendering.
Native callers select a slide-local elapsed time and click count, then receive
one deterministic page frame, the evaluated frame state, and ordered
diagnostics. Supported entrance, exit, emphasis, motion, transition, and
explicit-name morph cases execute without making timing a prerequisite for
static rendering. Unsupported or malformed timing stays visible through
diagnostics and does not acquire guessed behavior.

Audio and video editing is a package operation, not playback or decoding.
Native callers can inspect and atomically mutate media sources, poster images,
relationships, and bounded playback settings. Unknown safe payloads remain
extractable and diagnostic. Static rendering continues to use the poster.
The additive deterministic media timeline facade returns the ordinary timeline
frame with ordered audio and video playback states. A valid poster uses the
existing image path. An unresolved poster can become a deterministic labelled
fallback or a closed error according to explicit caller policy. Media payloads
remain outside renderer image input, and no codec is decoded.

Native callers can export explicit slide segments as deterministic animated
GIF or Motion JPEG AVI. Each segment declares its duration, fixed click count,
and optional outgoing transition slide. Sampling uses bounded integer
millisecond timestamps at a declared frame rate, reuses one prepared package,
resolver, and media context, and renders one opaque frame at a time. GIF loop
metadata and cumulative centisecond timing are explicit. AVI quality, frame
rate, dimensions, duration, chunks, and index are deterministic. Fixed frame,
pixel, and byte caps fail closed, and no system codec or subprocess is used.

### Conditional expansion

`oxml-sml` remains chart-workbook support rather than a spreadsheet library.
M19 may supersede that position only if F-184 finds a material gap still exists
in the Rust ecosystem at S81. A basic reader, writer, or formula evaluator is
not enough. The required gap is one loss-aware lifecycle covering advanced
editing, calculation, local pivot refresh, selected Power Query execution,
Office Scripts-compatible automation, and rendering. If a credible maintained
crate provides that boundary by then, M19 is archived rather than implemented.

### Still non-goals, and still permanent

- **Not a PowerPoint clone, and not a Word rendering clone.** The renderers
  target business documents. M24 completes the declared modern DOCX authoring
  matrix, but documents that lean on unsupported 3-D, heavy effects, or
  proprietary WordArt rendering remain diagnostic rather than falsely exact.
- **Drop-in `python-docx` and `python-pptx` compatibility is not promised.**
  Unchanged and for the unchanged reason: their real surface is inseparable from
  lxml.
- **EMF and WMF interpretation.** Still an outline placeholder. M18 adds
  formats, and this is not one of them.
- **Legacy binary Office formats.** Binary `.doc`, `.xls`, and `.ppt`, Word 2003
  XML, and equivalent pre-OOXML authoring surfaces are not scheduled. They do
  not share the OOXML package, model, preservation, or rendering foundations,
  and adding them would create separate legacy engines rather than deepen the
  current product.
- **Universal Excel service compatibility.** If M19 proceeds, it executes
  worksheet and table-backed pivots, a declared Power Query M and connector
  subset, and an explicitly versioned Office Scripts-compatible API. It
  preserves and reports unsupported OLAP and Power Pivot execution, proprietary
  and tenant-bound connectors, custom functions, Python cells, VBA, XLM, and
  Microsoft-hosted storage or automation services without claiming to run them.

### The WASM packages are deliberately unpublished

`@tensorbee/rdocx-wasm` and `@tensorbee/rpptx-wasm` are built, optimised,
packed as bundler tarballs and install-tested on every pull request. **They are
not published to npm, and npm publication is not authorised.**

This is enforced rather than intended. The WASM CI job is asserted to contain
none of `npm publish`, `wasm-pack publish`, `npm login`, `npm adduser`,
`npm token`, `NODE_AUTH_TOKEN`, `NPM_TOKEN`, `--registry`, `id-token:`,
`git tag` or `gh release`, so a step that could publish cannot be added without
failing the release preflight.

Both crates are `publish = false` for crates.io and inherit their Rust family's
version. That inheritance is harmless while nothing ships, and it is the only
thing that would need revisiting on the day npm publication is authorised.
F-X030 was filed against that inheritance and archived once this position was
confirmed, and its entry records what the work would be if the position changes.

### Deliberately not scheduled

Named so a reader knows they were considered rather than missed.

- **Legacy binary `.doc`, `.xls` and `.ppt`.** Each is a compound-file format
  with no relation to OOXML, and each would require a separate legacy engine.
  They remain excluded after the modern Office depth milestones complete.
- **A collaborative editing server.** Out of the shape of a library.

## The measurable bar

For a business deck built from a stock Office template, with title and content
slides, bullets, tables, images, theme colours and a gradient title bar, a
150 dpi PNG should be indistinguishable from PowerPoint's own export at a
glance: text baselines within about one point, shape edges exact, colours exact.

CI compares the pinned 50-deck corpus with LibreOffice's render and records at
least 0.95 SSIM on at least 80 percent of slides as a trend reference. The hard
automatic gate requires every slide to render without panic, missing output,
dimension mismatch, or a dropped bounded shape. LibreOffice is the CI oracle
only because PowerPoint is not scriptable on runners, so SSIM regressions are
review-required rather than automatic failures. A pinned native PowerPoint
representative review is the hard manual fidelity gate.
