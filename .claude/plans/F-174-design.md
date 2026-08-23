# F-174, PDF/A conformance

**Status**: completed
**Sprint**: S53
**Size**: M
**Depends on**: F-173

## Problem

`oxml-pdf::render_to_pdf` always returns ordinary PDF bytes at
`crates/oxml-pdf/src/lib.rs:13`. The writer emits a catalog and basic document
information at `crates/oxml-pdf/src/writer.rs:478`, but it has no conformance
selection, XMP identification metadata, output intent, embedded ICC profile,
or preflight for features prohibited by PDF/A-2b and PDF/A-3b.

The Word and PowerPoint facades call this unconditional path at
`crates/rdocx/src/document.rs:3486` and `crates/rpptx/src/lib.rs:487`. A caller
cannot request an archival profile or distinguish an unsupported document from
a conforming one. F-174 must retain the F-173 tagged structure instead of
silently replacing it with an untagged archival file.

## Spec reference

- ISO 19005-2, PDF/A-2, level B requirements.
- ISO 19005-3, PDF/A-3, level B requirements.
- ISO 32000-1, metadata streams, output intents, embedded files, and prohibited
  actions.
- `docs/hld/08-rendering-spec.md`, "The PDF backend" and "The renderer's
  input".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and "WASM".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering",
  "Packaging", and "Dependency policy".

## Approach

Add `PdfConformance::{PdfA2b, PdfA3b}` and `PdfOptions` to `oxml-pdf`, plus
`render_to_pdf_with_options(&LayoutResult, PdfOptions) -> Result<Vec<u8>, PdfError>`.
Keep `render_to_pdf` as the existing ordinary, infallible compatibility path.
Add native Word and Presentation methods that request one explicit profile.
Python, WASM, and CLI APIs remain unchanged.

For either profile, validate before writing that every used font is embedded,
all colours have an allowed device-independent interpretation through the
output intent, encryption and forbidden actions are absent, metadata is
complete, and F-173 structure data remains internally consistent. Write
deterministic XMP containing `pdfaid:part` 2 or 3 and
`pdfaid:conformance` B, an `/OutputIntents` entry, and one embedded sRGB ICC
profile. PDF/A-3b permits associated files only through a later explicit API.
This story does not manufacture attachments.

Validate deterministic fixtures with veraPDF 1.30.2 using profiles `2b` and
`3b`. Also run `ua1` over the tagged fixture produced through both archival
paths so conformance does not regress semantic structure.

## Rejected alternatives

- Declaring PDF/A only in metadata would create a labelled but non-conforming
  file.
- Making all PDF output archival would change existing callers and reject
  documents that ordinary PDF can represent.
- Downloading an ICC profile at runtime would make output dependent on network
  state and mutable external bytes.
- Using a library validator as production logic would add a Java-sized runtime
  dependency to a small Rust renderer.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `pdfa_profiles_emit_matching_xmp_and_output_intent` | Part, conformance level, metadata stream, ICC profile, catalog linkage, and identifiers are deterministic and profile-correct. |
| regression | `pdfa_rejects_prohibited_or_incomplete_features_before_output` | Missing font programs, invalid structure references, forbidden actions, and unsupported colour state return a named error without partial bytes. |
| regression | `ordinary_pdf_api_remains_byte_identical` | Existing `render_to_pdf`, Word PDF, and Presentation PDF calls do not change. |
| regression | `pdfa_retains_tagged_structure_tree` | Both profiles preserve `/StructTreeRoot`, marked content, heading, list, table, and alternate-text semantics. |
| differential | `pdfa_2b_and_3b_pass_verapdf` | veraPDF 1.30.2 profiles `2b` and `3b` pass, and `ua1` also passes on the same tagged fixture. |

The test gate is **regression**. A rendered PDF passes a conformance check for
the declared level.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout and rendering: re-read `docs/hld/08-rendering-spec.md`. Generate all
  baselines in deterministic font mode and require ordinary output to remain
  byte-identical.
- Public API of published crates: record the additive conformance options and
  facade methods. Run dry-run packaging for `oxml-pdf`, `rdocx`, and `rpptx`,
  and assert archive sizes and asset inventory.
- WASM bindings: run both WASM checks and prove the bundled ICC profile adds no
  host-only dependency or binding surface.
- Bundled asset: re-read `docs/hld/15-build-and-toolchain.md`. Verify the ICC
  profile source, licence, checked digest, crate-local location, and packaged
  archive presence.
- External oracle comparison: pin veraPDF 1.30.2 and record exact `2b`, `3b`,
  and `ua1` reports.

## Hash harness

Expected to be unchanged. PDF/A is an explicit new path. The existing ordinary
sample renderer remains byte-identical.

## Implementation checklist

- [x] Add explicit PDF/A-2b and PDF/A-3b options and named errors.
- [x] Preflight every prohibited or required feature before writing.
- [x] Embed deterministic XMP, sRGB output intent, and ICC bytes.
- [x] Preserve F-173 structure in both profiles.
- [x] Add native Word and Presentation entry points without binding expansion.
- [x] Pin and run veraPDF 1.30.2 for `2b`, `3b`, and `ua1`.
- [x] Verify ordinary bytes, WASM graphs, package contents, licence evidence,
      and harness output.

## Open questions

None. The new internal file `crates/oxml-pdf/src/conformance.rs`, bundled asset
`crates/oxml-pdf/assets/sRGB2014.icc`, and
`crates/oxml-pdf/assets/LICENSE-sRGB2014` are approved. The profile digest and
licence are recorded with the asset.
