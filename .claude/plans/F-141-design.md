# F-141, to_pdf in the browser

**Status**: completed
**Sprint**: S35
**Size**: M
**Depends on**: F-139, F-001

## Problem

The current WASM wrapper exposes DOCX, HTML, Markdown, and text methods but no
PDF method. The real `rdocx::Document` already supplies the shared layout and
PDF path, including embedded document fonts and bundled fallbacks. After F-139
removes system-font discovery from the WASM graph, that normal facade method is
the browser-safe bundled-font implementation.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The concurrency model" and shared PDF output.
- `docs/hld/10-bindings-spec.md`, "WASM".
- `docs/hld/12-testing-strategy.md`, "Binding tests" and "Gaps being closed".
- `docs/hld/14-development-backlog.md`, "F-141, to_pdf in the browser".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and "Feature flags".

## Approach

Add exactly `toPdf()` to `WasmDocument`, returning bytes and delegating to
`Document::to_pdf()`. Reuse F-139's concrete error mapper. Do not add base64,
a deterministic alias, or a WASM-only renderer.

Add a crate-root wasm-bindgen test that invokes the exported JavaScript method
through reflection and asserts a complete `%PDF-` through `%%EOF` file,
`/Subtype /Type0`, `/FontFile2`, and a bundled Carlito base font. This tests the
public JS name, real facade rendering path, and actual embedded font stream.

## Rejected alternatives

- Reimplement layout or PDF assembly in the binding. That recreates the fork
  removed by F-139.
- Add `toPdfDeterministic`. The WASM profile already excludes system fonts.
- Assert non-empty bytes only. That does not prove PDF completeness or embedded
  fonts.
- Compare complete PDF bytes. Structural assertions cover the contract without
  a brittle golden baseline.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `to_pdf_in_node_returns_a_complete_pdf_with_an_embedded_bundled_font` | The exported `toPdf` method returns a complete PDF with an embedded bundled font |
| unit | bundled-font legal inventory | Every embedded family retains its checked-in licence and notice |
| regression | existing deterministic rendering tests | The wrapper reuses rather than alters the native rendering path |

The test gate is the backlog requirement that a wasm-pack Node test produces a
non-empty PDF with embedded fonts. Sensitivity removes or corrupts `toPdf`,
proves the exact Node gate fails, restores byte-identically, and reruns green.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout, pagination, and text shaping. Read HLD08. Use bundled deterministic
  fonts and do not create or update a PDF-byte baseline.
- Bundled fonts. Read HLD15. Run the exact font and legal-file inventory and
  the no-default-features gate.
- WASM binding. Read HLD10. Run the wasm32 check and Node suite with the Python
  binding exclusions retained in workspace commands.
- No new file, crate, trait, generic, feature, or published-crate API.

## Hash harness

Expected unchanged. The method is additive on an unpublished binding and calls
the existing renderer.

## Implementation checklist

- [x] Add the byte-returning `toPdf` facade delegation.
- [x] Add the reflective Node PDF and embedded-font regression inline.
- [x] Prove the system-font feature is absent from the WASM graph.
- [x] Run font, rendering, WASM, and hash riders.

## Open questions

None. `toPdf` delegates to normal `Document::to_pdf()`, which F-139 makes
bundled-font-only in the WASM profile.
