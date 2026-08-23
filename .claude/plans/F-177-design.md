# F-177, RTF writer

**Status**: approved
**Sprint**: S54
**Size**: M
**Depends on**: F-176

## Problem

The facade has only DOCX path and byte writers at
`crates/rdocx/src/document.rs:813`. It cannot emit the RTF subset introduced by
F-176. The existing document tree already retains source-order body items,
typed run and paragraph formatting, tables, lists, and package-backed images,
but no code maps that structure to RTF header tables, destinations, or escaped
content.

M18 requires symmetric text, formatting, tables, lists, and image support. Any
unsupported typed item must produce a stable diagnostic rather than disappear
silently (`docs/hld/14-development-backlog.md:1451`).

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and "Facade conventions".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/14-development-backlog.md`, "Milestone 18, Format breadth" and
  "F-177, RTF writer".
- Microsoft RTF Specification 1.9.1, the same bounded sections named by the
  F-176 plan.

## Approach

Extend the F-176 `rtf` module and reuse its destination vocabulary and
diagnostic type. Do not add a second RTF model, module, or crate. First walk the
supported document projection to allocate deterministic font, colour, list,
and image references. Then emit the RTF header before body content.

Write explicit formatting groups and resets so state cannot leak between runs,
paragraphs, cells, or rows. Escape backslashes and braces. Encode non-ASCII
text as signed UTF-16 `\uN` code units with one fixed fallback character, using
surrogate pairs for supplementary scalars. Preserve list identity through list
tables and overrides rather than flattening markers into text. Resolve package
image relationships and emit supported image bytes as deterministic `\pict`
hex with goal dimensions converted through the existing truncating unit path.

Expose additive facade methods:

```rust
pub struct RtfWriteResult {
    pub bytes: Vec<u8>,
    pub diagnostics: Vec<RtfDiagnostic>,
}

impl Document {
    pub fn to_rtf_bytes(&self) -> Result<RtfWriteResult>;
    pub fn save_rtf<P: AsRef<Path>>(&self, path: P)
        -> Result<Vec<RtfDiagnostic>>;
}
```

The file method writes only after serialization succeeds. Each unsupported or
lossy source item emits one diagnostic with a stable document location and
keeps supported siblings.

## Rejected alternatives

- A separate writer module duplicates F-176 grammar and destination decisions.
- Writing from `PageFrame` loses editable tables, lists, and formatting.
- Reusing HTML output applies the wrong escaping and semantic model.
- Flattening list markers into text fails the round-trip contract.
- Silent omission violates the milestone diagnostic contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `rtf_writer_escapes_control_characters_and_signed_unicode` | Backslashes, braces, BMP text, supplementary scalars, tabs, and breaks serialize correctly. |
| unit | `rtf_writer_emits_stable_header_tables_before_body` | Font, colour, list, and destination allocation and order are deterministic. |
| unit | `rtf_writer_preserves_truncating_image_goal_dimensions` | EMU to twip conversion uses the existing truncating behavior. |
| round-trip | `rtf_writer_round_trip_preserves_supported_document_content` | F-176 reopens text, formatting, tables, lists, and PNG and JPEG images with equal normalized structure. |
| regression | `rtf_writer_reports_each_lossy_item_without_dropping_supported_siblings` | Unsupported items produce exact diagnostics and supported adjacent items remain. |

The **test gate** is round-trip. A document written to RTF and read back
preserves text, formatting, tables, lists, and images. Build every fixture in
code and add it to the existing integration binary.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Unit conversion**. Read `docs/hld/01-glossary.md`, "Units", and the pinned
  truncation rule in `CLAUDE.md`. Reuse `Emu::to_twips` and test exact positive
  and negative boundaries.
- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add exact RTF header and destination
  order checks, write-read structural preservation, and a DOCX reopen check
  proving existing unmodelled XML behavior is unchanged.
- **Public API of a published crate**. This is additive pre-1.0 `rdocx` API.
  Run the verified workspace package dry run and archive size checks.
- **A new module or file**. F-177 adds none. It reuses the F-176 module whose
  creation needs consolidated approval.

## Hash harness

Expected unchanged, 49 of 49. Existing samples do not export RTF. Any delta
blocks integration and the baseline is not re-recorded.

## Implementation checklist

- [ ] Reuse F-176 grammar, projection, diagnostics, and code-page decisions.
- [ ] Allocate stable font, colour, list, and image references.
- [ ] Emit header tables, formatting resets, Unicode text, tables, and lists.
- [ ] Resolve and emit supported images with truncating goal dimensions.
- [ ] Diagnose unsupported content once at stable document locations.
- [ ] Add the exact round-trip gate and focused regression cases.
- [ ] Run unit, packaging, full verify, and unchanged-harness checks.

## Open questions

- Resolved. Use the proposed diagnostic-bearing byte and atomic path writer.
  The supported F-176 matrix is the complete writer fidelity boundary, with
  diagnostics for everything outside it.
