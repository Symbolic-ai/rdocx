# Current Sprint, S05

**Milestone**: M3 Media.

**Goal**: Stage and prove `oxml-media` as the single owner of image format
sniffing, dimensions, DPI, intrinsic sizing, and media naming. Keep the crate
isolated and unpublished, with every released rdocx dependency and output
unchanged until the post-PowerPoint cutover.

## Spec references

- `docs/hld/03-architecture.md`, for the dependency rule that `oxml-media` is
  a dependency-free format-neutral leaf.
- `docs/hld/04-opc-and-packaging.md`, for the media API, sniffing precedence,
  DPI semantics, native-size calculation, naming contract, and safe header
  parsing invariants.
- `docs/hld/11-migration-plan.md`, for staging `oxml-media` without changing
  released consumers before PowerPoint development is complete.
- `docs/hld/12-testing-strategy.md`, for magic-byte, DPI, truncation-loop, and
  highest-existing-suffix regression coverage.
- `docs/hld/14-development-backlog.md`, for the F-023 through F-026 contracts,
  dependencies, and story test gates.
- `docs/hld/15-build-and-toolchain.md`, for keeping development crates at
  version 0.0.0 and outside the seven-package rdocx publication allowlist.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-023 | oxml-media format sniffing | M | in-progress | codex |
| F-025 | MediaNamer | S | in-progress | codex |
| F-024 | Image probing and DPI | L | in-progress | codex |
| F-026 | native_size with explicit DPI | S | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-023 establishes `ImageFormat` and blocks F-024. F-024 establishes
`ImageInfo` and blocks F-026. F-025 is independent of that parser chain and can
run in parallel once the crate exists. F-027 and F-028 remain deferred to
S32.2, so S05 does not alter released rdocx consumers or the hash baseline.

## Definition of done for this sprint

- `oxml-media` exists at version 0.0.0 with publication disabled and no crate
  dependencies.
- Every supported format is identified from magic bytes, and sniffing a JPEG
  overrides a misleading `.png` extension.
- PNG, JPEG, GIF, BMP, and WebP dimensions and DPI metadata are probed safely,
  including PNG unit modes, JFIF density units, EXIF before SOF, progressive
  JPEG, and every truncated prefix.
- `MediaNamer` allocates after the highest existing numeric suffix rather than
  counting parts.
- `native_size(default_dpi)` uses declared DPI when present and the explicit
  caller default otherwise.
- The full workspace, package, supply-chain, and hash gates pass with all 28
  existing hash entries unchanged.
- No `oxml-*` or `rpptx*` development crate is published, and no released
  rdocx dependency changes.
