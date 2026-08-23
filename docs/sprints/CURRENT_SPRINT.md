# Current Sprint, S54

**Milestone**: M18 Format breadth.

**Goal**: open M18 with the inbound format that blocks the most corpora by
reading and writing the RTF subset Word emits. Add shared image export controls
and honor document-facing family aliases supplied with caller fonts without
duplicating font bytes or weakening reusable-engine cache identity.

## Spec references

- `docs/hld/03-architecture.md`, for the Word facade, reusable layout engine,
  caller-font context, and ownership boundaries shared by the new format paths.
- `docs/hld/08-rendering-spec.md`, for backend-neutral page output and the
  raster behavior that image export options must preserve.
- `docs/hld/10-bindings-spec.md`, for native rendering entry points,
  caller-supplied font APIs, and unchanged Python and WASM boundaries.
- `docs/hld/12-testing-strategy.md`, for differential RTF evidence,
  round-trip fidelity, named regressions, and deterministic output gates.
- `docs/hld/14-development-backlog.md`, for the M18 goal and the exact scope,
  dependencies, and test gates of all four stories.
- `docs/hld/15-build-and-toolchain.md`, for deterministic caller-font
  isolation, WASM checks, packaging, and the unchanged hash-harness contract.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-176 | RTF reader | L | pending | - |
| F-177 | RTF writer | M | pending | - |
| F-183 | Image export options | S | pending | - |
| F-X051 | Honor caller-supplied font family aliases | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-176 establishes the grammar, destinations, code pages, and typed document
projection that F-177 writes back, so the writer starts only after the reader
contract is stable. F-183 is independent and rides along because every format
in M18 shares the image export entry points. F-X051 is also independent of RTF
work because its reusable-engine dependency, F-X043, is already complete. It
reimplements Issue 44 and PR 45 against the current bounded cache contracts.

## Definition of done for this sprint

- The RTF reader handles the Word-written subset for text, formatting, tables,
  lists, images, destinations, and code pages.
- An RTF fixture converted here to DOCX matches the pinned oracle conversion
  structurally at the declared differential boundary.
- The RTF writer preserves the same supported content when its output is read
  back, with every lossy conversion naming what it dropped in a diagnostic.
- Image entry points support multi-page TIFF, JPEG quality, transparent PNG
  backgrounds, and page ranges that select exactly the requested pages.
- Caller font resolution tries exact embedded families before byte-free caller
  aliases, then retains the existing mapped and generic fallback order.
- Unchanged aliases reuse safe work, changed aliases invalidate only affected
  resolution state, and warm and cold pages, fonts, diagnostics, and provenance
  remain equal.
- Issue 44 and PR 45 land through the hardened reusable-engine path with
  `@emptinessform` retained for credit in the next release containing the work.
- Both WASM targets pass and the deterministic hash harness remains unchanged.
