# F-180, ODT writer

**Status**: completed
**Sprint**: S56
**Size**: L
**Depends on**: F-179

## Problem

The native Word facade can import ODT through three constructors, but it has no
inverse byte or path writer. The existing ODT module exports only
`OdtDiagnostic` and `OdtReadResult`, and its `Document` implementation ends
after the read APIs at `crates/rdocx/src/odt.rs:36` and
`crates/rdocx/src/odt.rs:80`. F-180 requires text, formatting, tables, lists,
and images to survive a write followed by the completed F-179 reader.

ODT is a ZIP package but is not OPC, so the writer stays in the native `rdocx`
facade and does not generalize `OpcPackage` (`docs/hld/03-architecture.md:97`,
`docs/hld/04-opc-and-packaging.md:24`). The milestone also requires every
lossy conversion to identify what it dropped. The existing Word tree retains
body order and unsupported XML, so the writer can preserve supported siblings
and report stable source locations outside the F-179 fidelity boundary.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package", including deterministic
  archive output and the ODT ZIP versus OPC boundary.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and the F-179 normalized
  structural comparison.
- `docs/hld/14-development-backlog.md`, "Milestone 18, Format breadth" and
  "F-180, ODT writer".
- `docs/hld/15-build-and-toolchain.md`, "Dependency policy".
- ODF 1.3 Part 2, package, manifest, and MIME requirements.
- ODF 1.3 Part 3, document content, text, list, table, frame, and style
  requirements.

## Approach

Extend the existing private `crates/rdocx/src/odt.rs` module. Do not add a
crate, module, file, feature, dependency, ODT object model, or `OpcPackage`
mode. `rdocx` already directly depends on `zip`, `quick-xml`, and `oxml-media`.

Expose additive native pre-1.0 API and re-export the result beside the existing
ODT types:

```rust
pub struct OdtWriteResult {
    pub bytes: Vec<u8>,
    pub diagnostics: Vec<OdtDiagnostic>,
}

impl Document {
    pub fn to_odt_bytes(&self) -> Result<OdtWriteResult>;
    pub fn save_odt<P: AsRef<Path>>(&self, path: P)
        -> Result<Vec<OdtDiagnostic>>;
}
```

Reuse `OdtDiagnostic { path, message }` and `Error::Odt`. The byte API does not
mutate the source document. The path API serializes completely, stages beside
the destination, syncs, and publishes through the existing portable
replacement primitive.

One concrete `OdtWriter<'a>` walks `Document::document.body.content` in source
order. Its first pass resolves effective paragraph and run styles, allocates
deterministic automatic paragraph, text, and list style names, allocates
`Pictures/imageN.ext` names in encounter order, validates table spans, and
records deduplicated path-aware diagnostics. Every emitted collection uses
ordered maps or explicitly sorted vectors.

The second pass writes fixed-prefix ODF 1.3 `content.xml` with automatic styles
before the body. Materialize the exact F-179-supported formatting rather than
retaining Word style identifiers. Paragraph styles cover alignment, spacing,
indentation, and line height. Text styles cover one effective font family,
size, bold, italic, basic underline, strike, RGB foreground and highlight,
superscript, and subscript. Unsupported source properties are diagnosed while
the supported projection continues.

Write text with XML escaping and ODF whitespace elements for repeated or
boundary spaces, tabs, and line breaks. Flatten visible fallbacks only when the
source has a safe display value and diagnose that loss. Drop unsupported
fields, notes, comments, bookmarks, revisions, content controls, anchored
drawings, and preserved raw XML with one stable diagnostic while retaining
supported siblings.

Convert consecutive Word list paragraphs into valid nested `text:list`,
`text:list-item`, and paragraph structure. Use the resolved numbering
definition and levels 0 through 8. Preserve bullet versus numbered kind.
Producer numbering semantics that F-179 cannot recover are diagnosed.

Map tables to table, row, cell, and covered-cell elements. Translate
`grid_span` to column spans and valid vertical merge restarts plus continuations
to row spans and covered cells. Preserve multiple cell paragraphs. Reject
malformed or overlapping spans and diagnose unsupported table properties.

Resolve inline picture relationships through the document package. Preserve
byte-sniffed PNG, JPEG, GIF, BMP, and WebP media that F-179 can probe, allocate
canonical extensions and MIME types through `oxml-media`, and emit positive
frame dimensions from the existing truncating EMU path. Missing, external,
malformed, unsupported, anchored, or non-positive pictures are diagnosed and
omitted without dropping adjacent runs.

Build the deterministic ZIP in this order: stored first `mimetype` with no
extra field, deflated `content.xml`, images in allocated path order, then
deflated `META-INF/manifest.xml`. The manifest contains `/`, `content.xml`, and
exactly one entry for every image. Cap XML, media, total output, entry count,
and diagnostics at the existing ODT defaults. Two writes of the same document
must be byte-identical.

## Rejected alternatives

- Generalizing `OpcPackage` is wrong because ODT has no OPC content types or
  relationship graph.
- Adding an ODT crate or retained model introduces a second ownership tree and
  publication boundary for one facade consumer.
- Writing from `PageFrame` loses editable lists, tables, and source formatting.
- Supporting properties F-179 cannot read back would exceed the gate and hide
  loss. Diagnose them instead.
- Comparing package bytes with LibreOffice would test local serialization
  choices rather than the declared structural round trip.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `odt_writer_emits_conforming_deterministic_package` | `mimetype` is first and stored, required entries and fixed-prefix XML are ordered, output bounds apply, and two writes are byte-identical. |
| unit | `odt_writer_materializes_effective_formatting_and_whitespace` | Supported inherited formatting and ODF whitespace values survive exactly. |
| unit | `odt_writer_emits_nested_lists_table_spans_and_images` | List levels, horizontal and vertical spans, multiple cell paragraphs, media types, bytes, and frame sizes map correctly. |
| round-trip, gate | `odt_writer_round_trip_preserves_supported_document_content` | Write then F-179 reopen preserves body order, text, effective formatting, lists, table grid and spans, and image bytes and dimensions. |
| regression | `unsupported_document_content_is_diagnosed_without_dropping_supported_odt_siblings` | Every unsupported source category has one stable diagnostic and supported siblings survive. |
| regression | `save_odt_preserves_existing_destination_when_staging_fails` | Serialization or staging failure cannot truncate the destination. |

The **test gate** is round-trip. Text, formatting, tables, lists, and images
survive a write followed by F-179. Fixtures are assembled in source. Focused
tests stay in `odt.rs`, and the public gate joins the existing
`crates/rdocx/tests/integration_test.rs` binary.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Unit conversion**. Read `docs/hld/01-glossary.md`, "Units", and the pinned
  truncation rule in `CLAUDE.md`. Preserve integer Twips, half-point, and EMU
  behavior with exact boundary assertions.
- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Assert fixed namespaces, ODF child and
  ZIP order, deterministic bytes, write-read preservation, and no source DOCX
  or raw XML mutation.
- **Public API of a published crate**. This additive pre-1.0 `rdocx` API enters
  v0.10.0. Run the patched 22-package dry run, archive size assertions, and
  confirm Python, WASM, and CLI surfaces remain unchanged.

No new crate edge, file, module, feature, binding, oracle, file move, or hash
baseline is proposed.

## Hash harness

Expected unchanged across all 49 entries. Existing samples do not export ODT.
Any delta blocks integration and the baseline is not re-recorded.

## Implementation checklist

- [x] Add `OdtWriteResult`, byte and atomic path APIs, and the public re-export.
- [x] Allocate deterministic automatic styles, list styles, media, and diagnostics.
- [x] Emit fixed-prefix ODF content for supported text, formatting, lists, tables, and images.
- [x] Translate valid table spans and reject malformed overlap.
- [x] Build the bounded manifest and deterministic ZIP.
- [x] Diagnose every unsupported item without mutating the source document.
- [x] Add source-built round-trip, determinism, bounds, diagnostics, and atomic-save coverage.
- [x] Run scoped checks, all risk riders, full verification, packaging, and the unchanged harness.

## Open questions

None. The completed F-179 reader is the writer fidelity boundary, the gate
reopens through F-179, only native Rust gains methods, and ODT remains separate
from OPC.
