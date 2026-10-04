# Changelog

## Unreleased

No changes have been recorded after the prepared family sections below.

## v0.15.0

### Highlights

The seven stable Word Rust crates, the `rdocx` CLI, and the `rdocx` Python
distribution move together to 0.15.0 from the reviewed S86 source. One
release carries six CLI archives, six `cp39-abi3` wheels, one source
distribution, and a `SHA256SUMS` covering all thirteen payloads. Each
payload has build provenance verifiable with `gh attestation verify FILE -R
tensorbee/rdocx`. This answers
[@hadim](https://github.com/hadim)'s
[Issue 266](https://github.com/tensorbee/rdocx/issues/266) request for one
current-main release with matching CLI and Python versions.

### Added

- Word authoring now covers rich content and comments in all six header and
  footer variants, together with rich footnotes and endnotes whose references,
  relationships, pictures, hyperlinks, and unmodelled XML survive save and
  reopen. High-level style formatting covers alignment, spacing, indentation,
  font, size, bold, and colour for
  [Issue 264](https://github.com/tensorbee/rdocx/issues/264).
- Python editing exposes tables, sections, styles, bookmarks, fields, story
  content, pictures, hyperlinks, tracked revisions, and counted replacement.
  The CLI can inspect and compare related stories, anchor comments on text,
  select accepted or tracked revision views, and emit structured results.
  These surfaces incorporate
  [PR 176](https://github.com/tensorbee/rdocx/pull/176),
  [PR 186](https://github.com/tensorbee/rdocx/pull/186),
  [PR 187](https://github.com/tensorbee/rdocx/pull/187),
  [PR 194](https://github.com/tensorbee/rdocx/pull/194),
  [PR 198](https://github.com/tensorbee/rdocx/pull/198),
  [PR 201](https://github.com/tensorbee/rdocx/pull/201),
  [PR 203](https://github.com/tensorbee/rdocx/pull/203),
  [PR 204](https://github.com/tensorbee/rdocx/pull/204),
  [PR 212](https://github.com/tensorbee/rdocx/pull/212),
  [PR 220](https://github.com/tensorbee/rdocx/pull/220),
  [PR 236](https://github.com/tensorbee/rdocx/pull/236), and
  [PR 256](https://github.com/tensorbee/rdocx/pull/256).
- Native Word story traversal and replacement reach content controls, text
  boxes, footnotes, endnotes, tracked insertions, simple fields, smart tags,
  and custom XML wrappers. This incorporates
  [PR 177](https://github.com/tensorbee/rdocx/pull/177),
  [PR 179](https://github.com/tensorbee/rdocx/pull/179),
  [PR 180](https://github.com/tensorbee/rdocx/pull/180),
  [PR 191](https://github.com/tensorbee/rdocx/pull/191),
  [PR 195](https://github.com/tensorbee/rdocx/pull/195),
  [PR 202](https://github.com/tensorbee/rdocx/pull/202),
  [PR 210](https://github.com/tensorbee/rdocx/pull/210), and
  [PR 211](https://github.com/tensorbee/rdocx/pull/211).

### Fixed

- The `rdocx` CLI runs comparison and other document commands on an eight MiB
  stack on Windows, avoiding a stack overflow in the complete Word workflow.
- Editing Word parts writes compact XML without repeating canonical namespace
  declarations on every retained paragraph. It preserves producer aliases,
  root attributes, unknown content, and unchanged package parts, building on
  [PR 154](https://github.com/tensorbee/rdocx/pull/154) and
  [PR 251](https://github.com/tensorbee/rdocx/pull/251) for
  [Issue 245](https://github.com/tensorbee/rdocx/issues/245).
  Newly created documents now carry Word-compatible application metadata, and
  nested tables retain their required trailing paragraph.
- Word layout uses the governing section for each paragraph and table. Line
  height, inline picture height, tab stops, TOC entries, page fields,
  keep-with-next chains, row minimum heights, and terminal spaces follow the
  reviewed Word cases. This incorporates
  [PR 148](https://github.com/tensorbee/rdocx/pull/148),
  [PR 199](https://github.com/tensorbee/rdocx/pull/199),
  [PR 200](https://github.com/tensorbee/rdocx/pull/200),
  [PR 225](https://github.com/tensorbee/rdocx/pull/225),
  [PR 237](https://github.com/tensorbee/rdocx/pull/237),
  [PR 241](https://github.com/tensorbee/rdocx/pull/241), and
  [PR 242](https://github.com/tensorbee/rdocx/pull/242).
  [PR 265](https://github.com/tensorbee/rdocx/pull/265) supplied independent
  mixed-section regression coverage.
- Comparison now records changed picture bytes and final table or paragraph
  changes as revisions that survive acceptance and rejection. Accepted text,
  HTML, Markdown, JSON, PDF layout, and media traversal omit deleted table
  rows and join paragraphs whose marks were deleted. This incorporates
  [PR 228](https://github.com/tensorbee/rdocx/pull/228),
  [PR 229](https://github.com/tensorbee/rdocx/pull/229),
  [PR 257](https://github.com/tensorbee/rdocx/pull/257),
  [PR 258](https://github.com/tensorbee/rdocx/pull/258),
  [PR 259](https://github.com/tensorbee/rdocx/pull/259),
  [PR 260](https://github.com/tensorbee/rdocx/pull/260),
  [PR 261](https://github.com/tensorbee/rdocx/pull/261),
  [PR 262](https://github.com/tensorbee/rdocx/pull/262), and
  [PR 263](https://github.com/tensorbee/rdocx/pull/263).
- CLI and package saves refuse unintended overwrite and stage replacement
  atomically. Edited border and namespace declarations, shared-run field
  text, and XML-valid characters survive their supported read and write paths.
  This incorporates
  [PR 174](https://github.com/tensorbee/rdocx/pull/174),
  [PR 178](https://github.com/tensorbee/rdocx/pull/178),
  [PR 185](https://github.com/tensorbee/rdocx/pull/185),
  [PR 197](https://github.com/tensorbee/rdocx/pull/197),
  [PR 232](https://github.com/tensorbee/rdocx/pull/232),
  [PR 233](https://github.com/tensorbee/rdocx/pull/233), and
  [PR 239](https://github.com/tensorbee/rdocx/pull/239).

### Compatibility

The exact seven publishable stable crates move from 0.14.0 to 0.15.0:
`rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
`rdocx-pdf`, `rdocx`, and `rdocx-cli`. Update their pins together.
They use the separately prepared shared OOXML 0.13.0 family. The Python
distribution and import name stay `rdocx`, and its 0.15.0 wheels require
Python 3.9 or newer. Historical `v0.14.0` and `py-rdocx-v0.14.0`
remain unchanged. The unpublished `rdocx-wasm` and npm package are outside
this crates.io and PyPI release.

This pre-1.0 minor release has source-incompatible Rust model changes.
`rdocx_layout::PageGeometry` was `Copy` in 0.14.0 and is now `Clone`
because it owns column tracks and optional page metadata. Pass it by
reference or call `.clone()` where a copy was previously implicit. Full
`PageGeometry` struct literals must supply `columns`,
`column_separator`, `page_borders`, `line_numbers`,
`vertical_alignment`, `mirror_margins`,
`do_not_use_html_paragraph_auto_spacing`, and `body_rotation`.
Public `GlyphRun`, `LineBreakParams`, and `LayoutInput` also gained
fields. Update full literals and exact-output fixtures when upgrading.
`rdocx replace` now refuses input-as-output and existing output paths.
Scripts that overwrote a source file must choose a new output path.

### Contributors

[@hadim](https://github.com/hadim), Hadrien Mary, reported
[Issue 266](https://github.com/tensorbee/rdocx/issues/266) and
[Issue 245](https://github.com/tensorbee/rdocx/issues/245). The linked Word,
Python, CLI, layout, comparison, and preservation PRs above supplied
reference behavior that was reconciled or hardened in maintainer commits.
Those GitHub PRs were closed without direct merges.
[@set2374](https://github.com/set2374) supplied the independent
[PR 265](https://github.com/tensorbee/rdocx/pull/265) mixed-section regression.
[@Andrewkoro105](https://github.com/Andrewkoro105) requested the
[Issue 264](https://github.com/tensorbee/rdocx/issues/264) high-level style
API. Atul Sharma reviewed and integrated the release family.

## rpptx-v0.13.0

### Highlights

The shared OOXML and PowerPoint Rust crates, `rpptx` CLI, and `rpptx` Python
distribution move together to 0.13.0. One family release carries six CLI
archives, six `cp39-abi3` wheels, one source distribution, and a `SHA256SUMS`
covering all thirteen payloads. Each payload has build provenance that can be
checked with `gh attestation verify FILE -R tensorbee/rdocx`. This answers
[@hadim](https://github.com/hadim)'s
[Issue 266](https://github.com/tensorbee/rdocx/issues/266) request for a
current release from the reviewed main merge with matching CLI and Python
versions.

### Added

- Native and Python presentation editing now covers retained run handles,
  inherited shape geometry, table rows and cells, picture crop,
  grouping, shape hyperlinks, and text-range comment anchors.
- Shape authoring covers slide jumps, line dash and ends, outer shadows,
  preset geometry replacement, and theme effect selection. Cross-deck slide
  import carries supported media and notes. Counted replacement can target a
  single slide or text frame, and built-in table styles render their
  backgrounds.
- The CLI can add comments and report speaker notes. Text, outline, and
  inspection commands have JSON forms for automation.

### Fixed

- The `rpptx` CLI runs validation and other presentation commands on an eight
  MiB stack on Windows, avoiding a stack overflow in the complete deck chain.
- DrawingML text edits retain unmodelled body properties and turn assigned
  line feeds into paragraphs while preserving line separators in layout.
  Newly authored shapes receive theme style references in schema order.
- Presentation PDF output paints slide backgrounds, preserves supported
  producer paragraph properties, and keeps searchable text paired with its
  glyphs. Rich and plain line layout follow the reviewed spacing and line
  break cases.

### Compatibility

The exact 15 publishable shared OOXML and PowerPoint crates move from 0.12.1
to 0.13.0 together. They are `oxml-core`, `oxml-opc`, `oxml-media`,
`oxml-layout`, `oxml-drawing`, `oxml-pdf`, `oxml-sml`, `oxml-cli-support`,
`oxml-chart`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`, `rpptx-render`,
`rpptx`, and `rpptx-cli`. Rust callers should update every shared internal
crate pin together. This pre-1.0 minor release adds authored-shape and
presentation APIs and changes the output of affected shape, text, and PDF
operations. `oxml-layout::LineBreakParams` gained required public fields, so
Rust callers constructing it with a struct literal must update those
literals. Review exact-output fixtures and exhaustive use of public models.

The Python distribution and import name remain `rpptx`. The 0.13.0 wheels
require Python 3.9 or newer. Stable Word crates, `rdocx` Python, and the
unpublished `rpptx-wasm` and npm packages are outside this tag. Historical
`rpptx-v0.12.1` and `py-rpptx-v0.12.1` remain unchanged.

### Contributors

[@hadim](https://github.com/hadim), Hadrien Mary, reported
[Issue 266](https://github.com/tensorbee/rdocx/issues/266) and contributed the
presentation changes in
[PR 173](https://github.com/tensorbee/rdocx/pull/173),
[PR 181](https://github.com/tensorbee/rdocx/pull/181),
[PR 189](https://github.com/tensorbee/rdocx/pull/189),
[PR 192](https://github.com/tensorbee/rdocx/pull/192),
[PR 208](https://github.com/tensorbee/rdocx/pull/208),
[PR 209](https://github.com/tensorbee/rdocx/pull/209),
[PR 218](https://github.com/tensorbee/rdocx/pull/218),
[PR 219](https://github.com/tensorbee/rdocx/pull/219),
[PR 221](https://github.com/tensorbee/rdocx/pull/221),
[PR 223](https://github.com/tensorbee/rdocx/pull/223),
[PR 224](https://github.com/tensorbee/rdocx/pull/224),
[PR 230](https://github.com/tensorbee/rdocx/pull/230),
[PR 231](https://github.com/tensorbee/rdocx/pull/231),
[PR 234](https://github.com/tensorbee/rdocx/pull/234),
[PR 235](https://github.com/tensorbee/rdocx/pull/235),
[PR 238](https://github.com/tensorbee/rdocx/pull/238), and
[PR 252](https://github.com/tensorbee/rdocx/pull/252). The shared rendering
and layout work came from
[PR 175](https://github.com/tensorbee/rdocx/pull/175),
[PR 188](https://github.com/tensorbee/rdocx/pull/188),
[PR 196](https://github.com/tensorbee/rdocx/pull/196),
[PR 206](https://github.com/tensorbee/rdocx/pull/206),
[PR 207](https://github.com/tensorbee/rdocx/pull/207),
[PR 222](https://github.com/tensorbee/rdocx/pull/222), and
[PR 242](https://github.com/tensorbee/rdocx/pull/242). These PRs were closed
after their behavior was integrated or hardened in the repository. Atul
Sharma reviewed and integrated the release family.

## rpptx-v0.12.1

### Highlights

This patch recovery replaces the failed immutable `rpptx-v0.12.0` workflow
attempt. That tag published no crates and created no GitHub release. The
package content below is the reviewed S73 incubating family prepared at the
new coherent version.

The shared OOXML and PowerPoint family moves to 0.12.1 with correct picture
transparency, safe rendering of large and transparent raster pictures, notes
that render and update reliably, portable authored charts, and searchable PDF
output in logical reading order. The `rpptx` CLI now ships as prebuilt
binaries attached to its GitHub release.

### Added

- Picture transparency from `a:alphaModFix` is applied across slide, layout,
  master, background, preview, SVG, PDF, and raster output, based on
  [PR 105](https://github.com/tensorbee/rdocx/pull/105) for
  [Issue 91](https://github.com/tensorbee/rdocx/issues/91).
- Speaker notes text can be replaced while keeping the notes shape identity
  and run formatting. Presentation replacement counts slide and notes
  matches, and `rpptx replace` refuses to overwrite its input or an existing
  output, validates an expected match count, and publishes atomically. This
  resolves the replacement half of
  [Issue 120](https://github.com/tensorbee/rdocx/issues/120).
- Native presentation APIs expose shape geometry and identity, run font
  details, autofit mode, and formatting-preserving run text replacement for
  the requests in [Issue 121](https://github.com/tensorbee/rdocx/issues/121).
- Rust release tags attach checksummed `rpptx` CLI archives for six Linux,
  macOS, and Windows targets, and `cargo binstall rpptx-cli` resolves them, as
  requested in [Issue 100](https://github.com/tensorbee/rdocx/issues/100).
- Authored charts emit explicit title layout, overlay, marker, and smoothing
  defaults that viewers such as Word and Pages read consistently. This
  builds on [PR 71](https://github.com/tensorbee/rdocx/pull/71) and
  [PR 123](https://github.com/tensorbee/rdocx/pull/123).
- `Presentation::slide_png_deterministic` and `slide_pngs_deterministic` render
  slides with bundled fonts only, supporting the presentation rendering
  automation requested in
  [Issue 76](https://github.com/tensorbee/rdocx/issues/76).
- Layout output exposes `FieldSource` provenance on field glyph runs so
  page-dependent field caches can be written from one deterministic
  pagination pass, as planned in
  [Issue 93](https://github.com/tensorbee/rdocx/issues/93).

### Fixed

- Raster output now premultiplies straight-alpha pictures before compositing,
  and the decoded-pixel render limit rises from 16 MiB to 64 MiB, so larger
  pictures render instead of being skipped silently. The 16 MiB encoded-file
  limit is unchanged. This resolves
  [Issue 119](https://github.com/tensorbee/rdocx/issues/119).
- Notes render when a producer-valid notes slide omits its reverse
  relationship to the slide, while conflicting owners still fail closed. This
  resolves the rendering half of
  [Issue 120](https://github.com/tensorbee/rdocx/issues/120).
- Date, footer, and slide-number placeholders placed on the slide itself stay
  visible regardless of master `p:hf` flags, which only govern inherited
  placeholders. This answers
  [Issue 92](https://github.com/tensorbee/rdocx/issues/92).
- Generated PDFs keep complete logical text lines for search and extraction,
  which resolves the shared renderer part of
  [Issue 74](https://github.com/tensorbee/rdocx/issues/74).
- Shared line layout honors run-level page breaks for page-dependent Word
  output, from [Issue 88](https://github.com/tensorbee/rdocx/issues/88) and
  [PR 102](https://github.com/tensorbee/rdocx/pull/102).

### Compatibility

The exact 15-package shared OOXML and PowerPoint crates.io family moves
together from 0.11.0 to 0.12.1. The selected set is `oxml-core`, `oxml-opc`,
`oxml-media`, `oxml-layout`, `oxml-drawing`, `oxml-pdf`, `oxml-sml`,
`oxml-cli-support`, `oxml-chart`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`,
`rpptx-render`, `rpptx`, and `rpptx-cli`.

This pre-1.0 minor release contains source-incompatible Rust changes. Public
structs gained fields, including `ChartData` (axis titles and a palette, now
with `Default`), `LayoutLine` (`forced_break_after`), field glyph runs
(`field_source`), and `ResolvedImage` (`opacity`). Exhaustive enums gained
variants, including `OpcError::DuplicatePartName`. Code that builds these
structs with full struct literals or matches these enums exhaustively must be
updated. `Presentation::replace_text` now also replaces matches in speaker notes
and includes them in its returned count. Callers that must leave notes untouched
need to account for that change.

`rpptx replace` now fails when no match is found unless an expected count of
zero is given, and it refuses to overwrite an existing output file. Scripts
that relied on silent zero-match success or in-place overwrite must pass an
explicit expected count and a new output path.

Stable Word crates, Python distributions, WASM, and npm packages are outside
this release. `rpptx-wasm@0.12.1` is not a crates.io package.

### Contributors

[@hadim](https://github.com/hadim) reported the rendering, notes, header and
footer flag, automation, field, and binding gaps in Issues
[74](https://github.com/tensorbee/rdocx/issues/74),
[76](https://github.com/tensorbee/rdocx/issues/76),
[88](https://github.com/tensorbee/rdocx/issues/88),
[91](https://github.com/tensorbee/rdocx/issues/91),
[92](https://github.com/tensorbee/rdocx/issues/92),
[93](https://github.com/tensorbee/rdocx/issues/93),
[100](https://github.com/tensorbee/rdocx/issues/100),
[119](https://github.com/tensorbee/rdocx/issues/119),
[120](https://github.com/tensorbee/rdocx/issues/120), and
[121](https://github.com/tensorbee/rdocx/issues/121). They also contributed the
picture transparency implementation in
[PR 105](https://github.com/tensorbee/rdocx/pull/105) and the run page-break
layout in [PR 102](https://github.com/tensorbee/rdocx/pull/102).
[@chevinbrown](https://github.com/chevinbrown) contributed portable authored
charts in [PR 71](https://github.com/tensorbee/rdocx/pull/71) and the line-chart
viewer defaults in [PR 123](https://github.com/tensorbee/rdocx/pull/123).
Atul Sharma integrated, hardened, and released the family.

## rpptx-v0.12.0

### Highlights

The shared OOXML and PowerPoint family moves to 0.12.0 with correct picture
transparency, safe rendering of large and transparent raster pictures, notes
that render and update reliably, portable authored charts, and searchable PDF
output in logical reading order. The `rpptx` CLI now ships as prebuilt
binaries attached to its GitHub release.

### Added

- Picture transparency from `a:alphaModFix` is applied across slide, layout,
  master, background, preview, SVG, PDF, and raster output, based on
  [PR 105](https://github.com/tensorbee/rdocx/pull/105) for
  [Issue 91](https://github.com/tensorbee/rdocx/issues/91).
- Speaker notes text can be replaced while keeping the notes shape identity
  and run formatting. Presentation replacement counts slide and notes
  matches, and `rpptx replace` refuses to overwrite its input or an existing
  output, validates an expected match count, and publishes atomically. This
  resolves the replacement half of
  [Issue 120](https://github.com/tensorbee/rdocx/issues/120).
- Native presentation APIs expose shape geometry and identity, run font
  details, autofit mode, and formatting-preserving run text replacement for
  the requests in [Issue 121](https://github.com/tensorbee/rdocx/issues/121).
- Rust release tags attach checksummed `rpptx` CLI archives for six Linux,
  macOS, and Windows targets, and `cargo binstall rpptx-cli` resolves them, as
  requested in [Issue 100](https://github.com/tensorbee/rdocx/issues/100).
- Authored charts emit explicit title layout, overlay, marker, and smoothing
  defaults that viewers such as Word and Pages read consistently. This
  builds on [PR 71](https://github.com/tensorbee/rdocx/pull/71) and
  [PR 123](https://github.com/tensorbee/rdocx/pull/123).
- `Presentation::slide_png_deterministic` and `slide_pngs_deterministic` render
  slides with bundled fonts only, supporting the presentation rendering
  automation requested in
  [Issue 76](https://github.com/tensorbee/rdocx/issues/76).
- Layout output exposes `FieldSource` provenance on field glyph runs so
  page-dependent field caches can be written from one deterministic
  pagination pass, as planned in
  [Issue 93](https://github.com/tensorbee/rdocx/issues/93).

### Fixed

- Raster output now premultiplies straight-alpha pictures before compositing,
  and the decoded-pixel render limit rises from 16 MiB to 64 MiB, so larger
  pictures render instead of being skipped silently. The 16 MiB encoded-file
  limit is unchanged. This resolves
  [Issue 119](https://github.com/tensorbee/rdocx/issues/119).
- Notes render when a producer-valid notes slide omits its reverse
  relationship to the slide, while conflicting owners still fail closed. This
  resolves the rendering half of
  [Issue 120](https://github.com/tensorbee/rdocx/issues/120).
- Date, footer, and slide-number placeholders placed on the slide itself stay
  visible regardless of master `p:hf` flags, which only govern inherited
  placeholders. This answers
  [Issue 92](https://github.com/tensorbee/rdocx/issues/92).
- Generated PDFs keep complete logical text lines for search and extraction,
  which resolves the shared renderer part of
  [Issue 74](https://github.com/tensorbee/rdocx/issues/74).
- Shared line layout honors run-level page breaks for page-dependent Word
  output, from [Issue 88](https://github.com/tensorbee/rdocx/issues/88) and
  [PR 102](https://github.com/tensorbee/rdocx/pull/102).

### Compatibility

The exact 15-package shared OOXML and PowerPoint crates.io family moves
together from 0.11.0 to 0.12.0. The selected set is `oxml-core`, `oxml-opc`,
`oxml-media`, `oxml-layout`, `oxml-drawing`, `oxml-pdf`, `oxml-sml`,
`oxml-cli-support`, `oxml-chart`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`,
`rpptx-render`, `rpptx`, and `rpptx-cli`.

This pre-1.0 minor release contains source-incompatible Rust changes. Public
structs gained fields, including `ChartData` (axis titles and a palette, now
with `Default`), `LayoutLine` (`forced_break_after`), field glyph runs
(`field_source`), and `ResolvedImage` (`opacity`). Exhaustive enums gained
variants, including `OpcError::DuplicatePartName`. Code that builds these
structs with full struct literals or matches these enums exhaustively must be
updated. `Presentation::replace_text` now also replaces matches in speaker notes
and includes them in its returned count. Callers that must leave notes untouched
need to account for that change.

`rpptx replace` now fails when no match is found unless an expected count of
zero is given, and it refuses to overwrite an existing output file. Scripts
that relied on silent zero-match success or in-place overwrite must pass an
explicit expected count and a new output path.

Stable Word crates, Python distributions, WASM, and npm packages are outside
this release. `rpptx-wasm@0.12.0` is not a crates.io package.

### Contributors

[@hadim](https://github.com/hadim) reported the rendering, notes, header and
footer flag, automation, field, and binding gaps in Issues
[74](https://github.com/tensorbee/rdocx/issues/74),
[76](https://github.com/tensorbee/rdocx/issues/76),
[88](https://github.com/tensorbee/rdocx/issues/88),
[91](https://github.com/tensorbee/rdocx/issues/91),
[92](https://github.com/tensorbee/rdocx/issues/92),
[93](https://github.com/tensorbee/rdocx/issues/93),
[100](https://github.com/tensorbee/rdocx/issues/100),
[119](https://github.com/tensorbee/rdocx/issues/119),
[120](https://github.com/tensorbee/rdocx/issues/120), and
[121](https://github.com/tensorbee/rdocx/issues/121). They also contributed the
picture transparency implementation in
[PR 105](https://github.com/tensorbee/rdocx/pull/105) and the run page-break
layout in [PR 102](https://github.com/tensorbee/rdocx/pull/102).
[@chevinbrown](https://github.com/chevinbrown) contributed portable authored
charts in [PR 71](https://github.com/tensorbee/rdocx/pull/71) and the line-chart
viewer defaults in [PR 123](https://github.com/tensorbee/rdocx/pull/123).
Atul Sharma integrated, hardened, and released the family.

## v0.14.0

### Highlights

The complete stable Word family publishes the M23 from-scratch business
document boundary at 0.14.0. You can now author complete business documents
from a blank package in Rust, including tables, rich headers and footers,
drawings, text boxes, watermarks, styles, numbering, and page-accurate fields.
This release also brings every Word fix reported and contributed since 0.13.1
to crates.io, together with prebuilt `rdocx` CLI binaries.

### Added

- Create Word-compatible DOCX, DOCM, DOTX, and DOTM packages with complete
  settings, properties, themes, font tables, embedded fonts, style graphs, and
  numbering definitions.
- Edit ordered sections, page geometry, rich per-section headers and footers,
  story text in every container, and generic content through transactional
  insert, move, clone, remove, and cross-document fragment import operations.
- Author complete tables, rows, and cells, ordered run content, rich HTML
  fragments in any story, pictures with crop and floating placement, text
  boxes, and section-aware watermarks. Measure paragraphs and tables at a
  caller width for equal-height layouts.
- `update_layout_backed_fields` and `update_page_fields` write PAGE, NUMPAGES,
  PAGEREF, and TOC caches from one deterministic layout, as proposed in
  [Issue 93](https://github.com/tensorbee/rdocx/issues/93) and
  [PR 114](https://github.com/tensorbee/rdocx/pull/114).
- Clone and remove table rows
  ([Issue 95](https://github.com/tensorbee/rdocx/issues/95),
  [PR 113](https://github.com/tensorbee/rdocx/pull/113)), replace an existing
  picture atomically
  ([Issue 96](https://github.com/tensorbee/rdocx/issues/96),
  [PR 107](https://github.com/tensorbee/rdocx/pull/107)), split runs at Unicode
  character offsets
  ([Issue 97](https://github.com/tensorbee/rdocx/issues/97),
  [PR 112](https://github.com/tensorbee/rdocx/pull/112)), and control
  `w:updateFields`
  ([Issue 98](https://github.com/tensorbee/rdocx/issues/98),
  [PR 104](https://github.com/tensorbee/rdocx/pull/104)).
- Insert pictures into any story with `insert_picture_to_story` and anchor
  comments inside table cells with `add_story_comment`, from
  [Issue 121](https://github.com/tensorbee/rdocx/issues/121). Native content
  index lookup, paragraph search, named highlight, shading, and style helpers
  back the Python round-two surface from
  [Issue 94](https://github.com/tensorbee/rdocx/issues/94) and PRs
  [108](https://github.com/tensorbee/rdocx/pull/108),
  [109](https://github.com/tensorbee/rdocx/pull/109),
  [110](https://github.com/tensorbee/rdocx/pull/110), and
  [111](https://github.com/tensorbee/rdocx/pull/111).
- Story items report their direct main-body owner
  ([Issue 86](https://github.com/tensorbee/rdocx/issues/86)), TOC rebuilding
  returns ordered diagnostics, and comments accept optional RFC 3339 dates
  ([Issue 117](https://github.com/tensorbee/rdocx/issues/117)).
- `rdocx-cli` gains schema-versioned comment, revision, comparison, TOC,
  structured text, layout, and guarded replacement commands from
  [Issue 76](https://github.com/tensorbee/rdocx/issues/76). Rust release tags
  attach checksummed CLI archives for six targets that `cargo binstall
  rdocx-cli` resolves, as requested in
  [Issue 100](https://github.com/tensorbee/rdocx/issues/100).

### Fixed

- Documents with drawing ids reused across parts open
  ([Issue 72](https://github.com/tensorbee/rdocx/issues/72)), unused root
  default namespaces save safely
  ([Issue 73](https://github.com/tensorbee/rdocx/issues/73)), and generated PDFs
  keep logical reading order
  ([Issue 74](https://github.com/tensorbee/rdocx/issues/74)).
- Document comparison preserves drawings
  ([Issue 75](https://github.com/tensorbee/rdocx/issues/75)), paragraphs with
  complex fields
  ([Issue 85](https://github.com/tensorbee/rdocx/issues/85),
  [PR 106](https://github.com/tensorbee/rdocx/pull/106)), and two or more
  appended paragraphs
  ([Issue 115](https://github.com/tensorbee/rdocx/issues/115)).
- Explicit false table toggles stay false
  ([Issue 83](https://github.com/tensorbee/rdocx/issues/83),
  [PR 101](https://github.com/tensorbee/rdocx/pull/101)), content-control type
  payloads survive save
  ([Issue 84](https://github.com/tensorbee/rdocx/issues/84),
  [PR 103](https://github.com/tensorbee/rdocx/pull/103)), and fractional line
  spacing values are accepted
  ([PR 122](https://github.com/tensorbee/rdocx/pull/122)).
- Word PNG output premultiplies straight-alpha pictures before compositing
  ([Issue 119](https://github.com/tensorbee/rdocx/issues/119)).
- Run-level page breaks start a new page
  ([Issue 88](https://github.com/tensorbee/rdocx/issues/88),
  [PR 102](https://github.com/tensorbee/rdocx/pull/102)), and header and footer
  pictures render from their own story part
  ([Issue 89](https://github.com/tensorbee/rdocx/issues/89),
  [PR 102](https://github.com/tensorbee/rdocx/pull/102)).
- TOC rebuilding accepts Word's default `\z` switch
  ([Issue 90](https://github.com/tensorbee/rdocx/issues/90),
  [PR 101](https://github.com/tensorbee/rdocx/pull/101)) and writes entries with
  localized styles, section-derived tab stops, and structural numbering tabs
  ([Issue 116](https://github.com/tensorbee/rdocx/issues/116)).
- Comment content types, identities, thread links, and dates survive save and
  reopen ([Issue 117](https://github.com/tensorbee/rdocx/issues/117)), and story
  reads are linear and include tracked insertions and inline content controls
  ([Issue 118](https://github.com/tensorbee/rdocx/issues/118)).
- Note-only edits keep paragraph cache reuse, and restart checkpoints survive
  body-length changes, restoring the editing performance reported in
  [Issue 69](https://github.com/tensorbee/rdocx/issues/69).
- Authored Word charts are portable across Word and Pages
  ([PR 71](https://github.com/tensorbee/rdocx/pull/71),
  [PR 123](https://github.com/tensorbee/rdocx/pull/123)), and reader and
  serializer behavior incorporates hardened equivalents of
  [PR 77](https://github.com/tensorbee/rdocx/pull/77),
  [PR 78](https://github.com/tensorbee/rdocx/pull/78),
  [PR 79](https://github.com/tensorbee/rdocx/pull/79), and
  [PR 80](https://github.com/tensorbee/rdocx/pull/80).

### Compatibility

The exact seven-package stable crates.io family moves together from 0.13.1 to
0.14.0. The selected set is `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`,
`rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli`. It depends on the
separately published shared OOXML 0.12.1 family. This release also satisfies
the crates.io publication request in
[Issue 99](https://github.com/tensorbee/rdocx/issues/99). The unpublished 0.13.2
crates.io train is superseded rather than backfilled.

This pre-1.0 minor release contains source-incompatible Rust changes, including
the following.

- `TocRebuildReport` replaces `diagnostic_count` with ordered `diagnostics` and
  is no longer `Copy`. Read `diagnostics.len()` for the count.
- `ParagraphItemRef::CommentRangeStart` and `CommentRangeEnd` are struct
  variants, and `BookmarkStart` and `BookmarkEnd` gained `has_child_content`.
  Update patterns that match these variants.
- `ListLevel` is no longer `Copy` or `Eq` and now has private fields, so
  `ListLevel { format, start: Some(n) }` literals become
  `ListLevel::new(format).start(n)`.
  `ListNumberFormat` is no longer `Copy` and now has 61 variants, so exhaustive
  matches must handle the new formats.
- `rdocx::Error` gained a `Story` variant, and exhaustive matches must handle
  it.
- `ChartData` and several `rdocx-oxml` `CT_*` model structs gained public
  fields, so full struct literals must add them. `CT_Anchor::from_xml` and
  `CT_Inline::from_xml` now take a namespace-aware `NsReader`.

`rdocx replace` now stages its result
and refuses an output path that already exists, including the input file. It
also checks an exact expected match count before publishing when one is given.
Scripts that replaced a document in place must write to a new output path.
Python, WASM, npm, and PyPI publication authority is unchanged, and
`rdocx-wasm@0.14.0` is not a crates.io package.

### Contributors

[@hadim](https://github.com/hadim) reported the real-document failures and
requests in Issues [72](https://github.com/tensorbee/rdocx/issues/72),
[73](https://github.com/tensorbee/rdocx/issues/73),
[74](https://github.com/tensorbee/rdocx/issues/74),
[75](https://github.com/tensorbee/rdocx/issues/75),
[76](https://github.com/tensorbee/rdocx/issues/76),
[83](https://github.com/tensorbee/rdocx/issues/83),
[84](https://github.com/tensorbee/rdocx/issues/84),
[85](https://github.com/tensorbee/rdocx/issues/85),
[86](https://github.com/tensorbee/rdocx/issues/86),
[88](https://github.com/tensorbee/rdocx/issues/88),
[89](https://github.com/tensorbee/rdocx/issues/89),
[90](https://github.com/tensorbee/rdocx/issues/90),
[93](https://github.com/tensorbee/rdocx/issues/93),
[95](https://github.com/tensorbee/rdocx/issues/95),
[96](https://github.com/tensorbee/rdocx/issues/96),
[97](https://github.com/tensorbee/rdocx/issues/97),
[98](https://github.com/tensorbee/rdocx/issues/98),
[99](https://github.com/tensorbee/rdocx/issues/99),
[100](https://github.com/tensorbee/rdocx/issues/100),
[115](https://github.com/tensorbee/rdocx/issues/115),
[116](https://github.com/tensorbee/rdocx/issues/116),
[117](https://github.com/tensorbee/rdocx/issues/117),
[118](https://github.com/tensorbee/rdocx/issues/118),
[119](https://github.com/tensorbee/rdocx/issues/119), and
[121](https://github.com/tensorbee/rdocx/issues/121), and requested the Python
editing surface those native helpers support in
[Issue 94](https://github.com/tensorbee/rdocx/issues/94).
They contributed the implementations reconciled from PRs
[101](https://github.com/tensorbee/rdocx/pull/101),
[102](https://github.com/tensorbee/rdocx/pull/102),
[103](https://github.com/tensorbee/rdocx/pull/103),
[104](https://github.com/tensorbee/rdocx/pull/104),
[106](https://github.com/tensorbee/rdocx/pull/106),
[107](https://github.com/tensorbee/rdocx/pull/107),
[108](https://github.com/tensorbee/rdocx/pull/108),
[109](https://github.com/tensorbee/rdocx/pull/109),
[110](https://github.com/tensorbee/rdocx/pull/110),
[111](https://github.com/tensorbee/rdocx/pull/111),
[112](https://github.com/tensorbee/rdocx/pull/112),
[113](https://github.com/tensorbee/rdocx/pull/113), and
[114](https://github.com/tensorbee/rdocx/pull/114).
[@pedroassumpcao](https://github.com/pedroassumpcao) contributed the reader and
serializer contracts in PRs [77](https://github.com/tensorbee/rdocx/pull/77),
[78](https://github.com/tensorbee/rdocx/pull/78),
[79](https://github.com/tensorbee/rdocx/pull/79), and
[80](https://github.com/tensorbee/rdocx/pull/80), and fractional line spacing in
[PR 122](https://github.com/tensorbee/rdocx/pull/122).
[@chevinbrown](https://github.com/chevinbrown) contributed portable authored
Word charts in [PR 71](https://github.com/tensorbee/rdocx/pull/71) and the
line-chart viewer defaults in [PR 123](https://github.com/tensorbee/rdocx/pull/123).
[@emptinessform](https://github.com/emptinessform) diagnosed and measured the
note-edit and restart performance regression in
[Issue 69](https://github.com/tensorbee/rdocx/issues/69). Atul Sharma
integrated, hardened, and released the family.

## py-rdocx-v0.14.0

### Highlights

`rdocx 0.14.0` for Python matches the native 0.14.0 Word family and makes the
binding a practical automation surface. Python can now insert, clone, move,
and remove content, format paragraphs and runs, edit stories and hyperlinks,
resolve revisions, update page fields from layout, and insert pictures, all
with typed, owner-checked operations.

### Added

- Indexed content insertion, cloning, movement, removal, and counted
  replacement, based on [PR 109](https://github.com/tensorbee/rdocx/pull/109)
  and [PR 111](https://github.com/tensorbee/rdocx/pull/111).
- Paragraph style and numbering plus run style, named Word highlights, and
  separate shading, based on
  [PR 108](https://github.com/tensorbee/rdocx/pull/108).
- Header, footer, and story text mutation, hyperlink creation, revision
  resolution, field updates, and exact story item XML, based on
  [PR 109](https://github.com/tensorbee/rdocx/pull/109) and
  [PR 110](https://github.com/tensorbee/rdocx/pull/110). Together these complete
  the round-two requests in
  [Issue 94](https://github.com/tensorbee/rdocx/issues/94).
- `Table.clone_row` and row removal, including Word files whose root declares
  an unused default namespace
  ([Issue 95](https://github.com/tensorbee/rdocx/issues/95),
  [PR 113](https://github.com/tensorbee/rdocx/pull/113)), picture replacement
  ([Issue 96](https://github.com/tensorbee/rdocx/issues/96),
  [PR 107](https://github.com/tensorbee/rdocx/pull/107)), run splitting
  ([Issue 97](https://github.com/tensorbee/rdocx/issues/97),
  [PR 112](https://github.com/tensorbee/rdocx/pull/112)), and the
  `update_fields_on_open` setting
  ([Issue 98](https://github.com/tensorbee/rdocx/issues/98),
  [PR 104](https://github.com/tensorbee/rdocx/pull/104)).
- `update_page_fields` and `update_layout_backed_fields` with an owned report
  ([Issue 93](https://github.com/tensorbee/rdocx/issues/93),
  [PR 114](https://github.com/tensorbee/rdocx/pull/114)), ordered TOC rebuild
  diagnostics ([Issue 90](https://github.com/tensorbee/rdocx/issues/90)), and
  direct body ownership on story items
  ([Issue 86](https://github.com/tensorbee/rdocx/issues/86)).
- Picture insertion from bytes at story positions, comment ranges inside table
  cells, and optional comment dates, from
  [Issue 121](https://github.com/tensorbee/rdocx/issues/121) and
  [Issue 117](https://github.com/tensorbee/rdocx/issues/117).

### Fixed

- `story_items` and `hyperlinks` run in linear time, and `Paragraph.text` and
  `Paragraph.runs` include tracked insertions and inline content controls
  ([Issue 118](https://github.com/tensorbee/rdocx/issues/118)).
- The package includes the native 0.14.0 fixes for comparison with drawing
  namespaces declared on ancestor elements and with complex fields
  ([Issue 75](https://github.com/tensorbee/rdocx/issues/75),
  [Issue 85](https://github.com/tensorbee/rdocx/issues/85)), appended
  paragraphs ([Issue 115](https://github.com/tensorbee/rdocx/issues/115)),
  straight-alpha pictures in PNG output
  ([Issue 119](https://github.com/tensorbee/rdocx/issues/119)), table toggles
  ([Issue 83](https://github.com/tensorbee/rdocx/issues/83),
  [PR 101](https://github.com/tensorbee/rdocx/pull/101)), content controls
  ([Issue 84](https://github.com/tensorbee/rdocx/issues/84),
  [PR 103](https://github.com/tensorbee/rdocx/pull/103)), run page breaks and
  header pictures
  ([Issue 88](https://github.com/tensorbee/rdocx/issues/88),
  [Issue 89](https://github.com/tensorbee/rdocx/issues/89),
  [PR 102](https://github.com/tensorbee/rdocx/pull/102)), TOC entries
  ([Issue 116](https://github.com/tensorbee/rdocx/issues/116)), comment identity
  ([Issue 117](https://github.com/tensorbee/rdocx/issues/117)), comparison with
  complex fields ([PR 106](https://github.com/tensorbee/rdocx/pull/106)), and
  fractional line spacing
  ([PR 122](https://github.com/tensorbee/rdocx/pull/122)).

### Compatibility

The distribution and import name remains `rdocx`, now at 0.14.0 to match the
native crate. It requires Python 3.9 or newer through six `cp39-abi3` platform
wheels and one source distribution. Existing 0.13.2 document calls remain source
compatible, and the direct-body `RunPosition` call shape is unchanged. The one
incompatible signature change is the `TocRebuildReport` constructor, which now
takes `diagnostics` instead of `diagnostic_count`. Its `diagnostic_count`
property remains. `Paragraph.runs` now includes runs inside tracked insertions
and inline content controls, so code that stored run indexes from 0.13.2 should
look them up again. This release does not publish `rpptx`, any crates.io
package, WASM package, or npm package.

### Contributors

[@hadim](https://github.com/hadim) drove the round-two and round-three Python
surface through Issues [86](https://github.com/tensorbee/rdocx/issues/86),
[90](https://github.com/tensorbee/rdocx/issues/90),
[93](https://github.com/tensorbee/rdocx/issues/93),
[94](https://github.com/tensorbee/rdocx/issues/94),
[95](https://github.com/tensorbee/rdocx/issues/95),
[96](https://github.com/tensorbee/rdocx/issues/96),
[97](https://github.com/tensorbee/rdocx/issues/97),
[98](https://github.com/tensorbee/rdocx/issues/98),
[117](https://github.com/tensorbee/rdocx/issues/117),
[118](https://github.com/tensorbee/rdocx/issues/118), and
[121](https://github.com/tensorbee/rdocx/issues/121), and reported the native
defects in Issues [75](https://github.com/tensorbee/rdocx/issues/75),
[83](https://github.com/tensorbee/rdocx/issues/83),
[84](https://github.com/tensorbee/rdocx/issues/84),
[85](https://github.com/tensorbee/rdocx/issues/85),
[88](https://github.com/tensorbee/rdocx/issues/88),
[89](https://github.com/tensorbee/rdocx/issues/89),
[115](https://github.com/tensorbee/rdocx/issues/115),
[116](https://github.com/tensorbee/rdocx/issues/116), and
[119](https://github.com/tensorbee/rdocx/issues/119). They contributed the
implementations reconciled from PRs
[101](https://github.com/tensorbee/rdocx/pull/101),
[102](https://github.com/tensorbee/rdocx/pull/102),
[103](https://github.com/tensorbee/rdocx/pull/103),
[104](https://github.com/tensorbee/rdocx/pull/104),
[106](https://github.com/tensorbee/rdocx/pull/106),
[107](https://github.com/tensorbee/rdocx/pull/107),
[108](https://github.com/tensorbee/rdocx/pull/108),
[109](https://github.com/tensorbee/rdocx/pull/109),
[110](https://github.com/tensorbee/rdocx/pull/110),
[111](https://github.com/tensorbee/rdocx/pull/111),
[112](https://github.com/tensorbee/rdocx/pull/112),
[113](https://github.com/tensorbee/rdocx/pull/113), and
[114](https://github.com/tensorbee/rdocx/pull/114).
[@pedroassumpcao](https://github.com/pedroassumpcao) contributed fractional
line spacing in [PR 122](https://github.com/tensorbee/rdocx/pull/122). Atul
Sharma integrated, hardened, and released the distribution.

## py-rpptx-v0.12.1

### Highlights

This patch recovery follows the failed immutable `rpptx-v0.12.0` Rust tag,
which published no crates and created no GitHub release. It keeps the Python
distribution aligned with the recovered native family.

`rpptx 0.12.1` for Python matches the native 0.12.1 presentation family and
adds notes editing, shape inspection, and formatting-preserving text
replacement, together with correct transparency and notes rendering.

### Added

- Set speaker notes text through `Slide.notes_text` while keeping the notes
  shape identity and run formatting, requested in
  [Issue 121](https://github.com/tensorbee/rdocx/issues/121) and
  [Issue 120](https://github.com/tensorbee/rdocx/issues/120).
- Inspect shape geometry and identity, run font details, and autofit mode, and
  replace run text without losing formatting, from
  [Issue 121](https://github.com/tensorbee/rdocx/issues/121).

### Fixed

- Notes render when the notes slide omits its reverse relationship
  ([Issue 120](https://github.com/tensorbee/rdocx/issues/120)).
- Picture transparency is applied in every output
  ([Issue 91](https://github.com/tensorbee/rdocx/issues/91),
  [PR 105](https://github.com/tensorbee/rdocx/pull/105)), straight-alpha
  pictures composite correctly, and the decoded-pixel render limit rises from
  16 MiB to 64 MiB, so larger pictures render instead of being skipped
  ([Issue 119](https://github.com/tensorbee/rdocx/issues/119)).
- Slide-owned date, footer, and slide-number placeholders stay visible under
  master header and footer flags
  ([Issue 92](https://github.com/tensorbee/rdocx/issues/92)).

### Compatibility

The distribution and import name remains `rpptx`, now at 0.12.1 to match the
native crate. It requires Python 3.9 or newer through six `cp39-abi3` platform
wheels and one source distribution. Existing 0.11.0 calls remain source
compatible. This release does not publish `rdocx`, any crates.io package, WASM
package, or npm package.

### Contributors

[@hadim](https://github.com/hadim) reported the transparency, header and footer
flag, raster, notes, and binding gaps in Issues
[91](https://github.com/tensorbee/rdocx/issues/91),
[92](https://github.com/tensorbee/rdocx/issues/92),
[119](https://github.com/tensorbee/rdocx/issues/119),
[120](https://github.com/tensorbee/rdocx/issues/120), and
[121](https://github.com/tensorbee/rdocx/issues/121), and contributed the
transparency implementation in
[PR 105](https://github.com/tensorbee/rdocx/pull/105). Atul Sharma integrated,
hardened, and released the distribution.

## py-rpptx-v0.12.0

### Highlights

`rpptx 0.12.0` for Python matches the native 0.12.0 presentation family and
adds notes editing, shape inspection, and formatting-preserving text
replacement, together with correct transparency and notes rendering.

### Added

- Set speaker notes text through `Slide.notes_text` while keeping the notes
  shape identity and run formatting, requested in
  [Issue 121](https://github.com/tensorbee/rdocx/issues/121) and
  [Issue 120](https://github.com/tensorbee/rdocx/issues/120).
- Inspect shape geometry and identity, run font details, and autofit mode, and
  replace run text without losing formatting, from
  [Issue 121](https://github.com/tensorbee/rdocx/issues/121).

### Fixed

- Notes render when the notes slide omits its reverse relationship
  ([Issue 120](https://github.com/tensorbee/rdocx/issues/120)).
- Picture transparency is applied in every output
  ([Issue 91](https://github.com/tensorbee/rdocx/issues/91),
  [PR 105](https://github.com/tensorbee/rdocx/pull/105)), straight-alpha
  pictures composite correctly, and the decoded-pixel render limit rises from
  16 MiB to 64 MiB, so larger pictures render instead of being skipped
  ([Issue 119](https://github.com/tensorbee/rdocx/issues/119)).
- Slide-owned date, footer, and slide-number placeholders stay visible under
  master header and footer flags
  ([Issue 92](https://github.com/tensorbee/rdocx/issues/92)).

### Compatibility

The distribution and import name remains `rpptx`, now at 0.12.0 to match the
native crate. It requires Python 3.9 or newer through six `cp39-abi3` platform
wheels and one source distribution. Existing 0.11.0 calls remain source
compatible. This release does not publish `rdocx`, any crates.io package, WASM
package, or npm package.

### Contributors

[@hadim](https://github.com/hadim) reported the transparency, header and footer
flag, raster, notes, and binding gaps in Issues
[91](https://github.com/tensorbee/rdocx/issues/91),
[92](https://github.com/tensorbee/rdocx/issues/92),
[119](https://github.com/tensorbee/rdocx/issues/119),
[120](https://github.com/tensorbee/rdocx/issues/120), and
[121](https://github.com/tensorbee/rdocx/issues/121), and contributed the
transparency implementation in
[PR 105](https://github.com/tensorbee/rdocx/pull/105). Atul Sharma integrated,
hardened, and released the distribution.

## py-rdocx-v0.13.2

### Highlights

`rdocx 0.13.2` adds the complete user-facing PyPI project page that was absent
from the immutable first Python release. The distribution now publishes its
README, installation and quick-start guidance, capability summary, typing
contract, classifiers, and project links alongside the native Word automation
surface.

### Added

- The PyPI long description documents installation, DOCX creation and editing,
  comparison, comments, stories, deterministic layout, PDF and image output,
  type checking, compatibility boundaries, support links, and licensing.
- Package metadata now identifies the project author, focused search keywords,
  Python and Rust classifiers, and direct homepage, repository, issue tracker,
  and changelog links.
- The release contains six platform wheels and one source distribution for
  CPython 3.9 and newer through `cp39-abi3`.

### Fixed

- Artifact validation now rejects a wheel or source distribution whose
  embedded metadata lacks the reviewed Markdown description, project summary,
  author, keywords, classifiers, or project links.
- The latest package retains the direct fixes for
  [Issue 72](https://github.com/tensorbee/rdocx/issues/72),
  [Issue 73](https://github.com/tensorbee/rdocx/issues/73),
  [Issue 74](https://github.com/tensorbee/rdocx/issues/74), and
  [Issue 75](https://github.com/tensorbee/rdocx/issues/75), plus the Python
  publication outcome requested in
  [Issue 76](https://github.com/tensorbee/rdocx/issues/76).
- Reader and serializer behavior retains hardened equivalents of
  [PR 77](https://github.com/tensorbee/rdocx/pull/77),
  [PR 78](https://github.com/tensorbee/rdocx/pull/78),
  [PR 79](https://github.com/tensorbee/rdocx/pull/79), and
  [PR 80](https://github.com/tensorbee/rdocx/pull/80).

### Compatibility

The distribution and import name remains `rdocx`. It requires Python 3.9 or
newer and uses the stable ABI from Python 3.9. The native `rdocx`, binding
crate, built artifacts, release tag, and PyPI version are aligned at 0.13.2.
This packaging correction changes no Python runtime API. The immutable
`rdocx 0.13.1` release remains available.

### Contributors

[@hadim](https://github.com/hadim) reported the real-document failures in
Issues [72](https://github.com/tensorbee/rdocx/issues/72),
[73](https://github.com/tensorbee/rdocx/issues/73),
[74](https://github.com/tensorbee/rdocx/issues/74), and
[75](https://github.com/tensorbee/rdocx/issues/75), and requested the Python
publication surface in
[Issue 76](https://github.com/tensorbee/rdocx/issues/76).
[@pedroassumpcao](https://github.com/pedroassumpcao) contributed the reader and
serializer contracts retained through PRs
[77](https://github.com/tensorbee/rdocx/pull/77),
[78](https://github.com/tensorbee/rdocx/pull/78),
[79](https://github.com/tensorbee/rdocx/pull/79), and
[80](https://github.com/tensorbee/rdocx/pull/80). Atul Sharma integrated the
native, binding, metadata-validation, and release paths.

## py-rdocx-v0.13.1

### Highlights

The first `rdocx` Python release brings the distinctive Word automation
surface to PyPI at the same 0.13.1 version as the native `rdocx` crate. It
supports CPython 3.9 and newer through `cp39-abi3` wheels.

### Added

- Comparison, comments and replies, revision filtering and resolution,
  deterministic layout snapshots, TOC rebuilding, ordered section and style
  inspection, rich header and footer stories, and relationship-safe
  hyperlinks are available from Python.
- Installed type declarations are checked with strict mypy and stubtest. The
  release contains six platform wheels and one source distribution.

### Fixed

- Documents now accept drawing identifiers scoped per package part, retain
  unused Word default namespaces safely, preserve drawings through comparison,
  and emit searchable PDF text in logical reading order. These changes resolve
  [Issue 72](https://github.com/tensorbee/rdocx/issues/72),
  [Issue 73](https://github.com/tensorbee/rdocx/issues/73),
  [Issue 74](https://github.com/tensorbee/rdocx/issues/74), and
  [Issue 75](https://github.com/tensorbee/rdocx/issues/75).
- Reader and serializer behavior incorporates hardened equivalents of
  [PR 77](https://github.com/tensorbee/rdocx/pull/77),
  [PR 78](https://github.com/tensorbee/rdocx/pull/78),
  [PR 79](https://github.com/tensorbee/rdocx/pull/79), and
  [PR 80](https://github.com/tensorbee/rdocx/pull/80).
- Publication rejects a partial or mixed project artifact set, mismatched name
  or version metadata, a non-ABI3 wheel, manual-dispatch publication, and any
  release that lacks fresh approval at the reviewed commit.

### Compatibility

The distribution and import name is `rdocx`. It requires Python 3.9 or newer
and uses the stable ABI from Python 3.9. Existing locally built binding APIs
remain source compatible. This release does not publish or change `rpptx`, any
crates.io package, WASM package, or npm package.

### Contributors

[@hadim](https://github.com/hadim) reported the four real-document failures in
Issues [72](https://github.com/tensorbee/rdocx/issues/72),
[73](https://github.com/tensorbee/rdocx/issues/73),
[74](https://github.com/tensorbee/rdocx/issues/74), and
[75](https://github.com/tensorbee/rdocx/issues/75). They also identified the
missing high-value Python surface and made the case for PyPI publication in
[Issue 76](https://github.com/tensorbee/rdocx/issues/76).
[@pedroassumpcao](https://github.com/pedroassumpcao) contributed the reader and
serializer contracts in PRs [77](https://github.com/tensorbee/rdocx/pull/77),
[78](https://github.com/tensorbee/rdocx/pull/78),
[79](https://github.com/tensorbee/rdocx/pull/79), and
[80](https://github.com/tensorbee/rdocx/pull/80). Atul Sharma integrated and
hardened the native, CLI, binding, and release paths.

## py-rpptx-v0.11.0

### Highlights

The first `rpptx` Python release brings presentation automation to PyPI at the
same 0.11.0 version as the native `rpptx` crate. It supports CPython 3.9 and
newer through `cp39-abi3` wheels.

### Added

- Deterministic presentation PDF and slide PNG output, speaker notes as text,
  PDF, and PNG, plus modern comment authors, threads, replies, and ordered
  comment movement are available from Python.
- The PyPI long description documents installation, quick-start examples,
  presentation editing, rendering, notes, comments, type checking,
  compatibility boundaries, support links, and licensing. Project metadata
  also supplies the author, keywords, classifiers, and direct project URLs.
- Installed type declarations are checked with strict mypy and stubtest. The
  release contains six platform wheels and one source distribution.

### Fixed

- The Python binding crate, project metadata, built artifacts, release tag,
  and PyPI version now agree with the native `rpptx` 0.11.0 version.
- Artifact validation rejects a wheel or source distribution whose embedded
  metadata lacks the reviewed Markdown description or project links.
- Publication rejects a partial or mixed project artifact set, mismatched name
  or version metadata, a non-ABI3 wheel, manual-dispatch publication, and any
  release that lacks fresh approval at the reviewed commit.

### Compatibility

The distribution and import name is `rpptx`. It requires Python 3.9 or newer
and uses the stable ABI from Python 3.9. Existing locally built binding APIs
remain source compatible. This release does not publish or change `rdocx`, any
crates.io package, WASM package, or npm package.

### Contributors

[@hadim](https://github.com/hadim) identified the missing presentation binding
surface and made the case for publishing `rpptx` on PyPI in
[Issue 76](https://github.com/tensorbee/rdocx/issues/76). Atul Sharma
implemented and hardened the native binding and release paths.

## v0.13.1

### Highlights

The complete stable Word family publishes the M22 Word-depth boundary at
0.13.1. This patch release recovers from the immutable v0.13.0 attempt, which
published only five low-level stable packages before registry verification
stopped the workflow.

### Added

- Publish the complete DOCX, DOCM, DOTX, and DOTM package identity and bounded
  Flat OPC interchange surface.
- Publish bounded MHTML import and export with contained image and link
  resolution, deterministic output, and stable loss diagnostics.
- Publish the M22 OfficeMath, field, table-of-contents, merge, comparison,
  executable-content, glossary, and package-story outcomes in one coherent
  stable family.

### Fixed

- Pin the stable family to the separately published shared OOXML 0.11.0 family,
  which contains the Word main content-type constants required by `rdocx`.
- Replace the incomplete v0.13.0 publication boundary with a new coherent patch
  family without moving its tag or altering its five immutable registry entries.

### Compatibility

The exact seven-package stable crates.io family moves together from the last
complete 0.12.0 release to 0.13.1. The selected set is `rdocx-opc`,
`rdocx-oxml`, `rdocx-layout`, `rdocx-html`, `rdocx-pdf`, `rdocx`, and
`rdocx-cli`. It depends on the separately published shared OOXML 0.11.0 family.

The native Rust APIs remain additive pre-1.0 surfaces. The immutable v0.13.0
attempt and its five published packages remain available as historical release
evidence. Python, WASM, npm, and PyPI publication authority is unchanged, and
`rdocx-wasm@0.13.1` is not a crates.io package.

### Contributors

Atul Sharma maintained the release. No external issue or pull request belongs
to the selected stable-family changes since `v0.12.0`, so this release has no
external contribution notification.

## rpptx-v0.11.0

### Highlights

The shared OOXML and PowerPoint family moves to 0.11.0 with the Word main
content-type constants required by modern stable package-class handling.
`oxml-opc` now names DOCX, DOCM, DOTX, and DOTM main parts through one shared
vocabulary.

### Added

- Add `WORD_DOCUMENT`, `WORD_DOCUMENT_MACRO_ENABLED`, `WORD_TEMPLATE`, and
  `WORD_TEMPLATE_MACRO_ENABLED` to `oxml-opc`.
- Cover the four Word main content-type constants in the shared relationship
  vocabulary regression.

### Fixed

- Publish the shared contract required by packaged stable `rdocx`, which the
  immutable shared 0.10.0 archive does not contain.

### Compatibility

The exact 15-package shared OOXML and PowerPoint crates.io family moves
together from 0.10.0 to 0.11.0. The constants are additive pre-1.0 Rust API.
The stable Word family remains at 0.13.0 during this selected-family release.
Its immutable v0.13.0 attempt published five low-level packages before
registry verification exposed the missing shared constants.

Stable Word, Python, WASM, npm, and PyPI packages are outside this release's
publication authority. `rpptx-wasm@0.11.0` is not a crates.io package.

### Contributors

Atul Sharma maintained the release. No external issue or pull request belongs
to the selected family changes since `rpptx-v0.10.0`, so this release has no
external contribution notification.

## v0.13.0

### Highlights

The stable Word family completes the M22 Word-depth boundary at 0.13.0.
Documents can author and render OfficeMath, rebuild fields and tables of
contents, perform advanced mail merge and comparison, inspect executable and
building-block content, preserve modern package identity, and exchange bounded
Flat OPC and MHTML through the native Rust facade.

### Added

- Add OfficeMath modeling, authoring, deterministic rendering, and MathML and
  LaTeX conversion for the supported equation subset.
- Add field evaluation and updates, dynamic table-of-contents rebuilding,
  sectioned mail merge, and body, header, footer, footnote, and endnote
  comparison with explicit revision policy.
- Add exact VBA, OLE, and ActiveX inventory and mutation, plus glossary,
  building-block, and package-story access without executing embedded content.
- Add DOCX, DOCM, DOTX, and DOTM package identity, output-only class
  conversion, and bounded Flat OPC import and export with payload retention.
- Add bounded MHTML import and export with contained image and link resolution,
  deterministic MIME output, and location-aware loss diagnostics.

### Fixed

- Reject malformed XML lexical forms through the shared `oxml-core` validator
  while preserving each Word owner's established error and rollback surface.
- Preserve unsupported XML, namespace bindings, relationship ownership,
  executable bytes, template semantics, and package-signature invalidation
  evidence through the new editing and interchange paths.
- Keep unsafe, external, ambiguous, malformed, or over-limit MHTML and Flat OPC
  resources from publishing a partial document.

### Compatibility

The exact seven-package stable crates.io family moves together from 0.12.0 to
0.13.0. The selected set is `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`,
`rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli`. It depends on the
separately published shared OOXML 0.10.0 family.

The native Rust APIs are additive pre-1.0 surfaces. Package-class conversion
changes only output identity and does not remove macros. Flat OPC output uses a
deterministic container representation while preserving unsupported inner XML
and binary payload bytes. MHTML supports the documented contained PNG and JPEG
resource subset and never fetches external content. Python, WASM, npm, and PyPI
publication authority is unchanged, and `rdocx-wasm@0.13.0` is not a crates.io
package.

### Contributors

Atul Sharma maintained the release. No external issue or pull request belongs
to the selected stable-family changes since `v0.12.0`, so this release has no
external contribution notification.

## rpptx-v0.10.0

### Highlights

The shared OOXML and PowerPoint family moves to 0.10.0 with one strict XML
1.0 lexical validator owned by `oxml-core`. Format-specific readers can share
declaration, character, name, namespace, reference, comment, and processing
instruction checks while keeping their schema rules and public errors local.

### Added

- Add `XmlLexicalError` and `validate_strict_xml_1_0` to `oxml-core` for
  bounded format-neutral lexical validation.
- Add baseline-aware inline groups to `oxml-layout`, allowing shared layout
  consumers to carry exact ascent and descent without teaching a backend
  document grammar.
- Add the Word glossary content type and relationship constants to `oxml-opc`
  so package owners can resolve glossary parts through shared vocabulary.

### Fixed

- Reject malformed declarations, forbidden XML 1.0 characters, invalid names
  and namespace bindings, duplicate expanded attributes, invalid references,
  malformed comments, and reserved processing instruction targets through one
  reviewed policy.

### Compatibility

The exact 15-package shared OOXML and PowerPoint crates.io family moves
together from 0.9.0 to 0.10.0. The selected set is `oxml-core`, `oxml-opc`,
`oxml-media`, `oxml-layout`, `oxml-drawing`, `oxml-pdf`, `oxml-sml`,
`oxml-cli-support`, `oxml-chart`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`,
`rpptx-render`, `rpptx`, and `rpptx-cli`.

The lexical validator and glossary constants are additive pre-1.0 APIs.
Callers constructing `InlineItem::Group` or `LineItem::Group` literals must
initialize the new `baseline` field. `None` preserves the established
top-aligned group behavior.

The stable Word family remains at 0.12.0 and current source now pins shared
dependencies to 0.10.0. It is outside this release's publication authority.
Python, WASM, npm, and PyPI publication authority is unchanged, and
`rpptx-wasm@0.10.0` is not a crates.io package.

### Contributors

Atul Sharma maintained the release. No external issue or pull request belongs
to the selected family changes since `rpptx-v0.9.0`, so this release has no
external contribution notification.

## v0.12.0

### Highlights

The stable Word family moves to 0.12.0 with richer relationship-safe reader
facts and bounded restart pagination for ordinary prose. Warm layouts now keep
their complete prefix across note references and page-spanning paragraphs
without the document-wide second pagination pass reported after v0.11.1.

### Added

- Expose namespace-aware hyperlink targets, external-image relationships, and
  drawing safety facts shaped by [PR
  61](https://github.com/tensorbee/rdocx/pull/61).
- Expose document, table, row-grid, border, formatting, and retained-property
  completeness facts shaped by [PR
  62](https://github.com/tensorbee/rdocx/pull/62).
- Expose numbering identity, level metadata, and effective paragraph and run
  formatting shaped by [PR
  63](https://github.com/tensorbee/rdocx/pull/63).
- Expose bounded nested-revision projection, preserved insertion facts, and
  ordered complex-field display segments shaped by [PR
  64](https://github.com/tensorbee/rdocx/pull/64).

### Fixed

- Keep direct body paragraphs with footnote or endnote references eligible for
  exact cache reuse while retaining note-part invalidation, addressing [Issue
  65](https://github.com/tensorbee/rdocx/issues/65).
- Admit ordinary multi-line prose and complete block-boundary restart records
  under the shared aggregate cache budget, addressing [Issue
  66](https://github.com/tensorbee/rdocx/issues/66).
- Keep the completed recorded pagination pass when a paragraph spans a page,
  then publish only later complete-boundary checkpoints, addressing [Issue
  67](https://github.com/tensorbee/rdocx/issues/67).

### Compatibility

The exact seven-package stable crates.io family moves together from 0.11.1 to
0.12.0. The selected set is `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`,
`rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli`. It depends on the
separately published shared OOXML 0.9.0 family.

Native reader additions use existing non-exhaustive or additive pre-1.0
surfaces. Full low-level `rdocx-oxml` struct literals written against 0.11.1
must initialize the new preservation fields or use the existing constructors
and `Default` implementations. The cache and pagination fixes require no
migration and leave rendered output unchanged. Python, WASM, npm, and PyPI
publication authority is unchanged, and `rdocx-wasm@0.12.0` is not a crates.io
package.

### Contributors

Thanks to `@pedroassumpcao` for the relationship-safe hyperlink and drawing
reader design in [PR 61](https://github.com/tensorbee/rdocx/pull/61), document
and table completeness design in [PR
62](https://github.com/tensorbee/rdocx/pull/62), numbering and effective
formatting design in [PR 63](https://github.com/tensorbee/rdocx/pull/63), and
tracked insertion and field safety design in [PR
64](https://github.com/tensorbee/rdocx/pull/64).

Thanks to `@emptinessform` for the note-reference cache report in [Issue
65](https://github.com/tensorbee/rdocx/issues/65), ordinary-prose restart report
in [Issue 66](https://github.com/tensorbee/rdocx/issues/66), and page-spanning
paragraph regression report in [Issue
67](https://github.com/tensorbee/rdocx/issues/67).

No named external patch landed directly. Each contribution landed through a
reviewed hardened equivalent that retains namespace identity, raw XML,
bounded work, exact warm and fresh equality, and compatibility contracts. The
four pull requests and Issues 65 and 66 remain closed after their release-bound
thank-yous. Issue 67 remains open after its release-bound thank-you.

## rpptx-v0.9.0

### Highlights

The shared OOXML and PowerPoint family completes the M21 presentation-depth
boundary at 0.9.0. Native Rust callers can inspect and edit modern
collaboration, timing, media, SmartArt, embedded-content, security, and package
variant state. They can also exchange presentations through bounded ODP, HTML,
and PDF workflows and export deterministic animations, notes, and handouts.

### Added

- Read and author modern comments, threaded replies, sections, slide numbers,
  dates, footers, and notes-master and handout-master settings.
- Inspect, create, and invalidate presentation signatures, and read or write
  password-protected packages through explicit optional security features.
- Model animation timing, transitions, morph metadata, audio and video,
  posters, playback settings, and deterministic timeline state.
- Export animated GIF or Motion JPEG AVI output with bounded frame rate,
  dimensions, duration, and media fallback policy.
- Inspect and edit the supported SmartArt data and layout subset, retain
  unsupported diagram content, and render six pinned layout families.
- Inventory, extract, replace, and remove relationship-owned OLE, ActiveX, and
  VBA payloads without executing them.
- Read and write the declared ODP subset, preserve PPTX, PPTM, POTX, POTM,
  PPSX, and PPSM package identity, and export deterministic notes pages and all
  six handout grids.
- Import a bounded HTML5 and CSS subset as editable slide content with pinned
  Chrome structure and render comparisons.
- Import PDF pages as preserved graphics or as the declared editable text,
  image, path, and URI-link subset with strict resource limits and pinned
  Poppler render comparisons.

### Fixed

- Preserve relationship ownership, raw unsupported XML, executable payloads,
  package signatures, and source package class across the new mutation and
  conversion paths.
- Keep unsupported HTML, PDF, SmartArt, media, animation, and interchange
  content explicit through stable diagnostics or retained opaque content
  instead of silently approximating it.
- Keep ordinary static rendering and the 49-entry deterministic output harness
  unchanged while adding the new presentation-depth paths.

### Compatibility

The exact 15-package shared OOXML and PowerPoint crates.io family moves
together from 0.8.0 to 0.9.0. The selected set is `oxml-core`, `oxml-opc`,
`oxml-media`, `oxml-layout`, `oxml-drawing`, `oxml-pdf`, `oxml-sml`,
`oxml-cli-support`, `oxml-chart`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`,
`rpptx-render`, `rpptx`, and `rpptx-cli`.

The native Rust facade and model additions are additive pre-1.0 APIs. Existing
callers need no migration unless they opt into the new methods. HTML import
uses the existing default-template boundary and PDF import uses the existing
render boundary. The stable Word family remains at 0.11.1. Python, WASM, npm,
and PyPI publication authority is unchanged, and `rpptx-wasm@0.9.0` is not a
crates.io package.

### Contributors

Atul Sharma maintained the release. No external issue or pull request belongs
to the selected PowerPoint family for this release.

## v0.11.1

### Highlights

The stable Word family completes the S58 release at 0.11.1 after the immutable
partial v0.11.0 attempt published only `rdocx-opc@0.11.0` and
`rdocx-oxml@0.11.0`. It adds language-aware conditional hyphenation,
multi-script shaping, bidirectional paragraph and run layout, logical text
extraction over visually ordered output, and bounded restart pagination for
ordinary note, header, and footer workloads. The complete stable family now
uses the published shared 0.8.0 family.

### Added

- Apply Word automatic-hyphenation settings and run languages for English,
  French, German, and Spanish without assigning source ranges to generated
  hyphens.
- Shape Arabic, Devanagari, Thai, and Simplified Chinese with deterministic
  font selection, cluster-safe breaking, complete glyph offsets, and stable
  source mapping.
- Carry Word paragraph and run direction into paragraph-wide UAX 9 resolution,
  line-local visual ordering, and logical PDF and SVG extraction.
- Reuse bounded restart pagination for unchanged notes, headers, and footers.
  This hardened equivalent addresses the 700-paragraph note and header or
  footer workloads reported in
  [Issue 53](https://github.com/tensorbee/rdocx/issues/53).
- Avoid the redundant retained-context font byte comparison after the font
  manager has already accepted the exact ordered font set. This hardened
  equivalent addresses the 22 MiB caller-font workload reported in
  [Issue 54](https://github.com/tensorbee/rdocx/issues/54).
- Accept exact whole-valued decimal table measurements while rejecting
  fractional, exponent, overflow, unit-bearing, and malformed forms. This
  hardened equivalent includes the outcome proposed in
  [PR 55](https://github.com/tensorbee/rdocx/pull/55).
- Preserve tracked table-grid history as inert revision metadata while keeping
  the active grid as the only layout input. This hardened equivalent includes
  the outcome proposed in
  [PR 56](https://github.com/tensorbee/rdocx/pull/56).
- Classify the narrow enabled legacy VML horizontal-rule form for native
  inspection while retaining its exact raw XML. This hardened equivalent
  includes the outcome proposed in
  [PR 57](https://github.com/tensorbee/rdocx/pull/57).
- Prime locked Cargo dependencies before the intentionally offline Word
  fidelity harness. This hardened equivalent includes the locked Word fidelity
  dependency preparation proposed in
  [PR 58](https://github.com/tensorbee/rdocx/pull/58).

### Fixed

- Preserve exact warm and fresh layout equality when related stories, note
  references, language, hyphenation, direction, source-less generated content,
  or caller fonts participate in cache identity.
- Preserve logical searchable text while visually positioning mixed-direction
  rich runs, inline objects, numbering markers, stored fields, tab leaders,
  and generated conditional hyphens.
- Preserve namespace-aware unsupported table and run XML, including ancestor
  bindings, without changing schema child order or treating historical grids
  as active layout data.
- Recover the complete stable registry family without moving or deleting the
  immutable v0.11.0 tag. The five stable packages absent from that partial
  attempt are published only as part of the complete 0.11.1 family.

### Compatibility

The exact seven-package stable crates.io family moves together from 0.10.1 to
0.11.1 and requires the separately published shared 0.8.0 family. The stable
set remains `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
`rdocx-pdf`, `rdocx`, and `rdocx-cli`. Python, WASM, npm, and PyPI publication
authority is unchanged.

Low-level Rust callers that use full `CT_RPr`, `LayoutInput`, `CT_PPr`,
`CT_TblGrid`, `TextSegment`, and positioned-layout struct literals must
initialize the new language, automatic-hyphenation, direction, preservation,
and source-mapping fields or use existing defaults and constructors. In
particular, callers constructing full `TextSegment` literals must initialize
the `direction` field. These are intentional pre-1.0 source changes. The native
facade additions are additive, and existing binding method names remain
unchanged. The legacy VML horizontal rule is classified for inspection and
preservation, not rendered.

The immutable partial v0.11.0 attempt contains exactly `rdocx-opc@0.11.0` and
`rdocx-oxml@0.11.0`. It has no GitHub release and receives no contribution
notification. Recovery cleanup may yank those two incomplete entries only
after all seven 0.11.1 packages and the release body verify independently.

### Contributors

Atul Sharma maintained the release. Thanks to authenticated reporter
`@emptinessform` for the note and header or footer restart evidence in
[Issue 53](https://github.com/tensorbee/rdocx/issues/53), including the
corrected attribution and independent page-count ceiling, and for the isolated
caller-font byte-comparison evidence and unsound shallow-comparison caveat in
[Issue 54](https://github.com/tensorbee/rdocx/issues/54). Both final fixes are
hardened equivalents. Both issues remain open after their release-bound
thank-yous.

Thanks to authenticated contributor `@pedroassumpcao` for the whole-valued
decimal table-measurement case in
[PR 55](https://github.com/tensorbee/rdocx/pull/55) at
`056d48fdf23f35e3538ef3d6ff78cf9e3863e3a5`, tracked table-grid history in
[PR 56](https://github.com/tensorbee/rdocx/pull/56) at
`8b79c4cd0452defafe0a58e86b332c98e7fe52d7`, the legacy VML reader
classification in [PR 57](https://github.com/tensorbee/rdocx/pull/57) at
`44498f042a2290ef40c7a6c26025f38e38e9ce2a`, and locked Word fidelity
dependency preparation in
[PR 58](https://github.com/tensorbee/rdocx/pull/58) at
`c8fed1d1268fd765d602bac2da6524900c1c1cfd`. All four outcomes are hardened
equivalents. All four pull requests remain open after their release-bound
thank-yous.

No named external patch landed directly. Each named report or proposal landed
through the hardened equivalent described above so that namespace, exact
lexical parsing, raw preservation, bounded cache identity, and offline oracle
contracts remain intact.

## rpptx-v0.8.0

### Highlights

The shared OOXML and PowerPoint family publishes the complete text-direction
contract required by current stable Word source. This release supplies the
registry boundary needed for the stable 0.11.1 recovery after the immutable
partial v0.11.0 attempt stopped during `rdocx-layout` verification.

### Added

- Carry resolved text direction in the shared `TextSegment.direction` field so
  Word, PowerPoint, PDF, raster, and SVG paths use one reviewed direction
  contract.
- Preserve paragraph-wide bidirectional levels, line-local reordering, logical
  searchable text, inline objects, tab leaders, numbering markers, stored
  fields, and conditional hyphens across the shared layout path.
- Retain the deterministic Arabic, Devanagari, Thai, and Simplified Chinese
  font and shaping substrate published in the previous shared family.

### Fixed

- Make the current stable layout source compile against a complete published
  shared family instead of relying on an API added after 0.7.0.
- Keep malformed rich-run validation, source mapping, and logical extraction
  behavior aligned across PDF, raster, and SVG backends.

### Compatibility

The exact 15-package incubating crates.io family moves together from 0.7.0 to
0.8.0. The additive `TextSegment.direction` field is an intentional pre-1.0
Rust source change for callers that construct full `TextSegment` literals.
Callers using existing shaping and layout entry points receive the resolved
direction from those APIs.

The stable workspace remains prepared at 0.11.0 during this shared release.
The later stable 0.11.1 recovery pins shared dependencies to 0.8.0. The
unpublished `rpptx-wasm` preparation carrier moves to 0.8.0 without gaining
crates.io, npm, or other publication authority.

### Contributors

Atul Sharma maintained the release with the rdocx maintainers. This
shared-family carrier release adds no authenticated external issue or pull
request to its selected contribution inventory, so it prepares no external
notification.

## v0.11.0

### Highlights

The stable Word family adds language-aware conditional hyphenation,
multi-script shaping, bidirectional paragraph and run layout, and logical text
extraction over visually ordered output. Bounded restart pagination now covers
ordinary note, header, and footer workloads, while unchanged caller fonts no
longer pay a second full byte comparison on the warm layout path.

### Added

- Apply Word automatic-hyphenation settings and run languages for English,
  French, German, and Spanish without assigning source ranges to generated
  hyphens.
- Shape Arabic, Devanagari, Thai, and Simplified Chinese with deterministic
  font selection, cluster-safe breaking, complete glyph offsets, and stable
  source mapping.
- Carry Word paragraph and run direction into paragraph-wide UAX 9 resolution,
  line-local visual ordering, and logical PDF and SVG extraction.
- Reuse bounded restart pagination for unchanged notes, headers, and footers.
  This hardened equivalent addresses the 700-paragraph note and header or
  footer workloads reported in
  [Issue 53](https://github.com/tensorbee/rdocx/issues/53).
- Avoid the redundant retained-context font byte comparison after the font
  manager has already accepted the exact ordered font set. This hardened
  equivalent addresses the 22 MiB caller-font workload reported in
  [Issue 54](https://github.com/tensorbee/rdocx/issues/54).
- Accept exact whole-valued decimal table measurements while rejecting
  fractional, exponent, overflow, unit-bearing, and malformed forms. This
  hardened equivalent includes the outcome proposed in
  [PR 55](https://github.com/tensorbee/rdocx/pull/55).
- Preserve tracked table-grid history as inert revision metadata while keeping
  the active grid as the only layout input. This hardened equivalent includes
  the outcome proposed in
  [PR 56](https://github.com/tensorbee/rdocx/pull/56).
- Classify the narrow enabled legacy VML horizontal-rule form for native
  inspection while retaining its exact raw XML. This hardened equivalent
  includes the outcome proposed in
  [PR 57](https://github.com/tensorbee/rdocx/pull/57).
- Prime locked Cargo dependencies before the intentionally offline Word
  fidelity harness. This hardened equivalent includes the locked Word fidelity
  dependency preparation proposed in
  [PR 58](https://github.com/tensorbee/rdocx/pull/58).

### Fixed

- Preserve exact warm and fresh layout equality when related stories, note
  references, language, hyphenation, direction, source-less generated content,
  or caller fonts participate in cache identity.
- Preserve logical searchable text while visually positioning mixed-direction
  rich runs, inline objects, numbering markers, stored fields, tab leaders,
  and generated conditional hyphens.
- Preserve namespace-aware unsupported table and run XML, including ancestor
  bindings, without changing schema child order or treating historical grids
  as active layout data.

### Compatibility

The exact seven-package stable crates.io family moves together from 0.10.1 to
0.11.0 and requires the separately published shared 0.7.0 family. The stable
set remains `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
`rdocx-pdf`, `rdocx`, and `rdocx-cli`. Python, WASM, npm, and PyPI publication
authority is unchanged.

Low-level Rust callers that use full `CT_RPr`, `LayoutInput`, `CT_PPr`,
`CT_TblGrid`, and low-level positioned-layout struct literals must initialize
the new language, automatic-hyphenation, direction, preservation, and source
mapping fields or use existing defaults and constructors. These are
intentional pre-1.0 source changes. The native facade additions are additive,
and existing binding method names remain unchanged. The legacy VML horizontal
rule is classified for inspection and preservation, not rendered.

### Contributors

Atul Sharma maintained the release. Thanks to authenticated reporter
`@emptinessform` for the note and header or footer restart evidence in
[Issue 53](https://github.com/tensorbee/rdocx/issues/53), including the
corrected attribution and independent page-count ceiling, and for the isolated
caller-font byte-comparison evidence and unsound shallow-comparison caveat in
[Issue 54](https://github.com/tensorbee/rdocx/issues/54). Both final fixes are
hardened equivalents, and both issues remain open for their release-bound
notifications.

Thanks to authenticated contributor `@pedroassumpcao` for the whole-valued
decimal table-measurement case in
[PR 55](https://github.com/tensorbee/rdocx/pull/55) at
`056d48fdf23f35e3538ef3d6ff78cf9e3863e3a5`, tracked table-grid history in
[PR 56](https://github.com/tensorbee/rdocx/pull/56) at
`8b79c4cd0452defafe0a58e86b332c98e7fe52d7`, the legacy VML reader
classification in [PR 57](https://github.com/tensorbee/rdocx/pull/57) at
`44498f042a2290ef40c7a6c26025f38e38e9ce2a`, and locked Word fidelity
dependency preparation in
[PR 58](https://github.com/tensorbee/rdocx/pull/58) at
`c8fed1d1268fd765d602bac2da6524900c1c1cfd`. All four outcomes are hardened
equivalents, and all four pull requests remain open for their release-bound
thank-yous.

No named external patch landed directly. Each named report or proposal landed
through the hardened equivalent described above so that namespace, exact
lexical parsing, raw preservation, bounded cache identity, and offline oracle
contracts remain intact.

## rpptx-v0.7.0

### Highlights

The shared OOXML and PowerPoint family adds one complete multilingual text
substrate for the later Word hyphenation, complex-script, and bidirectional
layout stories. Conditional hyphenation, script-aware shaping, cluster-safe
breaking, paragraph direction, line-local visual ordering, and deterministic
complex-script fonts now share one format-neutral contract.

### Added

- Offer conditional hyphenation for English, French, German, and Spanish while
  retaining contiguous source spans and omitting a source for generated
  hyphens.
- Shape Arabic, Devanagari, Thai, and Simplified Chinese with deterministic
  font fallback, explicit script and language, logical clusters, and complete
  two-dimensional glyph advances and offsets.
- Apply ICU complex-script boundaries, CJK prohibited-punctuation rules, and
  UAX 9 bidirectional levels and line-local visual ordering without rewriting
  logical searchable text.
- Carry typed DrawingML paragraph direction through an additive PowerPoint
  sidecar into PDF, raster, and SVG output.
- Bundle licensed Noto Sans Arabic, Devanagari, and Thai fonts plus a
  reproducible Noto Sans Simplified Chinese subset for the approved fixture
  repertoire.

### Fixed

- Apply explicit right-to-left direction to numeric and Latin text across
  styled runs and forced line breaks using one paragraph-wide bidi context.
- Reject malformed rich glyph positioning safely across PDF, raster, and SVG
  backends. SVG retains logical searchable text with an explicit positioning
  approximation diagnostic.

### Compatibility

The exact 15-package incubating crates.io family moves together from 0.6.0 to
0.7.0. This is an intentional pre-1.0 minor boundary for new additive shared
text types, non-exhaustive variants, and sibling resolver and renderer entry
points. Existing legacy Latin struct and entrypoint shapes remain valid, and
their deterministic output remains byte-identical.

The seven-package stable family remains at 0.10.1 and does not opt into the new
shared path in this release. Word property parsing, facade authoring, and final
Word oracle acceptance remain in the later product stories. `rpptx-wasm` is
prepared at 0.7.0 for binding checks but remains unpublished on crates.io, npm,
and every other registry.

### Contributors

Atul Sharma maintained the release with the rdocx maintainers. The selected
F-X058 substrate adds no new authenticated external issue or pull-request
record after rpptx-v0.6.0, so this release prepares no external notification.

## v0.10.1

### Highlights

The stable Word family adds native RTF, HTML, and OpenDocument Text input,
native RTF and OpenDocument Text output, deterministic EPUB and SVG export,
and ordered compatibility readers that preserve unsupported XML. Layout also
honors caller font aliases and restores bounded editor-scale reuse.

This patch release recovers the complete stable family after v0.10.0 published
only `rdocx-opc` and `rdocx-oxml`, then stopped during `rdocx-layout` package
verification. Version 0.10.1 is the first complete stable family carrying the
S56 outcome.

### Added

- Read RTF, HTML5, and OpenDocument Text into the native `Document` facade with
  bounded inputs, ordered diagnostics, and no network access.
- Write deterministic RTF byte streams with stable lossy diagnostics and
  failure-atomic path replacement.
- Write deterministic OpenDocument Text archives with stable lossy diagnostics
  and failure-atomic path replacement.
- Export deterministic EPUB publications that retain supported links, images,
  headings, lists, tables, and accessibility structure while reporting
  unsupported source content.
- Export searchable SVG pages with deterministic fixed-page geometry, images,
  and safe links while reporting unsupported visual content.
- Export exact selected pages as opaque or transparent PNG, quality-controlled
  JPEG, or one deterministic multi-page TIFF through native Word, Python, and
  the general Word and PowerPoint CLI paths.
- Inspect direct table-cell children through `CellRef::items`. This hardened
  equivalent includes the outcome proposed in
  [PR 47](https://github.com/tensorbee/rdocx/pull/47).
- Inspect direct run content through `RunRef::items`, including text, breaks,
  drawings, fields, notes, comments, and preserved XML. This hardened
  equivalent includes the outcome proposed in
  [PR 48](https://github.com/tensorbee/rdocx/pull/48).
- Inspect direct paragraph and hyperlink content without changing established
  flattened accessors. This hardened equivalent includes the outcome proposed
  in [PR 49](https://github.com/tensorbee/rdocx/pull/49).
- Classify retained unsupported body XML through borrowed qualified-name,
  namespace, and child-content facts without inventing source bytes. This
  hardened equivalent includes the outcome proposed in
  [PR 50](https://github.com/tensorbee/rdocx/pull/50).

### Fixed

- Honor caller-supplied font family aliases without duplicating font bytes,
  while retaining deterministic fallback and bounded caches. This hardened
  equivalent addresses
  [Issue 44](https://github.com/tensorbee/rdocx/issues/44) and the reference
  implementation in [PR 45](https://github.com/tensorbee/rdocx/pull/45).
- Restore bounded reusable layout performance for document load, typing, undo,
  and table mutation while retaining exact shaping, source mappings, and page
  structure. This hardened equivalent addresses
  [Issue 46](https://github.com/tensorbee/rdocx/issues/46).
- Reject undecodable ordinary and deleted Word text instead of publishing an
  empty lossy value. This hardened equivalent includes the correction proposed
  in [PR 52](https://github.com/tensorbee/rdocx/pull/52).

### Compatibility

The exact seven-package stable crates.io family moves together from 0.9.0 to
0.10.1. This is an intentional pre-1.0 Rust source boundary. The shared OOXML
and PowerPoint family is published at 0.6.0. Python, WASM, npm, and PyPI
publication authority is unchanged.

The immutable v0.10.0 attempt contains only `rdocx-opc` and `rdocx-oxml`.
Callers should select 0.10.1 for a coherent seven-package stable graph. No
v0.10.0 tag or registry entry was moved, replaced, or reused.

`ST_NumberFormat` now preserves producer-defined values in `Other(String)`.
The enum no longer implements `Copy`, and exhaustive matches must handle the
new value-bearing variant. Callers should borrow or clone numbering formats as
needed and retain unknown values rather than substituting a modeled marker.
This hardened equivalent includes the preservation outcome proposed in
[PR 51](https://github.com/tensorbee/rdocx/pull/51).

External layout backends must continue to recurse through
`PositionedElement::MarkedContent` or use `oxml_layout::walk`, as documented for
v0.9.0. No migration is required for callers that use the high-level document
facade and do not exhaustively match `ST_NumberFormat`.

### Contributors

Atul Sharma maintained the release. Thanks to `@emptinessform` for the caller
font-alias report and reference implementation in
[Issue 44](https://github.com/tensorbee/rdocx/issues/44) and
[PR 45](https://github.com/tensorbee/rdocx/pull/45), and for the editor
performance measurements and migration evidence in
[Issue 46](https://github.com/tensorbee/rdocx/issues/46).

Thanks to `@pedroassumpcao` for the ordered cell, run, paragraph, and hyperlink
reader designs in [PR 47](https://github.com/tensorbee/rdocx/pull/47),
[PR 48](https://github.com/tensorbee/rdocx/pull/48), and
[PR 49](https://github.com/tensorbee/rdocx/pull/49), the unsupported XML facts
in [PR 50](https://github.com/tensorbee/rdocx/pull/50), producer-defined
numbering preservation in [PR 51](https://github.com/tensorbee/rdocx/pull/51),
and fail-closed text decoding in
[PR 52](https://github.com/tensorbee/rdocx/pull/52).

No named external patch landed directly. Each named report or proposal landed
through the hardened equivalent described above so that current namespace,
non-exhaustive API, bounded-allocation, diagnostic, and compatibility contracts
remain intact.

## rpptx-v0.6.0

### Highlights

The shared OOXML and PowerPoint family adds caller-controlled font aliases,
bounded reusable layout work, deterministic page image output, and common CLI
page selection. This release publishes the shared APIs required by the stable
Word family before its separate v0.10.1 recovery release.

### Added

- Configure bounded caller font family aliases through the shared font manager
  without repeating font bytes or changing deterministic fallback.
- Render exact selected pages as transparent or opaque PNG, quality-controlled
  JPEG, or deterministic multi-page TIFF through the shared raster backend.
- Select page ranges and image output options through the common CLI support
  layer and the PowerPoint CLI.

### Fixed

- Honor caller-supplied family aliases through deterministic fallback and
  bounded caches. This hardened equivalent addresses
  [Issue 44](https://github.com/tensorbee/rdocx/issues/44) and the reference
  implementation in [PR 45](https://github.com/tensorbee/rdocx/pull/45).
- Restore bounded reusable layout performance for document load, typing, undo,
  and table mutation while retaining exact shaping, source mappings, and page
  structure. This hardened equivalent addresses
  [Issue 46](https://github.com/tensorbee/rdocx/issues/46).

### Compatibility

All 15 crates.io packages in the shared OOXML and PowerPoint family move
together from 0.5.0 to 0.6.0. This is an intentional pre-1.0 minor boundary.
Callers that build a `FontManager` directly can use the new caller-alias
configuration. Existing callers that do not configure aliases retain the same
deterministic fallback behavior.

The stable Word family remains at its prepared 0.10.0 source boundary while
this incubating family publishes. `rpptx-wasm` is prepared at 0.6.0 but remains
unpublished on crates.io. Python, WASM, npm, and PyPI publication authority is
unchanged.

### Contributors

Atul Sharma maintained the release. Thanks to `@emptinessform` for the caller
font-alias report and reference implementation in
[Issue 44](https://github.com/tensorbee/rdocx/issues/44) and
[PR 45](https://github.com/tensorbee/rdocx/pull/45), and for the editor
performance measurements and migration evidence in
[Issue 46](https://github.com/tensorbee/rdocx/issues/46).

No named external patch landed directly. Each named report or proposal landed
through the hardened equivalent described above so that the current bounded
cache, deterministic fallback, and reusable layout contracts remain intact.

## v0.10.0

### Highlights

The stable Word family adds native RTF, HTML, and OpenDocument Text input,
native RTF and OpenDocument Text output, deterministic EPUB and SVG export,
and ordered compatibility readers that preserve unsupported XML. Layout also
honors caller font aliases and restores bounded editor-scale reuse.

### Added

- Read RTF, HTML5, and OpenDocument Text into the native `Document` facade with
  bounded inputs, ordered diagnostics, and no network access.
- Write deterministic RTF byte streams with stable lossy diagnostics and
  failure-atomic path replacement.
- Write deterministic OpenDocument Text archives with stable lossy diagnostics
  and failure-atomic path replacement.
- Export deterministic EPUB publications that retain supported links, images,
  headings, lists, tables, and accessibility structure while reporting
  unsupported source content.
- Export searchable SVG pages with deterministic fixed-page geometry, images,
  and safe links while reporting unsupported visual content.
- Export exact selected pages as opaque or transparent PNG, quality-controlled
  JPEG, or one deterministic multi-page TIFF through native Word, Python, and
  the general Word and PowerPoint CLI paths.
- Inspect direct table-cell children through `CellRef::items`. This hardened
  equivalent includes the outcome proposed in
  [PR 47](https://github.com/tensorbee/rdocx/pull/47).
- Inspect direct run content through `RunRef::items`, including text, breaks,
  drawings, fields, notes, comments, and preserved XML. This hardened
  equivalent includes the outcome proposed in
  [PR 48](https://github.com/tensorbee/rdocx/pull/48).
- Inspect direct paragraph and hyperlink content without changing established
  flattened accessors. This hardened equivalent includes the outcome proposed
  in [PR 49](https://github.com/tensorbee/rdocx/pull/49).
- Classify retained unsupported body XML through borrowed qualified-name,
  namespace, and child-content facts without inventing source bytes. This
  hardened equivalent includes the outcome proposed in
  [PR 50](https://github.com/tensorbee/rdocx/pull/50).

### Fixed

- Honor caller-supplied font family aliases without duplicating font bytes,
  while retaining deterministic fallback and bounded caches. This hardened
  equivalent addresses
  [Issue 44](https://github.com/tensorbee/rdocx/issues/44) and the reference
  implementation in [PR 45](https://github.com/tensorbee/rdocx/pull/45).
- Restore bounded reusable layout performance for document load, typing, undo,
  and table mutation while retaining exact shaping, source mappings, and page
  structure. This hardened equivalent addresses
  [Issue 46](https://github.com/tensorbee/rdocx/issues/46).
- Reject undecodable ordinary and deleted Word text instead of publishing an
  empty lossy value. This hardened equivalent includes the correction proposed
  in [PR 52](https://github.com/tensorbee/rdocx/pull/52).

### Compatibility

The exact seven-package stable crates.io family moves together from 0.9.0 to
0.10.0. This is an intentional pre-1.0 Rust source boundary. The shared OOXML
and PowerPoint family remains at its published 0.5.0 boundary. Python, WASM,
CLI, npm, and PyPI publication authority is unchanged.

`ST_NumberFormat` now preserves producer-defined values in `Other(String)`.
The enum no longer implements `Copy`, and exhaustive matches must handle the
new value-bearing variant. Callers should borrow or clone numbering formats as
needed and retain unknown values rather than substituting a modeled marker.
This hardened equivalent includes the preservation outcome proposed in
[PR 51](https://github.com/tensorbee/rdocx/pull/51).

External layout backends must continue to recurse through
`PositionedElement::MarkedContent` or use `oxml_layout::walk`, as documented for
v0.9.0. No migration is required for callers that use the high-level document
facade and do not exhaustively match `ST_NumberFormat`.

### Contributors

Atul Sharma maintained the release. Thanks to `@emptinessform` for the caller
font-alias report and reference implementation in
[Issue 44](https://github.com/tensorbee/rdocx/issues/44) and
[PR 45](https://github.com/tensorbee/rdocx/pull/45), and for the editor
performance measurements and migration evidence in
[Issue 46](https://github.com/tensorbee/rdocx/issues/46).

Thanks to `@pedroassumpcao` for the ordered cell, run, paragraph, and hyperlink
reader designs in [PR 47](https://github.com/tensorbee/rdocx/pull/47),
[PR 48](https://github.com/tensorbee/rdocx/pull/48), and
[PR 49](https://github.com/tensorbee/rdocx/pull/49), the unsupported XML facts
in [PR 50](https://github.com/tensorbee/rdocx/pull/50), producer-defined
numbering preservation in [PR 51](https://github.com/tensorbee/rdocx/pull/51),
and fail-closed text decoding in
[PR 52](https://github.com/tensorbee/rdocx/pull/52).

No named external patch landed directly. Each named report or proposal landed
through the hardened equivalent described above so that current namespace,
non-exhaustive API, bounded-allocation, diagnostic, and compatibility contracts
remain intact.

## v0.9.0

### Highlights

The stable Word family adds native package encryption and signing, accessible
and archival PDF output, exact redaction, and editor-scale layout reuse. It
also corrects duplicated shaped text, header and footer delivery, and dense
form table layout while retaining unsupported OOXML.

### Added

- Open and write Microsoft Agile encrypted OOXML packages with authenticated
  AES-256 and SHA-512 processing, bounded inputs, and failure-atomic output.
- Verify and create RSA-SHA256 OPC digital signatures with exact declared part
  and relationship coverage. Certificate-chain trust remains caller policy.
- Emit tagged PDF structure with deterministic marked-content ownership, and
  emit PDF/A-2b or PDF/A-3b with an output intent and conformance metadata.
- Remove exact non-empty literals from Word stories, metadata, chart caches,
  and embedded workbooks through a transactional native redaction API.
- Share immutable font bytes and page frames, transfer reusable layout work
  only across an exact checked context, restart pagination at safe boundaries,
  and retain bounded transactional caches. The hardened equivalent was shaped
  by [Issue 39](https://github.com/tensorbee/rdocx/issues/39),
  [PR 40](https://github.com/tensorbee/rdocx/pull/40), and
  [PR 41](https://github.com/tensorbee/rdocx/pull/41).

### Fixed

- Preserve default, first-page, even-page, inherited, and multi-section header
  and footer text through reopened layout and deterministic PDF output, closing
  [Issue 15](https://github.com/tensorbee/rdocx/issues/15).
- Reshape final Unicode break segments exactly, so spaces, hyphens, ligatures,
  combining text, and CJK do not duplicate source text or glyphs. This resolves
  [Issue 23](https://github.com/tensorbee/rdocx/issues/23).
- Keep nested tables recursive and honor grid-span-aware vertical merges,
  exact and minimum row rules, table-style cascades, paragraph-mark metrics,
  outer border fallbacks, and cell-relative anchors. This hardened equivalent
  addresses [Issue 42](https://github.com/tensorbee/rdocx/issues/42) and
  [PR 43](https://github.com/tensorbee/rdocx/pull/43).

### Compatibility

The exact seven-package stable crates.io family moves together from 0.8.0 to
0.9.0. This is an intentional pre-1.0 Rust source boundary. `FontData.data`
now uses `Arc<[u8]>`, `LayoutResult.pages` now uses
`Vec<Arc<PageFrame>>`, table cells retain ordered `CellBlock` values, and
typed table styles expose additional preserved and conditional properties.
Callers that construct these low-level values must update their literals or
use the provided constructors.

`PositionedElement` remains non-exhaustive. Visible content in
`PageFrame::elements` can be nested under
`PositionedElement::MarkedContent`. External backends must recurse through
`MarkedContent::children` or use `oxml_layout::walk` when visiting page
elements. A wildcard arm that ignores the wrapper can otherwise produce empty
output.

The `agile-encryption` and `digital-signatures` features remain default-off.
The high-level native additions do not expand Python, WASM, or CLI method
surfaces, and those packages remain unpublished on crates.io. The separate
shared OOXML and PowerPoint family remains at its published 0.5.0 boundary.

### Contributors

Atul Sharma maintained the release. Thanks to `@mantissaman` for the
authenticated header and footer report in
[Issue 15](https://github.com/tensorbee/rdocx/issues/15) and the duplicated
text report in [Issue 23](https://github.com/tensorbee/rdocx/issues/23).
Thanks to `@emptinessform` for the break-opportunity diagnosis on Issue 23,
the editor profiling and reference implementations in
[Issue 39](https://github.com/tensorbee/rdocx/issues/39),
[PR 40](https://github.com/tensorbee/rdocx/pull/40), and
[PR 41](https://github.com/tensorbee/rdocx/pull/41), and the dense-form report
and reference implementation in
[Issue 42](https://github.com/tensorbee/rdocx/issues/42) and
[PR 43](https://github.com/tensorbee/rdocx/pull/43). Those contributions
landed directly where noted or through the hardened equivalents described
above.

## rpptx-v0.5.0

### Highlights

The shared OOXML and PowerPoint family adds native package encryption and
digital signatures, accessible PDF structure, and deterministic PDF/A output.
Layout results also retain their largest immutable page and font payloads by
shared ownership, avoiding deep copies when results are cloned or retained.

### Added

- Read and write Microsoft Agile encrypted OOXML packages with authenticated
  AES-256 and SHA-512 output, bounded input processing, and failure-atomic
  publication.
- Verify and create RSA-SHA256 OPC digital signatures with exact declared part
  and relationship coverage. Certificate trust remains caller policy.
- Emit tagged PDF structure with deterministic marked-content ownership,
  document language, titles, outlines, links, and structure destinations.
- Emit deterministic PDF/A-2b and PDF/A-3b files with an output intent,
  conformance metadata, embedded-file relationship rules, and validator-backed
  fixtures.
- Share immutable `FontData` bytes and completed `PageFrame` values through
  `Arc`, so cloning or retaining a layout result keeps those payloads shared.
  This ownership boundary was shaped by
  [Issue 39](https://github.com/tensorbee/rdocx/issues/39),
  [PR 40](https://github.com/tensorbee/rdocx/pull/40), and
  [PR 41](https://github.com/tensorbee/rdocx/pull/41).

### Fixed

There are no user-facing defect corrections unique to the incubating family in
this release. Word-only rendering and document-editing corrections remain on
the separate stable release train.

### Compatibility

All 15 crates.io packages in the shared OOXML and PowerPoint family move
together from 0.4.0 to 0.5.0. This is an intentional pre-1.0 Rust source
boundary. `FontData.data` now uses `Arc<[u8]>`, and `LayoutResult.pages` now
uses `Vec<Arc<PageFrame>>`. Callers that construct those low-level values must
wrap owned data with `Arc::from` or `.into()`. Callers that only inspect values
can continue through deref coercion or `.as_ref()`.

The `agile-encryption` and `digital-signatures` package capabilities remain
default-off. Existing PowerPoint facade behavior requires no migration.
`rpptx-wasm` is prepared at 0.5.0 but remains unpublished on crates.io.

### Contributors

Atul Sharma maintained the release. Thanks to `@emptinessform` for the
authenticated editor profiling and reference implementations in
[Issue 39](https://github.com/tensorbee/rdocx/issues/39),
[PR 40](https://github.com/tensorbee/rdocx/pull/40), and
[PR 41](https://github.com/tensorbee/rdocx/pull/41). Their font-copy and
page-copy measurements informed the shared ownership surface that landed as a
hardened equivalent. The format-specific transfer, pagination, and cache work
remains on the stable release train.

## v0.8.0

### Highlights

The stable Word family now combines native document automation with a complete
layout result that downstream renderers and editors can inspect and reuse.
This release includes structured fields, templates, mail merge, tracked
comparison, watermarks, chart support, source provenance, and bounded relayout
caches while preserving unsupported OOXML.

### Added

- Parse and evaluate Word fields with explicit update policies, including safe
  displayed results for complex fields.
- Create, reply to, resolve, and remove comments and threaded conversations.
  Bind content controls to namespace-aware custom XML without rewriting
  unrelated package data.
- Create bookmarks and resolve `REF` and `PAGEREF` cross-references through
  fields and final pagination.
- Inspect tracked revisions and accept or reject all or a filtered selection
  while preserving unsupported revision XML.
- Render accepted or tracked revision views with visible insertions,
  deletions, and changed paragraphs. Read document-protection intent and its
  recorded enforcement metadata without claiming to enforce the restriction.
- Author Word charts and render them through the shared ChartML model.
- Expand structural templates with conditions and loops, then produce separate
  or sectioned mail-merge documents from flat records.
- Compare documents into deterministic tracked revisions whose accepted and
  rejected views reproduce the edited and original bodies.
- Author and render text or image watermarks through header-scoped VML.
- Expose complete native `WordLayoutResult` bundles with owned font data,
  diagnostics, and result-local source paths for body and related stories.
- Reuse safe paragraph layout, shaping, and font work through bounded caches
  that preserve cold-layout bytes, diagnostics, and current provenance.
- Traverse direct body paragraphs, tables, content controls, and unsupported
  XML in source order through `Document::body_items`.

### Fixed

- Preserve reader-owned unsupported XML, namespace bindings, table facts,
  paragraph borders, hyperlink tooltips, header and footer content, and safe
  field results across opened-document round trips.
- Keep watermark edits, failed relayouts, tracked views, caller fonts, and
  context-sensitive paragraphs from leaking stale cached layout state.

### Compatibility

The seven crates.io packages move together to 0.8.0. The release contains
intentional pre-1.0 Rust source breaks in low-level OOXML and layout structs.
Python, WASM, CLI, and the high-level `rdocx::Document` facade retain their
existing surface contracts. The shared and PowerPoint family remains on its
separate 0.4.0 train.

#### Migration table

| Previous path or crate | Replacement | Compatibility |
|---|---|---|
| `rdocx::Length` | `oxml_core::Length` | `rdocx::Length` remains an exact re-export |
| `rdocx_oxml::{core_properties, error, raw_xml, units}` | The same modules under `oxml_core` | The `rdocx_oxml` paths remain exact re-exports |
| `rdocx_opc` | `oxml_opc` | `rdocx-opc` is a deprecated exact re-export shim, except for the removed Word-only constructors listed below |
| Word-owned image sniffing, sizing, and media naming | `oxml_media::{resolve, probe, ImageFormat, ImageInfo, NativeSize, MediaNamer}` | These shared APIs are available directly from `oxml-media` |
| `rdocx_layout::bundled_fonts` | `oxml_layout::bundled_fonts` | The old module path is removed |
| `rdocx_layout::font::{FontManager, FontMetrics, ShapedText}` | The same types at the `oxml_layout` root | `rdocx_layout::input::FontFile` and `rdocx_layout::FontFile` remain exact re-exports of `oxml_layout::FontFile` |
| `rdocx_layout::error::{LayoutError, Result}` | `oxml_layout::{LayoutError, Result}` | The types also remain exact re-exports at the `rdocx_layout` root |
| `rdocx_layout::line::{InlineItem, LayoutLine, LineBreakParams, LineItem, TextSegment, break_into_lines}` | The same names at the `oxml_layout` root | The old `rdocx_layout::line` module is removed |
| `rdocx_layout::output::{Color, DocumentMetadata, FieldKind, FontData, FontId, GlyphRun, LayoutResult, OutlineEntry, PageFrame, Point, PositionedElement, Rect}` | The same names at the `oxml_layout` root | Types previously exported at the `rdocx_layout` root remain exact re-exports there |
| Exhaustive `TextSegment` and `GlyphRun` literals | Add `source: Option<SourceSpan>` | Use `None` for generated or unattributed text. Word provenance results supply exact result-local node ids and Unicode-scalar ranges |
| `rdocx_pdf` | `oxml_pdf` | `rdocx-pdf` is a deprecated exact re-export shim |
| `rdocx_pdf::raster::{render_page_to_png, render_all_pages}` | `oxml_pdf::{render_page_to_png, render_all_pages}` | The old nested `raster` path is removed. The functions remain available at the `rdocx_pdf` root through the shim |

`rdocx-oxml` and `rdocx-layout` are retained format-specific crates, not
deprecated shims. `rdocx-oxml` continues to own WordprocessingML types.
`rdocx-layout` continues to own the Word flow engine, paginator, blocks,
tables, style resolver, and Word-to-shared conversion boundary. The `rdocx`,
`rdocx-cli`, and `rdocx-html` crate names are unchanged.

### Shared dependencies

New direct users can select the format-neutral crate that owns each surface:

```toml
[dependencies]
oxml-core = "0.4.0"   # Length, units, XML helpers, document properties
oxml-opc = "0.4.0"    # OPC package, relationships, and content types
oxml-media = "0.4.0"  # Image detection, dimensions, and media naming
oxml-layout = "0.4.0" # Layout output, fonts, and line breaking
oxml-pdf = "0.4.0"    # PDF and PNG rendering backends
```

### Breaking API changes

- `rdocx_opc::OpcPackage::new_docx()` and
  `rdocx_opc::ContentTypes::new_docx()` are removed. Use
  `oxml_opc::OpcPackage::new()` or `OpcPackage::with_main_part(...)`, plus
  `oxml_opc::ContentTypes::minimal()`, and add Word-specific defaults and
  overrides at the application boundary.
- `rdocx::Error::Opc` now contains `oxml_opc::OpcError`, and
  `rdocx::Error::Layout` now contains `oxml_layout::LayoutError`. The
  deprecated OPC shim and retained layout facade re-export those exact shared
  types, but code that spells payload paths in exhaustive matches should use
  the shared paths.
- The public `rdocx_layout::line` module is removed. Its shared replacement
  uses `MediaId` instead of relationship-scoped `embed_id` strings for image
  items. `TextSegment` uses `oxml_layout::Underline` and adds `line_gap`.
  `LayoutLine` adds `line_gap`. `LineBreakParams` replaces Word tab stops,
  alignment, and stringly typed line rules with `TabStop`, `Align`, and
  `LineSpacing`, and adds `wrap`.
- `rdocx_layout::engine::layout_paragraph(...)` and
  `rdocx_layout::table::layout_table(...)`, plus
  `rdocx_layout::paginator::paginate(...)` and
  `rdocx_layout::paginator::paginate_sections(...)`, now take a shared
  `MediaRegistry`. Construct it once from `LayoutInput::images` so relationship
  lookup and pagination use the same collision-resolved IDs, bytes, and
  content types.
- `rdocx_layout::AnchoredContent::Image` replaces `embed_id: String` with
  `media_id: MediaId`.
- `rdocx_layout::ParagraphBlock::jc` replaces `Option<ST_Jc>` with
  `Option<oxml_layout::Align>`.
- `PositionedElement` is non-exhaustive, replaces the optional image
  `embed_id` with `MediaId`, and adds `Path` and `Group` variants. External
  matches must include a wildcard arm.
- `PageFrame` is non-exhaustive and adds `background`. Construct it with
  `PageFrame::new(...)` when a default background is wanted.
- `LayoutResult` is non-exhaustive and adds `diagnostics`. Construct it with
  `LayoutResult::new(...)` when an empty diagnostics list is wanted.
- `oxml_layout::TextSegment` and `oxml_layout::GlyphRun` add the required
  `source: Option<SourceSpan>` field. External exhaustive literals must set it
  to `None` unless they own an exact source range. This source change ships in
  the incubating 0.4.0 family and the stable 0.8.0 family. Word callers can use
  `rdocx_layout::layout_document_with_provenance` or its deterministic variant
  to receive `WordLayoutResult`, resolve result-local nodes to
  `WordSourcePath`, and interpret exclusive character ranges as Unicode scalar
  indices in the recorded revision view.
- The nested `rdocx_pdf::raster` module is removed. Import its two rendering
  functions from the `oxml_pdf` root or from the compatible `rdocx_pdf` root.

### Media behavior and additive API

Word media insertion now detects the image format from its bytes before using
the filename extension. It allocates the next numeric media suffix after the
greatest occupied suffix, so gaps do not overwrite an existing part.

`rdocx::Document::add_picture_auto(image_data, image_filename)` adds an image
at its intrinsic size. It uses declared per-axis DPI when valid and a 72 DPI
fallback otherwise. If dimensions cannot be determined, it returns
`rdocx::Error::UnavailableImageDimensions` before changing the document.

### Contributors

Thanks to Pedro Assumpcao for the ordered-body contribution in PR 36 and the
reader compatibility work included in this release. Thanks to `@emptinessform`
for the Issue 37 complete-layout report and the Issue 39 relayout measurements
and cache proposal.

## rpptx-v0.4.0

### Highlights

The complete shared OOXML and PowerPoint family moves together to 0.4.0. This
is the first release to publish `oxml-chart`, making the typed ChartML model,
authoring surface, and renderer available from its format-neutral home.

### Added

- `oxml-chart` now owns shared ChartML parsing, editing, authoring, and render
  geometry. `rpptx-chart` remains an exact compatibility re-export.
- `oxml-layout` glyph runs can carry exact `SourceSpan` provenance through
  shaping and line splitting, with generated or transformed text left
  truthfully unattributed.
- Normal host-font layout reuses a bounded process font snapshot, file-backed
  bytes, and exact-key shaping results. Deterministic and caller-font paths
  remain isolated from that state.

### Fixed

- Bounded OPC reads reject oversized declared ZIP entry counts before the ZIP
  index is constructed, and retain the configured byte and entry ceilings
  throughout package access.
- Deterministic PDF output now writes font, Unicode-map, and image resources in
  stable order, so identical inputs produce identical bytes.

### Compatibility

This is an intentional pre-1.0 source boundary. External exhaustive literals
for `TextSegment` and `GlyphRun` must add `source: None` unless they own an
exact `SourceSpan`. Existing `rpptx-chart` imports remain valid through the
exact re-export, while new direct users should depend on `oxml-chart`.

Normal system-font discovery is now a process-lifetime snapshot. Installing,
removing, or replacing host fonts requires a process restart. Deterministic and
caller-provided font behavior is unchanged. `rpptx-wasm` is prepared at 0.4.0
but remains unpublished on crates.io.

### Contributors

Atul Sharma maintained the release. `@emptinessform` supplied the provenance
and cache reports behind Issues 38 and 39. Pedro Assumpcao
(`@pedroassumpcao`) contributed bounded OPC reads in PR 33 and carried the
entry-limit hardening through PR 34. Jon Stokes (`@jonstokes`) authored the
ZIP entry-admission hardening commit integrated by PR 34.
