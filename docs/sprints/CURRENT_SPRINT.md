# Current Sprint, S35

**Milestone**: M13 Bindings and tooling.

**Goal**: Make both WASM packages thin wrappers around the real Rust facades
and keep them continuously exercised by CI. Replace the destructive rdocx
mini-model, add browser PDF output with bundled fonts, and introduce rpptx WASM
profiles with a bounded default package.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the requirement that both libraries
  ship supported WASM modules alongside their Rust, CLI, and Python surfaces.
- `docs/hld/03-architecture.md`, for facade ownership and the dependency
  direction that keeps the WASM packages as consumers of the real libraries.
- `docs/hld/08-rendering-spec.md`, for the shared document-to-PDF rendering path
  that browser PDF output must reuse.
- `docs/hld/10-bindings-spec.md`, for the destructive current rdocx behavior,
  preserved JavaScript method names, bundled-font browser rendering, CI gate,
  and two rpptx feature profiles.
- `docs/hld/12-testing-strategy.md`, for package-preserving behavioral tests,
  wasm target checks, and the regression gap this sprint closes.
- `docs/hld/13-risks-and-open-questions.md`, for the carried rdocx-wasm save-path
  defect that discards package parts.
- `docs/hld/14-development-backlog.md`, for F-139 through F-142 dependencies
  and their named acceptance gates.
- `docs/hld/15-build-and-toolchain.md`, for the `system-fonts` isolation,
  bundled-font WASM path, target checks, and unpublished package boundary.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-139 | Rewrite rdocx-wasm | L | pending | - |
| F-142 | rpptx-wasm | M | pending | - |
| F-140 | wasm CI job | S | pending | - |
| F-141 | to_pdf in the browser | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not by F-ID. F-139 and F-142 can start
independently because their dependencies F-029 and F-116 are complete. F-140
and F-141 both follow F-139 so the CI and browser-rendering gates exercise the
rewritten facade wrapper rather than the destructive mini-model.

## Definition of done for this sprint

- An rdocx document with images, headers, and numbering round-trips through
  `fromBytes` and `toDocxBytes` with every package part intact.
- Pull requests run both the `wasm32-unknown-unknown` target check and
  `wasm-pack test --node`.
- A Node WASM test produces a non-empty PDF with embedded bundled fonts.
- The default rpptx WASM profile is under 1 MiB gzipped and round-trips a deck,
  while the render profile uses the real facade and rendering stack.
- The reviewed hosted wheel run demonstrates that both Python wheels install
  and pass their parity suites on every target platform in the M13 matrix.
- WASM-focused gates pass without regressing the 28 deterministic document
  hashes or introducing forbidden dependency edges.
