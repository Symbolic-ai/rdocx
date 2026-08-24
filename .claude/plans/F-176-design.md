# F-176, RTF reader

**Status**: completed
**Sprint**: S54
**Size**: L
**Depends on**: none

## Problem

The public facade opens only DOCX packages through `Document::open` and
`Document::from_bytes` (`crates/rdocx/src/document.rs:499` and
`crates/rdocx/src/document.rs:513`). It has no RTF grammar, destination stack,
code-page decoder, or projection from Word-written RTF into its existing typed
paragraph, run, table, list, and image ownership tree. The facade already has
the target construction operations, including tables at
`crates/rdocx/src/document.rs:1249`, pictures at
`crates/rdocx/src/document.rs:1376`, and list definitions at
`crates/rdocx/src/document.rs:2323`.

M18 requires the subset Word writes for text, formatting, tables, lists, and
images. It also requires every lossy conversion to return a diagnostic naming
what was dropped (`docs/hld/14-development-backlog.md:1439`).

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and "Facade conventions".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "Binding tests".
- `docs/hld/14-development-backlog.md`, "Milestone 18, Format breadth" and
  "F-176, RTF reader".
- Microsoft RTF Specification 1.9.1, "Basic Entities", "Contents of an RTF
  File", "Header", "Character Set", "Unicode RTF", "Font Table", "Color
  Table", "List Tables", "Paragraph Formatting Properties", "Table
  Definitions", "Font Formatting Properties", and "Pictures".

## Approach

Add one private `rtf` module to `rdocx`. Its scanner will emit group boundaries,
control words with optional signed parameters, control symbols, literal text,
and hexadecimal bytes. A stack of concrete parser state values will track the
current destination, `\ucN` fallback width, font and code-page selection, run
properties, paragraph properties, list state, table state, and picture state.
There is no new trait or generic parameter.

Parse font, colour, list, and list-override destinations into bounded lookup
tables. Decode `\uN` values and skip exactly the declared fallback characters.
Decode `\'hh` byte runs through an explicit `\ansicpgN` and font charset map,
using one direct `encoding_rs` dependency rather than a partial home-grown
decoder. Unknown starred destinations are skipped as groups. Unsupported
content that can be skipped safely produces one stable diagnostic with its byte
offset and destination.

Project completed paragraphs, runs, tables, lists, and `\pict` groups directly
into a new `Document`. Preserve source order and reuse the existing image
sniffer, relationship writer, list definitions, and formatting setters. Reject
unbalanced groups, malformed numeric controls, invalid hex, invalid Unicode
fallback state, and invalid image payloads through a dedicated `Error::Rtf`
variant.

Expose an additive pre-1.0 facade:

```rust
pub struct RtfDiagnostic {
    pub offset: usize,
    pub destination: Option<String>,
    pub message: String,
}

pub struct RtfReadResult {
    pub document: Document,
    pub diagnostics: Vec<RtfDiagnostic>,
}

impl Document {
    pub fn from_rtf_bytes(bytes: &[u8]) -> Result<RtfReadResult>;
    pub fn open_rtf<P: AsRef<Path>>(path: P) -> Result<RtfReadResult>;
}
```

The result is not a forwarding wrapper. It carries both the converted document
and the required lossy-conversion evidence. No builder is introduced.

## Rejected alternatives

- A new `rdocx-rtf` crate adds a publication boundary for one facade consumer.
- Building a second long-lived RTF document model duplicates the typed Word
  ownership tree and makes later editing ambiguous.
- Decoding only Windows-1252 does not meet the code-page scope.
- Committing binary RTF or DOCX fixtures violates the repository fixture rule.
- Silently skipping unsupported destinations violates the M18 diagnostic gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `rtf_groups_restore_formatting_and_destination_state` | Nested groups restore run, paragraph, code-page, and skip state exactly. |
| unit | `rtf_unicode_skips_the_declared_fallback_width` | Signed UTF-16 values, surrogate pairs, `\ucN`, hex bytes, escaped braces, and backslashes decode correctly. |
| unit | `rtf_malformed_input_fails_without_partial_document` | Unbalanced groups, invalid controls, invalid hex, and oversized tables or pictures are rejected. |
| integration | `rtf_reader_projects_word_text_formatting_tables_lists_and_images` | One source-encoded RTF input yields the expected typed paragraphs, runs, table cells, numbering, and image metadata. |
| differential | `rtf_reader_matches_the_pinned_word_docx_structure` | The normalized typed tree matches the same source-encoded RTF opened and saved as DOCX by Microsoft Word 16.104 build 16.104.25121423. |
| regression | `unsupported_rtf_destinations_are_diagnosed_without_dropping_supported_siblings` | Every lossy skip has one stable diagnostic and supported adjacent content survives. |

The **test gate** is differential. An RTF file converted to DOCX here matches
the same file opened and saved as DOCX by the pinned oracle, compared
structurally. The checked test stores source RTF and normalized expected records
as source, never an opaque binary. An ignored regeneration gate first asserts
the exact Word version and build. It compares the parsed tree, not DOCX bytes.

Tests stay in the existing `rdocx` unit module and
`crates/rdocx/tests/integration_test.rs`, so no new test binary is added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. RTF has group and destination order
  rather than XML `xsd:sequence`. The extra checks prove destination order,
  reopen the generated DOCX, and confirm existing unmodelled OOXML stays
  preserved.
- **Public API of a published crate**. This is additive pre-1.0 `rdocx` API.
  Run the verified workspace packaging dry run and require every archive to
  remain below 10 MiB.
- **A new module or file**. `crates/rdocx/src/rtf.rs` needs explicit approval.
  The module keeps the grammar and projection together. No new trait, generic,
  or crate is introduced.
- **An external oracle comparison**. Follow
  `.claude/skills/differential-testing.md`, pin Microsoft Word 16.104 build
  16.104.25121423 in the harness, construct the input in source, compare the
  structural tree, and record every deliberate divergence as an assertion.

## Hash harness

Expected unchanged, 49 of 49. Existing sample generators do not read RTF.
Any delta is unrelated and blocks the sprint. Do not edit
`scripts/hash_baseline.json`.

## Implementation checklist

- [x] Add the bounded scanner, group stack, destination handling, and RTF error.
- [x] Decode Word code pages, Unicode controls, and fallback bytes.
- [x] Project fonts, colours, formatting, paragraphs, tables, and lists.
- [x] Decode and embed supported picture destinations.
- [x] Return stable diagnostics for every safe lossy skip.
- [x] Add the source-encoded structural differential gate and focused cases.
- [x] Run focused `rdocx`, oracle-checked structural, and unchanged-harness checks.
  The package dry run passed earlier in the worker and was not retried after
  the approval reviewer rejected the retry and prohibited workarounds.

## Open questions

- Resolved. Create `crates/rdocx/src/rtf.rs`, add the direct `encoding_rs`
  dependency, and use the proposed diagnostic-bearing byte and path API.
