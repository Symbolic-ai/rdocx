# F-179, ODT reader

**Status**: completed
**Sprint**: S55
**Size**: L
**Depends on**: none

## Problem

The facade cannot open OpenDocument Text. ODT is a ZIP package but not an OPC
package, so routing it through `OpcPackage` would incorrectly require
`[Content_Types].xml` and relationship parts
(`crates/oxml-opc/src/package.rs:169`). The existing private RTF importer shows
the correct ownership boundary: parse a foreign format inside `rdocx`, then
project it directly into the one editable `Document` tree
(`crates/rdocx/src/rtf.rs:30`).

M18 requires supported text, formatting, tables, lists, and images. It also
requires safe lossy skips to identify what was dropped and a source-built
structural comparison against an exactly pinned LibreOffice conversion
(`docs/hld/14-development-backlog.md:1439` and
`docs/hld/14-development-backlog.md:1493`).

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package", "Part naming", and
  "Media".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/14-development-backlog.md`, "Milestone 18, Format breadth" and
  "F-179, ODT reader".
- `docs/hld/15-build-and-toolchain.md`, the pinned LibreOffice runtime and
  package dependency policy.
- ODF 1.3 Part 2, "Packages", and Part 3, "OpenDocument Schema", for package,
  text, style, list, table, drawing, and length semantics.

## Approach

Add one private `odt` module to `rdocx`. Reuse the workspace `zip` dependency
directly from the facade and the existing `quick-xml` and `oxml-media`
dependencies. Do not generalize `OpcPackage`, add an ODT crate, or retain a
second document model.

Expose an additive native pre-1.0 API:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdtDiagnostic {
    pub path: String,
    pub message: String,
}

pub struct OdtReadResult {
    pub document: Document,
    pub diagnostics: Vec<OdtDiagnostic>,
}

impl Document {
    pub fn from_odt_bytes(bytes: &[u8]) -> Result<OdtReadResult>;
    pub fn from_odt_bytes_with_limits(
        bytes: &[u8],
        limits: PackageReadLimits,
    ) -> Result<OdtReadResult>;
    pub fn open_odt<P: AsRef<Path>>(path: P) -> Result<OdtReadResult>;
}
```

Add `Error::Odt { part: Option<String>, offset: u64, message: String }`. The
result carries the fresh converted document and every stable lossy-conversion
diagnostic. Python, WASM, and CLI gain no implicit ODT entry points.

Index the archive before parsing any XML. Enforce the supplied entry, part,
and total expansion bounds, with conservative bounded defaults for the simple
constructors. Reject unsafe, absolute, backslash, NUL, traversal, duplicate,
non-file, unsupported-compression, or encrypted entries rather than
normalizing them. Require the exact root `mimetype` value
`application/vnd.oasis.opendocument.text` and `content.xml`. Parse
`META-INF/manifest.xml` when present, and reject encrypted referenced content.
Read only styles, content, manifest, and referenced image parts after the
complete index is validated.

Parse XML with `quick_xml::NsReader` by expanded namespace URI and local name.
Reject DTDs, unresolved entities, malformed XML, depth above 256, more than
100,000 projected blocks or runs, 10,000 rows, 256 columns, 50,000 cells, or
10,000 diagnostics. Consume unsupported subtrees completely and emit one
stable source-path diagnostic while preserving supported siblings. ODT bytes
are a one-way conversion source, so they are not inserted as opaque raw OOXML.

Read common, default, named, and automatic styles before body projection.
Resolve each family through its default, parent chain, and local properties,
rejecting cycles and diagnosing missing parents. Materialize effective text
and paragraph formatting directly onto Word paragraphs and runs. Support font
family and size, bold, italic, underline, strike, color, highlight,
superscript, subscript, alignment, indents, paragraph spacing, and exact or
percentage line height.

Project `text:p`, headings, spans, repeated spaces, tabs, line breaks, and
nested spans in source order. Project bullet and numbered lists through bounded
nine-level Word definitions. Diagnose unsupported continuation and start
semantics. Expand bounded repeated table columns, rows, and cells, compute the
logical grid first, preserve multiple cell paragraphs, and map representable
row and column spans to vertical merges and grid spans.

Resolve only safe package-relative image targets. Sniff their bytes, use a
positive frame width and height when present, and otherwise use intrinsic size
with the existing 72 DPI fallback. Embed supported image bytes through the
normal Word media path. Missing, unsafe, external, encrypted, malformed, or
unsupported image content produces a path-aware diagnostic or a fatal error
according to whether visible supported siblings can safely continue.

Build and validate a fresh candidate before returning it. An archive, XML,
style, or projection error exposes no partially populated document.

## Rejected alternatives

- `OpcPackage` is an OOXML package owner and cannot correctly represent ODT
  package semantics.
- A new ODT crate adds a publication boundary for one facade consumer.
- A retained ODT object model duplicates the editable Word tree and makes
  mutation ownership ambiguous.
- Comparing LibreOffice-produced DOCX bytes would fail on deliberate prefix,
  relationship, and serialization differences.
- Smuggling unsupported ODT XML into DOCX would pretend to preserve content
  that no Word consumer understands.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `odt_archive_rejects_unsafe_duplicate_encrypted_and_oversized_entries` | Every archive path, compression, encryption, entry, part, and total bound fails before partial projection. |
| unit | `odt_styles_resolve_defaults_parents_and_automatic_overrides` | Namespace aliases, style families, inheritance order, cycles, missing parents, text properties, and paragraph properties have exact outcomes. |
| unit | `odt_reader_rejects_malformed_or_unbounded_xml` | DTD, entity, namespace, depth, repeat, block, run, row, column, cell, and diagnostic failures are bounded and contextual. |
| regression | `unsupported_odt_content_is_diagnosed_without_dropping_supported_siblings` | One exact source path is reported per dropped subtree or property and adjacent text, lists, tables, and images survive. |
| integration | `odt_reader_projects_text_formatting_lists_tables_and_images` | One source-built ODT package produces the exact typed body order, effective formatting, list kind and level, table grid and spans, image bytes, and dimensions, then saves and reopens equally. |
| differential | `odt_reader_matches_pinned_libreoffice_structure` | The same source-built ODT converted by the exact LibreOffice build has an equal normalized structural record. |

The **test gate** is differential. An ODT converted here matches the pinned
LibreOffice conversion structurally. The gate constructs the ODT ZIP, XML, and
image in source, requires
`LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb`, runs headless with
an isolated user profile, opens the resulting DOCX, and compares normalized
body order, effective text and paragraph formatting, list kind and level,
table grid and spans, and image bytes and dimensions. It does not compare
package bytes, relationship ids, part names, or prefixes.

Tests stay in the new module's unit tests and the existing
`crates/rdocx/tests/integration_test.rs`, so no new test binary is added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Read by expanded namespace, reject
  unsafe archive names, consume unknown source subtrees exactly once, save and
  reopen the projected DOCX, and prove its typed schema order. ODT has no
  writer or retained source subtree in this story.
- **Crate dependency graph**. Read `docs/hld/03-architecture.md`. Add the
  already pinned `zip` dependency only to `rdocx`, without changing any
  `oxml-*` edge.
- **Public API of a published crate**. The result, diagnostic, error, limits
  overload, and three constructors are additive native pre-1.0 API. Run the
  verified `rdocx` package dry run and archive-size assertion.
- **WASM or PyO3 bindings**. Python, WASM, and CLI gain no methods, but `rdocx`
  gains a direct archive dependency. Run both wasm32 checks and retain both
  Python binding exclusions in workspace tests.
- **A new trait, generic parameter, crate, module or file**. The private
  `crates/rdocx/src/odt.rs` module needs explicit approval. No trait, generic,
  crate, or second model is introduced.
- **An external oracle comparison**. Follow
  `.claude/skills/differential-testing.md`. Use the repository's exact pinned
  LibreOffice installer and identity, construct the ODT in source, compare the
  normalized tree, and assert every intentional divergence.

## Hash harness

Expected unchanged, 49 of 49. Existing samples do not open ODT. Any delta
blocks integration and the baseline is not re-recorded.

## Implementation checklist

- [x] Add the approved private module, direct workspace ZIP edge, facade error,
      result, diagnostics, limits overload, and native constructors.
- [x] Validate the complete bounded archive index, required ODT parts, and
      manifest encryption state before parsing content.
- [x] Parse namespace-aware styles and body XML with exact bounds and stable
      source paths.
- [x] Resolve effective formatting and project text, lists, tables, and images
      into the existing Word ownership tree.
- [x] Diagnose every safe lossy skip while retaining supported siblings.
- [x] Add source-built unit, integration, save, reopen, and exact pinned
      LibreOffice structural differential coverage.
- [x] Run scoped facade, package, WASM, full verification, and unchanged-harness
      checks.

## Open questions

- Resolved. Create `crates/rdocx/src/odt.rs`. It keeps the archive, XML, style
  resolution, and projection in one readable owner. The direct `zip` edge
  reuses the existing workspace version and does not add a new external
  package.
