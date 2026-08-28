# Changelog

## Unreleased

No changes have been recorded since the v0.11.0 preparation.

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
