# F-181, EPUB export

**Status**: approved
**Sprint**: S56
**Size**: M
**Depends on**: none

## Problem

The native Word facade can emit complete HTML, an HTML fragment, and Markdown
through `rdocx-html` at `crates/rdocx/src/document.rs:3717`, but it cannot
package that reflowable structure as EPUB 3. The document already exposes flat
headings and a hierarchical outline at `crates/rdocx/src/document.rs:4208`, and
the outbound emitter already preserves supported headings, lists, tables,
hyperlinks, and inline images at `crates/rdocx-html/src/lib.rs:47`.

M18 requires every lossy conversion to name what it dropped. The EPUB writer
therefore needs one deterministic archive, one source-ordered navigation and
spine projection, and stable diagnostics rather than an HTML ZIP assembled by
callers.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package" and deterministic saves.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and source-built fixtures.
- `docs/hld/14-development-backlog.md`, "Milestone 18, Format breadth" and
  "F-181, EPUB export".
- `docs/hld/15-build-and-toolchain.md`, "Dependency policy" and pinned external
  validation tools.
- EPUB 3.3 package and navigation requirements, validated by EPUBCheck 5.3.0.

## Approach

Add a private `rdocx` EPUB writer that consumes the current document tree,
metadata, resolved image relationships, and the established `rdocx-html`
escaping and semantic projection. It writes the required uncompressed first
`mimetype` entry, `META-INF/container.xml`, one package document, one navigation
document, deterministic XHTML spine items, shared CSS, and referenced images.
Archive entry order, timestamps, compression choices, identifiers, generated
paths, and XML attribute order stay deterministic.

Top-level outline entries define spine boundaries. Content before the first
top-level heading becomes an optional front-matter item. Each top-level heading
starts one XHTML item containing its nested headings and following content until
the next top-level heading. A document without headings produces one document
item. The navigation tree preserves `Document::document_outline()` order and
links to stable heading anchors in those items.

Expose additive native APIs:

```rust
pub struct EpubDiagnostic {
    pub path: String,
    pub message: String,
}

pub struct EpubWriteResult {
    pub bytes: Vec<u8>,
    pub diagnostics: Vec<EpubDiagnostic>,
}

impl Document {
    pub fn to_epub_bytes(&self) -> Result<EpubWriteResult>;
    pub fn save_epub<P: AsRef<Path>>(&self, path: P)
        -> Result<Vec<EpubDiagnostic>>;
}
```

The byte path is bounded before allocation growth. The path method stages the
complete archive beside the destination and publishes it atomically. Metadata
uses the document title and author when present, stable explicit fallbacks when
absent, and no clock or random identifier. Unsupported or lossy source content
adds one location-aware diagnostic while supported siblings remain in the
publication. Python, WASM, and CLI surfaces remain unchanged.

## Rejected alternatives

- Treating `Document::to_html()` as the EPUB body omits package, navigation,
  metadata, spine, media, and diagnostic contracts.
- Paginating through `PageFrame` would create fixed-layout EPUB and lose the
  requested reflowable document structure.
- Adding a new EPUB crate would introduce a publication and dependency boundary
  for one native facade consumer.
- Random UUIDs and current timestamps would make identical exports differ.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `epub_spine_and_navigation_follow_the_document_outline` | A source-built document produces front matter and source-ordered chapter items whose spine and nested navigation match the outline exactly, and EPUBCheck 5.3.0 reports no errors. |
| unit | `epub_archive_starts_with_uncompressed_mimetype_and_is_deterministic` | Required entry order, stored mimetype bytes, stable timestamps, names, metadata, and two identical exports are exact. |
| integration | `epub_preserves_reflowable_text_lists_tables_links_and_images` | XHTML and media entries retain the supported outbound HTML structure and relationship-resolved image bytes. |
| regression | `epub_reports_lossy_content_without_dropping_supported_siblings` | Each unsupported source item has one stable diagnostic and adjacent supported content remains. |
| regression | `epub_save_replaces_an_existing_file_atomically` | Serialization failure or staging collision cannot truncate the prior destination. |

The **test gate** is regression. A generated EPUB passes pinned EPUBCheck 5.3.0
and its spine matches the document outline. Fixtures are assembled in source
inside existing test targets, with no checked-in binary EPUB.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Assert required EPUB child and archive
  order, namespace-correct XML, deterministic serialization, and write-open
  structural preservation. Reopen the source DOCX to prove retained unmodelled
  XML is unchanged.
- **Public API of a published crate**. This is additive pre-1.0 `rdocx` API.
  Run the verified 22-package dry run and archive size assertions, and confirm
  Python, WASM, and CLI surfaces remain unchanged.
- **A new module or file**. A dedicated private `epub` module keeps archive and
  XML machinery out of `document.rs`. It requires consolidated approval before
  implementation.
- **External oracle comparison**. Pin EPUBCheck 5.3.0 from the W3C release,
  record its identity, and run it only as a development and CI validator. It is
  not a runtime or packaged dependency.

## Hash harness

Expected unchanged across all 49 entries. Existing samples do not export EPUB.
Any delta blocks integration and the baseline is not re-recorded.

## Implementation checklist

- [ ] Add bounded deterministic EPUB archive and XML emission.
- [ ] Split reflowable content and navigation at approved outline boundaries.
- [ ] Reuse outbound HTML semantics, escaping, media, and hyperlink resolution.
- [ ] Add diagnostic-bearing byte output and atomic path output.
- [ ] Pin and run EPUBCheck 5.3.0 without adding a runtime dependency.
- [ ] Add source-built regression, integration, determinism, and failure tests.
- [ ] Run public API packaging, dependency, full verify, and unchanged-harness checks.

## Open questions

Resolved. Top-level headings split the spine. Pre-heading content becomes front
matter, and documents without headings produce one item. The implementation may
add private `crates/rdocx/src/epub.rs` and reuse the existing `zip` dependency.
