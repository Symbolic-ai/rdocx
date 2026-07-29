# F-001, Deterministic font mode

**Status**: completed
**Sprint**: S01
**Size**: M
**Depends on**: none

## Problem

`FontManager::new()` always calls `load_system_fonts()` at
`crates/rdocx-layout/src/font.rs:84`, with the call itself at line 93. Rendered
output therefore depends on fonts installed on the host, so the PNG hashes that
F-003 needs cannot reproduce across machines.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".
- `docs/hld/11-migration-plan.md`, "The safety net comes first".

## Approach

Add `FontManager::new_deterministic() -> Result<Self>` that loads bundled font
bytes and never calls `load_system_fonts()`. It returns a layout error when the
`bundled-fonts` feature is disabled rather than creating a manager that will
fail later during resolution.

Correct the `rdocx-layout` manifest so `bundled-fonts` is default-on, matching
the existing code comments and HLD. The `rdocx` facade is the existing named
consumer that requires bundled fallback fonts in a normal build.

Thread that mode through `Engine::new_deterministic()`, a
`layout_document_deterministic()` entry point, and
`Document::render_page_to_png_deterministic()`. Keep all existing constructors
and rendering methods unchanged so library users retain system-font discovery.

## Rejected alternatives

- An environment variable was rejected because ambient process state would
  make a supposedly deterministic API implicit.
- Making deterministic mode the default was rejected because library users
  expect installed and embedded fonts to remain available.
- Supplying bundled fonts through `to_pdf_with_fonts` was rejected because the
  existing engine still loads system fonts and can select them as fallbacks.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `deterministic_font_manager_uses_only_bundled_fonts` | The deterministic constructor resolves bundled fallbacks without loading the system database. |
| unit | `deterministic_font_manager_requires_bundled_fonts` | The no-default-features build returns an error instead of silently using system fonts. |
| golden | `deterministic_render_is_independent_of_system_fonts` | Rendering the same document through deterministic mode produces identical PNG bytes regardless of system-font availability. |

The **test gate** is `deterministic_render_is_independent_of_system_fonts`:
rendering the same document twice with system fonts installed and absent
produces identical PNG bytes.

## HLD impact

- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout and text shaping. Read `docs/hld/08-rendering-spec.md`. Run the PNG
  equality test in deterministic mode and do not record a system-font baseline.
- Bundled fonts. Read `docs/hld/15-build-and-toolchain.md`. Verify every loaded
  family has its real licence and run
  `cargo test -p rdocx-layout --no-default-features`.
- Default feature correction. The existing consumer is `rdocx`, whose normal
  rendering path already documents bundled fallback fonts. Run
  `cargo test -p rdocx-layout --no-default-features` to exercise the feature-off
  path and confirm `rdocx-layout` defaults to `bundled-fonts` with
  `cargo tree -e features -p rdocx-layout`.
- Public API of published crates. Read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State the additive semver impact, run
  `cargo publish --dry-run` for the affected crates, and inspect their package
  sizes.

## Hash harness

Expected to be unchanged. This story adds an opt-in rendering path, while the
initial baseline is owned by F-003.

## Implementation checklist

- [x] Add the deterministic `FontManager` constructor and its feature-off error.
- [x] Correct the manifest so `bundled-fonts` is default-on for the existing
  `rdocx` consumer.
- [x] Thread deterministic construction through the layout engine.
- [x] Expose deterministic page-one PNG rendering through `Document`.
- [x] Add focused unit and golden tests.
- [x] Run the risk-routing checks and record the exact evidence.

## Open questions

None. The additive public methods needed to carry deterministic mode from
`Document` to `FontManager` are approved.
